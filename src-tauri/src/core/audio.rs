use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use crate::core::traits::{AudioModule, ModuleConfig};
use crate::core::modules::expander::ExpanderModule;
use crate::core::modules::rnnoise::RNNoiseModule;
use crate::error::{EngineError, Result as EngineResult};
use rubato::{Resampler, FastFixedIn, PolynomialDegree};
use std::time::Instant;
use ringbuf::HeapRb;
use ringbuf::traits::{Producer, Consumer, Split, Observer};
use sysinfo::System;
use rustfft::{FftPlanner, Fft, num_complex::Complex};
use crossbeam_channel::{Sender, unbounded};
use std::collections::HashMap;

/// Pre-allocated state for audio visualization to ensure real-time safety.
struct VisualizerState {
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    scratch_buffer: Vec<Complex<f32>>,
}

impl VisualizerState {
    fn new(chunk_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(chunk_size);
        Self {
            fft_buffer: vec![Complex::default(); chunk_size],
            scratch_buffer: vec![Complex::default(); fft.get_inplace_scratch_len()],
            fft,
        }
    }

    fn process(&mut self, chunk: &[f32], sample_rate: f32, metrics: &EngineMetrics) {
        for (i, &sample) in chunk.iter().enumerate() {
            if i < self.fft_buffer.len() {
                self.fft_buffer[i] = Complex { re: sample, im: 0.0 };
            }
        }

        self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);

        let mut spectrum_bins = [0.0f32; 12];
        let mut tonality_bins = [0.0f32; 12];
        let num_bins = self.fft_buffer.len() / 2;

        if num_bins > 0 {
            let f_min = 80.0f32;
            let f_max = 20000.0f32;
            let bin_sz = sample_rate / chunk.len() as f32;

            for i in 0..12 {
                let freq_start = f_min * (f_max / f_min).powf(i as f32 / 12.0);
                let freq_end = f_min * (f_max / f_min).powf((i + 1) as f32 / 12.0);

                let start_bin = (freq_start / bin_sz).floor() as usize;
                let end_bin = (freq_end / bin_sz).ceil() as usize;

                let start_bin = start_bin.min(num_bins);
                let end_bin = end_bin.max(start_bin + 1).min(num_bins);

                let mut sum = 0.0f32;
                let mut max_bin_val = 0.0f32;
                let count = (end_bin - start_bin).max(1);

                for b in start_bin..end_bin {
                    let val = self.fft_buffer[b].norm();
                    sum += val;
                    if val > max_bin_val {
                        max_bin_val = val;
                    }
                }

                let avg = sum / count as f32;
                let normalized_mag = avg / (num_bins as f32);
                let db = 20.0 * normalized_mag.max(1e-6).log10();
                let linear_val = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
                spectrum_bins[i] = linear_val.sqrt();

                if avg > 1e-6 {
                    let ratio = max_bin_val / avg;
                    let threshold = (count as f32).sqrt().max(2.0);
                    tonality_bins[i] = (ratio / (threshold * 1.5)).min(1.0);
                }
            }
        }

        let mut peak = 0.0f32;
        for &sample in chunk {
            let abs = sample.abs();
            if abs > peak {
                peak = abs;
            }
        }

        metrics.update_processing_metrics(0.0, peak, &spectrum_bins, &tonality_bins);
    }
}

pub struct StreamWrapper(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for StreamWrapper {}

pub struct EngineMetrics {
    pub latency_ms: AtomicU32,
    pub input_latency_ms: AtomicU32,
    pub output_latency_ms: AtomicU32,
    pub cpu_usage: AtomicU32,
    pub input_level: AtomicU32,
    pub spectrum: [AtomicU32; 12],
    pub tonality: [AtomicU32; 12],
}

impl EngineMetrics {
    fn new() -> Self {
        Self {
            latency_ms: AtomicU32::new(0),
            input_latency_ms: AtomicU32::new(0),
            output_latency_ms: AtomicU32::new(0),
            cpu_usage: AtomicU32::new(0),
            input_level: AtomicU32::new(0),
            spectrum: Default::default(),
            tonality: Default::default(),
        }
    }

    fn update_processing_metrics(&self, processing_ms: f32, level: f32, bins: &[f32; 12], tonality: &[f32; 12]) {
        self.latency_ms.store(processing_ms.to_bits(), Ordering::Relaxed);
        self.input_level.store(level.to_bits(), Ordering::Relaxed);
        for i in 0..12 {
            self.spectrum[i].store(bins[i].to_bits(), Ordering::Relaxed);
            self.tonality[i].store(tonality[i].to_bits(), Ordering::Relaxed);
        }
    }

    pub fn get(&self) -> (f32, f32, f32, [f32; 12], [f32; 12]) {
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
        )
    }

    pub fn get_hardware_latencies(&self) -> (f32, f32) {
        (
            f32::from_bits(self.input_latency_ms.load(Ordering::Relaxed)),
            f32::from_bits(self.output_latency_ms.load(Ordering::Relaxed)),
        )
    }
}

