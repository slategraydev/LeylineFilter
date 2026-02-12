// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

use crate::core::chain::SignalChain;
use crate::core::modules::ModuleFactory;
use crate::core::traits::{AudioModule, EngineCommand, EngineState, ModuleConfig};
use crate::error::{EngineError, Result as EngineResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{unbounded, Receiver, Sender};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use rubato::{FastFixedIn, PolynomialDegree, Resampler};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use sysinfo::System;

use crate::core::visualizer::VisualizerState;

/// # Internal Command Architecture
/// We use a separate internal command enum to handle non-serializable types like `Box<dyn AudioModule>`.
/// This creates a clear boundary between the serializable public API (`EngineCommand`) and
/// the internal thread-safe message passing.
pub enum InternalEngineCommand {
    UpdateConfig(ModuleConfig),
    AddModule(Box<dyn AudioModule>),
    RemoveModule(String),
    SetParam {
        id: String,
        param: String,
        value: f32,
    },
    Reorder(Vec<String>),
    #[allow(dead_code)]
    MidiEvent(crate::core::traits::MidiMessage),
}

pub struct StreamWrapper(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for StreamWrapper {}

/// # Lock-Free Metrics
/// Atomic metrics allow the UI to poll the engine state without ever locking the audio thread.
///
/// We use relaxed ordering because slight stale-ness (microsecond scale) is acceptable for UI visualization.
pub struct EngineMetrics {
    pub latency_ms: AtomicU32,
    pub input_latency_ms: AtomicU32,
    pub output_latency_ms: AtomicU32,
    pub cpu_usage: AtomicU32,
    pub input_level: AtomicU32,
    pub spectrum: [AtomicU32; 12],
    pub tonality: [AtomicU32; 12],
    pub state_version: AtomicU32,
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self {
            latency_ms: AtomicU32::new(0),
            input_latency_ms: AtomicU32::new(0),
            output_latency_ms: AtomicU32::new(0),
            cpu_usage: AtomicU32::new(0),
            input_level: AtomicU32::new(0),
            spectrum: Default::default(),
            tonality: Default::default(),
            state_version: AtomicU32::new(0),
        }
    }

    fn update_latency(&self, processing_ms: f32) {
        self.latency_ms
            .store(processing_ms.to_bits(), Ordering::Relaxed);
    }

    pub fn update_visualizer_metrics(&self, level: f32, bins: &[f32; 12], tonality: &[f32; 12]) {
        self.input_level.store(level.to_bits(), Ordering::Relaxed);
        for i in 0..12 {
            self.spectrum[i].store(bins[i].to_bits(), Ordering::Relaxed);
            self.tonality[i].store(tonality[i].to_bits(), Ordering::Relaxed);
        }
    }

    pub fn get(&self) -> (f32, f32, f32, [f32; 12], [f32; 12], u32) {
        let mut bins = [0.0f32; 12];
        let mut tonal = [0.0f32; 12];
        for i in 0..12 {
            bins[i] = f32::from_bits(self.spectrum[i].load(Ordering::Relaxed));
            tonal[i] = f32::from_bits(self.tonality[i].load(Ordering::Relaxed));
        }
        (
            f32::from_bits(self.latency_ms.load(Ordering::Relaxed)),
            f32::from_bits(self.cpu_usage.load(Ordering::Relaxed)),
            f32::from_bits(self.input_level.load(Ordering::Relaxed)),
            bins,
            tonal,
            self.state_version.load(Ordering::Relaxed),
        )
    }

    pub fn get_hardware_latencies(&self) -> (f32, f32) {
        (
            f32::from_bits(self.input_latency_ms.load(Ordering::Relaxed)),
            f32::from_bits(self.output_latency_ms.load(Ordering::Relaxed)),
        )
    }
}

/// # The Audio Engine
/// The central coordinator. It owns the CPAL streams and orchestrates the signal chain.
///
/// ## Key Design Principle
/// The Main Thread controls the Engine, but the Audio Thread (inside the stream callback)
/// runs autonomously. We use `crossbeam-channel` to bridge these worlds lock-free.
pub struct AudioEngine {
    input_stream: Option<StreamWrapper>,
    output_stream: Option<StreamWrapper>,
    module_configs: Arc<Mutex<HashMap<String, ModuleConfig>>>,
    // Changed to InternalEngineCommand
    command_tx: Arc<Mutex<Option<Sender<InternalEngineCommand>>>>,
    pub metrics: Arc<EngineMetrics>,
    sys: System,
    pid: sysinfo::Pid,
    sample_rate: f32,
    internal_sample_rate: f32,
    last_latency: f32,
    last_cpu: f32,
    rb_occupancy: Arc<AtomicUsize>,
    module_latency_samples: Arc<AtomicUsize>,
    chain_state: Arc<Mutex<EngineState>>,
    last_cpu_update: Instant,
    // Visualizer Thread State
    vis_thread: Option<std::thread::JoinHandle<()>>,
    vis_running: Arc<AtomicBool>,
    // Garbage Collection for Audio Thread
    garbage_rx: Option<Receiver<Box<dyn AudioModule>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get current PID");

        Self {
            input_stream: None,
            output_stream: None,
            module_configs: Arc::new(Mutex::new(HashMap::new())),
            command_tx: Arc::new(Mutex::new(None)),
            metrics: Arc::new(EngineMetrics::new()),
            sys,
            pid,
            sample_rate: 48000.0,
            internal_sample_rate: 48000.0,
            last_latency: 0.0,
            last_cpu: 0.0,
            rb_occupancy: Arc::new(AtomicUsize::new(0)),
            module_latency_samples: Arc::new(AtomicUsize::new(0)),
            chain_state: Arc::new(Mutex::new(EngineState {
                modules: Vec::new(),
                is_running: false,
                sample_rate: 48000.0,
            })),
            last_cpu_update: Instant::now(),
            vis_thread: None,
            vis_running: Arc::new(AtomicBool::new(false)),
            garbage_rx: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.input_stream.is_some()
    }

    pub fn get_total_latency_ms(&mut self) -> f32 {
        if !self.is_running() {
            return 0.0;
        }

        let processing_ms = f32::from_bits(self.metrics.latency_ms.load(Ordering::Relaxed));

        // Chunk size is based on internal sample rate (10ms of internal rate)
        let chunk_size = (self.internal_sample_rate * 0.01) as usize;
        let chunk_latency_ms = (chunk_size as f32 / self.internal_sample_rate) * 1000.0;

        // Ring buffer contains output samples
        let rb_samples = self.rb_occupancy.load(Ordering::Relaxed);
        let rb_latency_ms = (rb_samples as f32 / self.sample_rate) * 1000.0;

        // Modules operate at internal sample rate
        let mod_samples = self.module_latency_samples.load(Ordering::Relaxed);
        let mod_latency_ms = (mod_samples as f32 / self.internal_sample_rate) * 1000.0;

        let (in_lat, out_lat) = self.metrics.get_hardware_latencies();

        let current_latency =
            processing_ms + chunk_latency_ms + rb_latency_ms + mod_latency_ms + in_lat + out_lat;

        if self.last_latency == 0.0 {
            self.last_latency = current_latency;
        } else {
            self.last_latency = self.last_latency * 0.9 + current_latency * 0.1;
        }

        self.last_latency
    }

    pub fn update_cpu_usage(&mut self) -> f32 {
        if self.last_cpu_update.elapsed() < std::time::Duration::from_millis(200) {
            return self.last_cpu;
        }

        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::Some(&[self.pid]), true);
        self.last_cpu_update = Instant::now();

        if let Some(process) = self.sys.process(self.pid) {
            let raw_cpu = process.cpu_usage() / self.sys.cpus().len() as f32;
            self.last_cpu = self.last_cpu * 0.9 + raw_cpu * 0.1;
            self.metrics
                .cpu_usage
                .store(self.last_cpu.to_bits(), Ordering::Relaxed);
            self.last_cpu
        } else {
            0.0
        }
    }

    pub fn send_command(&self, command: EngineCommand) {
        // Track persistence
        match &command {
            EngineCommand::UpdateConfig(config) => {
                let type_name = match config {
                    ModuleConfig::Expander { .. } => "Expander",
                    ModuleConfig::RNNoise { .. } => "RNNoise",
                    ModuleConfig::Gain { .. } => "Gain",
                    ModuleConfig::Compressor { .. } => "Compressor",
                    ModuleConfig::Filter { .. } => "Filter",
                    ModuleConfig::FX { .. } => "FX",
                    ModuleConfig::Visualizer { .. } => "Visualizer",
                    _ => "Unknown",
                };
                if let Ok(mut configs) = self.module_configs.lock() {
                    configs.insert(type_name.to_string(), config.clone());
                }
            }
            _ => {}
        }

        // Translate public EngineCommand to InternalEngineCommand
        let internal_cmd = match command {
            EngineCommand::UpdateConfig(c) => Some(InternalEngineCommand::UpdateConfig(c)),
            EngineCommand::AddModule { module_type } => {
                // Instantiation happens HERE on the main thread
                if let Some(module) = ModuleFactory::create(&module_type, self.internal_sample_rate)
                {
                    Some(InternalEngineCommand::AddModule(module))
                } else {
                    log::warn!("Failed to create module type: {}", module_type);
                    None
                }
            }
            EngineCommand::RemoveModule { id } => Some(InternalEngineCommand::RemoveModule(id)),
            EngineCommand::SetParam { id, param, value } => {
                Some(InternalEngineCommand::SetParam { id, param, value })
            }
            EngineCommand::Reorder { order } => Some(InternalEngineCommand::Reorder(order)),
            EngineCommand::MidiEvent(m) => Some(InternalEngineCommand::MidiEvent(m)),
        };

        if let Some(cmd) = internal_cmd {
            if let Ok(tx_lock) = self.command_tx.lock() {
                if let Some(tx) = tx_lock.as_ref() {
                    let _ = tx.send(cmd);
                }
            }
        }
    }

    pub fn update_module_config(&self, config: ModuleConfig) {
        self.send_command(EngineCommand::UpdateConfig(config));
    }

    pub fn get_input_devices(&self) -> Vec<String> {
        let host = cpal::default_host();
        match host.input_devices() {
            Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
            Err(_) => vec![],
        }
    }

    pub fn get_output_devices(&self) -> Vec<String> {
        let host = cpal::default_host();
        match host.output_devices() {
            Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
            Err(_) => vec![],
        }
    }

    pub fn get_engine_state(&self) -> EngineState {
        if let Ok(state) = self.chain_state.lock() {
            state.clone()
        } else {
            EngineState {
                modules: Vec::new(),
                is_running: false,
                sample_rate: self.sample_rate,
            }
        }
    }

    /// # Garbage Collection
    /// Critical for Audio Safety: Memory deallocation (`Drop`) is non-deterministic and can block.
    /// The Audio Thread sends dropped modules to a channel, and this method (running on Main Thread)
    /// processes the actual deallocation safely.
    pub fn process_garbage(&self) {
        if let Some(rx) = &self.garbage_rx {
            // Drain the channel and drop modules on this thread (Main Thread)
            while let Ok(_) = rx.try_recv() {
                // Module is dropped here
            }
        }
    }

    pub fn start(
        &mut self,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
    ) -> EngineResult<()> {
        let host = cpal::default_host();

        let input_device = if let Some(name) = input_device_name {
            if name == "Default" {
                host.default_input_device()
            } else {
                host.input_devices()?
                    .find(|d| d.name().ok().as_ref() == Some(&name))
            }
        } else {
            host.default_input_device()
        }
        .ok_or_else(|| EngineError::DeviceError("Input device not found".to_string()))?;

        let output_device = if let Some(name) = output_device_name {
            if name == "Default" {
                host.default_output_device()
            } else {
                host.output_devices()?
                    .find(|d| d.name().ok().as_ref() == Some(&name))
            }
        } else {
            host.default_output_device()
        }
        .ok_or_else(|| EngineError::DeviceError("Output device not found".to_string()))?;

        let input_config: cpal::StreamConfig = input_device.default_input_config()?.into();
        let output_config: cpal::StreamConfig = output_device.default_output_config()?.into();

        let in_sample_rate = input_config.sample_rate.0 as f64;
        let out_sample_rate = output_config.sample_rate.0 as f64;
        let in_channels = input_config.channels as usize;

        self.sample_rate = out_sample_rate as f32;

        // Internal processing sample rate setup
        // We normalize processing to a single rate (usually 48kHz, or highest requirement) to simplify DSP
        let mut internal_sample_rate = self.sample_rate;
        let mut chain = SignalChain::new(internal_sample_rate);

        // Initial population of modules
        for module_type in ModuleFactory::available_types() {
            if let Some(m) = ModuleFactory::create(module_type, internal_sample_rate) {
                chain.add_module(m);
            }
        }

        // Check requirements
        for m in chain.modules().iter() {
            if let (Some(req_rate), _) = m.requirements() {
                if req_rate > internal_sample_rate {
                    internal_sample_rate = req_rate;
                }
            }
        }

        self.internal_sample_rate = internal_sample_rate;
        chain.set_sample_rate(internal_sample_rate);

        // Apply persisted configs
        if let Ok(configs) = self.module_configs.lock() {
            for (_, config) in configs.iter() {
                chain.update_config(config);
            }
        }

        // Setup Command Channel
        let (tx, rx) = unbounded::<InternalEngineCommand>();
        if let Ok(mut tx_lock) = self.command_tx.lock() {
            *tx_lock = Some(tx);
        }

        // Setup Garbage Collection Channel
        let (garbage_tx, garbage_rx) = unbounded::<Box<dyn AudioModule>>();
        self.garbage_rx = Some(garbage_rx);

        // Shared State
        let metrics = self.metrics.clone();
        let metrics_out = self.metrics.clone();
        let rb_occ_in = self.rb_occupancy.clone();
        let rb_occ_out = self.rb_occupancy.clone();
        let mod_lat_atomic = self.module_latency_samples.clone();

        let initial_lat = chain.latency_samples();
        mod_lat_atomic.store(initial_lat, Ordering::Relaxed);

        if let Ok(mut state_lock) = self.chain_state.lock() {
            *state_lock = chain.get_state(true);
        }

        // Setup Audio Ring Buffer
        let rb = HeapRb::<f32>::new((self.sample_rate as usize * 2).max(4800 * 2));
        let (mut producer, mut consumer) = rb.split();

        // Setup Visualizer Ring Buffer & Thread
        let vis_rb = HeapRb::<f32>::new(8192);
        let (mut vis_prod, mut vis_cons) = vis_rb.split();

        self.vis_running.store(true, Ordering::Relaxed);
        let vis_running_flag = self.vis_running.clone();
        let vis_metrics = self.metrics.clone();
        let vis_rate = internal_sample_rate;

        // --- Visualizer Thread ---
        // Decoupled from audio thread to prevent lock contention.
        // It consumes a copy of the signal for FFT analysis.
        self.vis_thread = Some(std::thread::spawn(move || {
            let vis_chunk_size = 2048;
            let mut visualizer = VisualizerState::new(vis_chunk_size);
            let mut buffer = Vec::with_capacity(vis_chunk_size);

            while vis_running_flag.load(Ordering::Relaxed) {
                // Drain ring buffer into local buffer
                while let Some(sample) = vis_cons.try_pop() {
                    buffer.push(sample);
                    if buffer.len() >= vis_chunk_size {
                        visualizer.process(&buffer, vis_rate, &vis_metrics);
                        buffer.clear();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
        }));

        // Engine-level Resampler In
        let mut resampler_in = if (in_sample_rate - internal_sample_rate as f64).abs() > 1.0 {
            Some(
                FastFixedIn::<f32>::new(
                    internal_sample_rate as f64 / in_sample_rate,
                    2.0,
                    PolynomialDegree::Cubic,
                    (internal_sample_rate * 0.01) as usize,
                    1,
                )
                .map_err(|e| EngineError::ResamplerError(e.to_string()))?,
            )
        } else {
            None
        };

        // Engine-level Resampler Out
        let mut resampler_out = if (internal_sample_rate as f64 - out_sample_rate).abs() > 1.0 {
            Some(
                FastFixedIn::<f32>::new(
                    out_sample_rate / internal_sample_rate as f64,
                    2.0,
                    PolynomialDegree::Cubic,
                    (internal_sample_rate * 0.01) as usize,
                    1,
                )
                .map_err(|e| EngineError::ResamplerError(e.to_string()))?,
            )
        } else {
            None
        };

        let internal_chunk_size = (internal_sample_rate * 0.01) as usize;
        let mut working_chunk = vec![0.0f32; internal_chunk_size];
        let mut input_accumulator = Vec::with_capacity((in_sample_rate as f32 * 0.1) as usize);
        let mut internal_accumulator = Vec::with_capacity((internal_sample_rate * 0.1) as usize);
        let mut output_accumulator = Vec::with_capacity((out_sample_rate as f32 * 0.1) as usize);
        let mut resample_buf = vec![vec![0.0f32; 0]];

        let chain_state_clone = self.chain_state.clone();

        // --- INPUT STREAM (Audio Thread) ---
        // This closure runs on the high-priority audio thread.
        // MALLOC/FREE FORBIDDEN. LOCKING FORBIDDEN.
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                let start_time = Instant::now();

                // 1. Process Command Queue (Lock-Free)
                let mut chain_changed = false;
                while let Ok(command) = rx.try_recv() {
                    match command {
                        InternalEngineCommand::UpdateConfig(config) => {
                            chain.update_config(&config);
                            chain_changed = true;
                        }
                        InternalEngineCommand::AddModule(module) => {
                            // Module is already boxed and created!
                            chain.add_module(module);
                            chain_changed = true;
                        }
                        InternalEngineCommand::RemoveModule(id) => {
                            if let Some(module) = chain.remove_module(&id) {
                                // Send back to main thread for dropping
                                let _ = garbage_tx.send(module);
                            }
                            chain_changed = true;
                        }
                        InternalEngineCommand::SetParam { id, param, value } => {
                            chain.update_module_param(&id, &param, value);
                            chain_changed = true;
                        }
                        InternalEngineCommand::Reorder(order) => {
                            chain.reorder(&order);
                            chain_changed = true;
                        }
                        InternalEngineCommand::MidiEvent(_midi) => {}
                    }
                }

                if chain_changed {
                    let total_lat = chain.latency_samples();
                    mod_lat_atomic.store(total_lat, Ordering::Relaxed);
                    // Try-lock for UI sync (best effort)
                    if let Ok(mut state_lock) = chain_state_clone.try_lock() {
                        *state_lock = chain.get_state(true);
                        metrics.state_version.fetch_add(1, Ordering::Relaxed);
                    }
                }

                if let Some(diff) = info
                    .timestamp()
                    .callback
                    .duration_since(&info.timestamp().capture)
                {
                    metrics
                        .input_latency_ms
                        .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                }

                for frame in data.chunks(in_channels) {
                    input_accumulator.push(frame[0]);
                }

                // 2. Resample In
                if let Some(ref mut rs) = resampler_in {
                    while input_accumulator.len() >= rs.input_frames_next() {
                        let len = rs.input_frames_next();
                        resample_buf[0] = input_accumulator.drain(..len).collect();
                        if let Ok(resampled) = rs.process(&resample_buf, None) {
                            internal_accumulator.extend_from_slice(&resampled[0]);
                        }
                    }
                } else {
                    internal_accumulator.extend_from_slice(&input_accumulator);
                    input_accumulator.clear();
                }

                // 3. Process in Chunks
                while internal_accumulator.len() >= internal_chunk_size {
                    for (i, sample) in internal_accumulator
                        .drain(..internal_chunk_size)
                        .enumerate()
                    {
                        working_chunk[i] = sample;
                    }

                    // --- THE CORE DSP CHAIN ---
                    chain.process(&mut working_chunk);

                    // Send to visualizer thread (non-blocking push)
                    vis_prod.push_slice(&working_chunk);

                    // 4. Resample Out
                    if let Some(ref mut rs) = resampler_out {
                        resample_buf[0].clear();
                        resample_buf[0].extend_from_slice(&working_chunk);
                        if let Ok(resampled) = rs.process(&resample_buf, None) {
                            output_accumulator.extend_from_slice(&resampled[0]);
                        }
                    } else {
                        output_accumulator.extend_from_slice(&working_chunk);
                    }

                    // Push to output ringbuffer
                    producer.push_slice(&output_accumulator);
                    output_accumulator.clear();

                    rb_occ_in.store(producer.occupied_len(), Ordering::Relaxed);
                    let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
                    metrics.update_latency(elapsed);
                }
            },
            |err| log::error!("Input stream error: {}", err),
            None,
        )?;

        // --- OUTPUT STREAM ---
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                if let Some(diff) = info
                    .timestamp()
                    .playback
                    .duration_since(&info.timestamp().callback)
                {
                    metrics_out
                        .output_latency_ms
                        .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                }

                let out_channels = output_config.channels as usize;
                let frames_needed = data.len() / out_channels;

                for i in 0..frames_needed {
                    let sample = consumer.try_pop().unwrap_or(0.0);
                    for c in 0..out_channels {
                        data[i * out_channels + c] = sample;
                    }
                }
                rb_occ_out.store(consumer.occupied_len(), Ordering::Relaxed);
            },
            |err| log::error!("Output stream error: {}", err),
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(StreamWrapper(input_stream));
        self.output_stream = Some(StreamWrapper(output_stream));

        log::info!(
            "Audio engine started successfully (Internal Rate: {}Hz)",
            internal_sample_rate
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.is_running() {
            self.input_stream = None;
            self.output_stream = None;
            if let Ok(mut tx_lock) = self.command_tx.lock() {
                *tx_lock = None;
            }

            if let Ok(mut state_lock) = self.chain_state.lock() {
                state_lock.is_running = false;
            }

            // Stop visualizer
            self.vis_running.store(false, Ordering::Relaxed);
            if let Some(handle) = self.vis_thread.take() {
                let _ = handle.join();
            }

            log::info!("Audio engine stopped");
        }
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization() {
        let engine = AudioEngine::new();
        assert!(engine.input_stream.is_none());
        assert!(engine.output_stream.is_none());
        assert!(!engine.is_running());
    }

    // Integration test for pipeline logic
    #[test]
    fn test_audio_pipeline_throughput() {
        use crate::core::chain::SignalChain;
        // This test simulates the processing load of the chain without hardware
        let mut chain = SignalChain::new(48000.0);
        // We need a dummy module, Gain is good
        if let Some(m) = ModuleFactory::create("Gain", 48000.0) {
            chain.add_module(m);
        }

        // Simulate 1 second of audio at 48kHz
        let mut buffer = vec![0.0; 48000];
        let start = std::time::Instant::now();

        // Process in 10ms chunks (480 samples)
        for chunk in buffer.chunks_mut(480) {
            chain.process(chunk);
        }

        let elapsed = start.elapsed();
        // Should be VERY fast on any modern machine (microsecond/millisecond scale)
        // Definitely under 100ms
        assert!(
            elapsed.as_millis() < 100,
            "Processing took too long: {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_garbage_collection_mechanism() {
        let mut engine = AudioEngine::new();
        let (tx, rx) = unbounded();
        engine.garbage_rx = Some(rx);

        // Create a dummy module and send it to the garbage channel
        if let Some(m) = ModuleFactory::create("Gain", 48000.0) {
            tx.send(m).unwrap();
        }

        // Channel should not be empty
        assert!(!engine.garbage_rx.as_ref().unwrap().is_empty());

        // Run garbage collector
        engine.process_garbage();

        // Channel should be empty now
        assert!(engine.garbage_rx.as_ref().unwrap().is_empty());
    }
}