pub struct AudioEngine {
    input_stream: Option<StreamWrapper>,
    output_stream: Option<StreamWrapper>,

    /// Cached configurations for modules.
    /// These are used to initialize modules when the engine starts.
    module_configs: Arc<Mutex<HashMap<String, ModuleConfig>>>,

    /// Active channel for sending live updates to the audio thread.
    config_tx: Arc<Mutex<Option<Sender<ModuleConfig>>>>,

    pub metrics: Arc<EngineMetrics>,
    sys: System,
    pid: sysinfo::Pid,
    sample_rate: f32,
    last_latency: f32,
    last_cpu: f32,
    rb_occupancy: Arc<AtomicUsize>,
    module_latency_samples: Arc<AtomicUsize>,
    last_cpu_update: Instant,
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
            config_tx: Arc::new(Mutex::new(None)),
            metrics: Arc::new(EngineMetrics::new()),
            sys,
            pid,
            sample_rate: 48000.0,
            last_latency: 0.0,
            last_cpu: 0.0,
            rb_occupancy: Arc::new(AtomicUsize::new(0)),
            module_latency_samples: Arc::new(AtomicUsize::new(0)),
            last_cpu_update: Instant::now(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.input_stream.is_some()
    }

    pub fn get_total_latency_ms(&mut self) -> f32 {
        if !self.is_running() {
            return 0.0;
        }

        // Processing latency (from metrics)
        let processing_ms = f32::from_bits(self.metrics.latency_ms.load(Ordering::Relaxed));

        // Accumulator latency (10ms)
        let chunk_size = (self.sample_rate * 0.01) as usize;
        let chunk_latency_ms = (chunk_size as f32 / self.sample_rate) * 1000.0;

        // Ring buffer occupancy latency
        let rb_samples = self.rb_occupancy.load(Ordering::Relaxed);
        let rb_latency_ms = (rb_samples as f32 / self.sample_rate) * 1000.0;

        // Module-specific latency (e.g., RNNoise lookahead)
        let mod_samples = self.module_latency_samples.load(Ordering::Relaxed);
        let mod_latency_ms = (mod_samples as f32 / self.sample_rate) * 1000.0;

        // Hardware latencies
        let (in_lat, out_lat) = self.metrics.get_hardware_latencies();

        let current_latency = processing_ms + chunk_latency_ms + rb_latency_ms + mod_latency_ms + in_lat + out_lat;

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

        self.sys.refresh_all();
        self.last_cpu_update = Instant::now();

        if let Some(process) = self.sys.process(self.pid) {
            let raw_cpu = process.cpu_usage() / self.sys.cpus().len() as f32;
            self.last_cpu = self.last_cpu * 0.9 + raw_cpu * 0.1;
            self.metrics.cpu_usage.store(self.last_cpu.to_bits(), Ordering::Relaxed);
            self.last_cpu
        } else {
            0.0
        }
    }

    pub fn update_module_config(&self, config: ModuleConfig) {
        // 1. Update the cached config for future engine restarts
        let type_name = match &config {
            ModuleConfig::Expander { .. } => "Expander",
            ModuleConfig::RNNoise { .. } => "RNNoise",
            _ => "Unknown",
        };

        if let Ok(mut configs) = self.module_configs.lock() {
            configs.insert(type_name.to_string(), config.clone());
        }

        // 2. If the engine is running, send the update to the audio thread
        if let Ok(tx_lock) = self.config_tx.lock() {
            if let Some(tx) = tx_lock.as_ref() {
                let _ = tx.send(config);
            }
        }
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

    pub fn start(&mut self, input_device_name: Option<String>, output_device_name: Option<String>) -> EngineResult<()> {
        let host = cpal::default_host();

        let input_device = if let Some(name) = input_device_name {
            if name == "Default" { host.default_input_device() }
            else { host.input_devices()?.find(|d| d.name().ok().as_ref() == Some(&name)) }
        } else { host.default_input_device() }.ok_or_else(|| EngineError::DeviceError("Input device not found".to_string()))?;

        let output_device = if let Some(name) = output_device_name {
            if name == "Default" { host.default_output_device() }
            else { host.output_devices()?.find(|d| d.name().ok().as_ref() == Some(&name)) }
        } else { host.default_output_device() }.ok_or_else(|| EngineError::DeviceError("Output device not found".to_string()))?;

        let input_config: cpal::StreamConfig = input_device.default_input_config()?.into();
        let output_config: cpal::StreamConfig = output_device.default_output_config()?.into();

        let in_sample_rate = input_config.sample_rate.0 as f64;
        let out_sample_rate = output_config.sample_rate.0 as f64;
        let in_channels = input_config.channels as usize;

        self.sample_rate = out_sample_rate as f32;

        // Initialize local modules for the audio thread
        let mut local_modules: Vec<Box<dyn AudioModule>> = vec![
            Box::new(ExpanderModule::new(self.sample_rate)),
            Box::new(RNNoiseModule::new(self.sample_rate)),
        ];

        // Apply saved configurations
        if let Ok(configs) = self.module_configs.lock() {
            for (_, config) in configs.iter() {
                for module in local_modules.iter_mut() {
                    module.update_config(config);
                }
            }
        }

        // Initialize latency reporting
        let initial_lat: usize = local_modules.iter().map(|m| m.latency_samples()).sum();
        self.module_latency_samples.store(initial_lat, Ordering::Relaxed);

        // Setup communication channel
        let (tx, rx) = unbounded::<ModuleConfig>();
        if let Ok(mut tx_lock) = self.config_tx.lock() {
            *tx_lock = Some(tx);
        }

        let metrics = self.metrics.clone();
        let metrics_out = self.metrics.clone();
        let rb_occ_in = self.rb_occupancy.clone();
        let rb_occ_out = self.rb_occupancy.clone();
        let mod_lat_atomic = self.module_latency_samples.clone();

        let rb = HeapRb::<f32>::new((self.sample_rate as usize * 2).max(4800 * 2));
        let (mut producer, mut consumer) = rb.split();

        let mut resampler = if (in_sample_rate - out_sample_rate).abs() > 1.0 {
            Some(FastFixedIn::<f32>::new(
                out_sample_rate / in_sample_rate,
                2.0,
                PolynomialDegree::Cubic,
                (out_sample_rate as f32 * 0.01) as usize,
                1,
            ).map_err(|e| EngineError::ResamplerError(e.to_string()))?)
        } else {
            None
        };

        let chunk_size = (self.sample_rate * 0.01) as usize;
        let mut visualizer = VisualizerState::new(chunk_size);
        let mut working_chunk = vec![0.0f32; chunk_size];
        let mut input_accumulator = Vec::with_capacity((self.sample_rate * 0.1) as usize);
        let mut process_accumulator = Vec::with_capacity((self.sample_rate * 0.1) as usize);
        let mut resample_input = vec![vec![0.0f32; 0]];

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                let start_time = Instant::now();

                // 1. Check for configuration updates (Lock-Free)
                let mut config_changed = false;
                while let Ok(config) = rx.try_recv() {
                    config_changed = true;
                    for module in local_modules.iter_mut() {
                        module.update_config(&config);
                    }
                }

                if config_changed {
                    let total_lat: usize = local_modules.iter().map(|m| m.latency_samples()).sum();
                    mod_lat_atomic.store(total_lat, Ordering::Relaxed);
                }

                let callback = info.timestamp().callback;
                let capture = info.timestamp().capture;
                if let Some(diff) = callback.duration_since(&capture) {
                    let ms = diff.as_secs_f32() * 1000.0;
                    metrics.input_latency_ms.store(ms.to_bits(), Ordering::Relaxed);
                }

                for frame in data.chunks(in_channels) {
                    input_accumulator.push(frame[0]);
                }

                if let Some(ref mut rs) = resampler {
                    while input_accumulator.len() >= rs.input_frames_next() {
                        let chunk: Vec<f32> = input_accumulator.drain(0..rs.input_frames_next()).collect();
                        resample_input[0] = chunk;
                        if let Ok(resampled) = rs.process(&resample_input, None) {
                            process_accumulator.extend_from_slice(&resampled[0]);
                        }
                    }
                } else {
                    process_accumulator.extend_from_slice(&input_accumulator);
                    input_accumulator.clear();
                }

                let sr = out_sample_rate as f32;
                while process_accumulator.len() >= chunk_size {
                    for (i, sample) in process_accumulator.drain(0..chunk_size).enumerate() {
                        working_chunk[i] = sample;
                    }

                    // 2. Process modules (No Mutex!)
                    for module in local_modules.iter_mut() {
                        module.process(&mut working_chunk);
                    }

                    producer.push_slice(&working_chunk);
                    rb_occ_in.store(producer.occupied_len(), Ordering::Relaxed);
                    visualizer.process(&working_chunk, sr, &metrics);

                    let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
                    metrics.latency_ms.store(elapsed.to_bits(), Ordering::Relaxed);
                }
            },
            |err| log::error!("Input stream error: {}", err),
            None
        )?;

        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                let callback = info.timestamp().callback;
                let playback = info.timestamp().playback;
                if let Some(diff) = playback.duration_since(&callback) {
                    let ms = diff.as_secs_f32() * 1000.0;
                    metrics_out.output_latency_ms.store(ms.to_bits(), Ordering::Relaxed);
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
            None
        )?;

        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(StreamWrapper(input_stream));
        self.output_stream = Some(StreamWrapper(output_stream));

        log::info!("Audio engine started successfully");
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.is_running() {
            self.input_stream = None;
            self.output_stream = None;

            // Clear the sender so we don't try to send to a dead receiver
            if let Ok(mut tx_lock) = self.config_tx.lock() {
                *tx_lock = None;
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

    #[test]
    fn test_stop_idempotency() {
        let mut engine = AudioEngine::new();
        engine.stop();
        assert!(!engine.is_running());
        engine.stop();
        assert!(!engine.is_running());
    }
}
