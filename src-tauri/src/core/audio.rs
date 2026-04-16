// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// AUDIO ENGINE
// Core orchestrator for audio I/O, stream management, and signal chain execution.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
    SetMonitoring(bool),
}

pub struct StreamWrapper(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for StreamWrapper {}

pub type MetricSnapshot = (
    f32,
    f32,
    f32,
    f32,
    u32,
    [f32; 12],
    [f32; 12],
    [f32; 64],
    u32,
);

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
    pub input_level_db: AtomicU32,
    pub buffer_size: AtomicU32,
    pub spectrum: [AtomicU32; 12],
    pub tonality: [AtomicU32; 12],
    pub waveform: [AtomicU32; 64],
    pub state_version: AtomicU32,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self {
            latency_ms: AtomicU32::new(0),
            input_latency_ms: AtomicU32::new(0),
            output_latency_ms: AtomicU32::new(0),
            cpu_usage: AtomicU32::new(0),
            input_level: AtomicU32::new(0),
            input_level_db: AtomicU32::new((-60.0_f32).to_bits()),
            buffer_size: AtomicU32::new(256),
            spectrum: Default::default(),
            tonality: Default::default(),
            waveform: std::array::from_fn(|_| AtomicU32::new(0)),
            state_version: AtomicU32::new(0),
        }
    }

    fn update_latency(&self, processing_ms: f32) {
        self.latency_ms
            .store(processing_ms.to_bits(), Ordering::Relaxed);
    }

    pub fn update_visualizer_metrics(
        &self,
        level: f32,
        bins: &[f32; 12],
        tonality: &[f32; 12],
        waveform: &[f32; 64],
    ) {
        self.input_level.store(level.to_bits(), Ordering::Relaxed);
        let db = 20.0 * level.max(1e-6).log10();
        self.input_level_db.store(db.to_bits(), Ordering::Relaxed);

        for (i, &bin) in bins.iter().enumerate() {
            self.spectrum[i].store(bin.to_bits(), Ordering::Relaxed);
        }
        for (i, &tonal) in tonality.iter().enumerate() {
            self.tonality[i].store(tonal.to_bits(), Ordering::Relaxed);
        }
        for (i, &wave) in waveform.iter().enumerate() {
            self.waveform[i].store(wave.to_bits(), Ordering::Relaxed);
        }
    }

    pub fn get(&self) -> MetricSnapshot {
        let mut bins = [0.0f32; 12];
        let mut tonal = [0.0f32; 12];
        let mut wave = [0.0f32; 64];
        for (i, bin) in bins.iter_mut().enumerate() {
            *bin = f32::from_bits(self.spectrum[i].load(Ordering::Relaxed));
        }
        for (i, tonal_val) in tonal.iter_mut().enumerate() {
            *tonal_val = f32::from_bits(self.tonality[i].load(Ordering::Relaxed));
        }
        for (i, wave_val) in wave.iter_mut().enumerate() {
            *wave_val = f32::from_bits(self.waveform[i].load(Ordering::Relaxed));
        }
        (
            f32::from_bits(self.latency_ms.load(Ordering::Relaxed)),
            f32::from_bits(self.cpu_usage.load(Ordering::Relaxed)),
            f32::from_bits(self.input_level.load(Ordering::Relaxed)),
            f32::from_bits(self.input_level_db.load(Ordering::Relaxed)),
            self.buffer_size.load(Ordering::Relaxed),
            bins,
            tonal,
            wave,
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
    /// Optional second output stream for headphone/speaker monitoring.
    monitor_stream: Option<StreamWrapper>,
    module_configs: Arc<Mutex<HashMap<String, ModuleConfig>>>,
    // Changed to InternalEngineCommand
    command_tx: Arc<Mutex<Option<Sender<InternalEngineCommand>>>>,
    pub metrics: Arc<EngineMetrics>,
    sys: System,
    pid: sysinfo::Pid,
    sample_rate: f32,
    internal_sample_rate: f32,
    buffer_size: u32,
    last_latency: f32,
    last_cpu: f32,
    monitoring_enabled: Arc<AtomicBool>,
    rb_occupancy: Arc<AtomicUsize>,
    module_latency_samples: Arc<AtomicUsize>,
    chain_state: Arc<Mutex<EngineState>>,
    last_cpu_update: Instant,
    // Visualizer Thread State
    vis_thread: Option<std::thread::JoinHandle<()>>,
    vis_running: Arc<AtomicBool>,
    // Garbage Collection for Audio Thread
    garbage_rx: Option<Receiver<Box<dyn AudioModule>>>,
    // Persistent chain for offline management
    offline_chain: Arc<Mutex<SignalChain>>,
    prefill_samples: usize,
    // Device Tracking for Persistence
    input_device_name: Arc<Mutex<Option<String>>>,
    output_device_name: Arc<Mutex<Option<String>>>,
    // Layout Tracking
    pub positions: Arc<Mutex<HashMap<String, crate::core::persistence::GridPosition>>>,
    pub heights: Arc<Mutex<HashMap<String, u32>>>,
    pub widths: Arc<Mutex<HashMap<String, u32>>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    /// Safe wrapper to get the default host, catching potential panics in headless environments.
    fn get_host() -> Option<cpal::Host> {
        std::panic::catch_unwind(cpal::default_host).ok()
    }

    pub fn new() -> Self {
        log::info!("Initializing AudioEngine...");
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        let pid = std::panic::catch_unwind(|| sysinfo::get_current_pid().ok())
            .ok()
            .flatten()
            .unwrap_or(sysinfo::Pid::from(0));

        log::info!("AudioEngine: Querying hardware for initial state...");
        // Query hardware for initial UI state - Safely handle cases where cpal fails in headless CI
        let (initial_sr, initial_bs) =
            if let Some(device) = Self::get_host().and_then(|h| h.default_output_device()) {
                if let Ok(config) = device.default_output_config() {
                    let sr = config.sample_rate().0 as f32;
                    (sr, 256)
                } else {
                    (48000.0, 256)
                }
            } else {
                (48000.0, 256)
            };

        log::info!("AudioEngine: Initial sample rate: {initial_sr}Hz, buffer size: {initial_bs}");
        let offline_chain = Arc::new(Mutex::new(SignalChain::new(initial_sr)));

        Self {
            input_stream: None,
            output_stream: None,
            monitor_stream: None,
            module_configs: Arc::new(Mutex::new(HashMap::new())),
            command_tx: Arc::new(Mutex::new(None)),
            metrics: Arc::new(EngineMetrics::new()),
            sys,
            pid,
            sample_rate: initial_sr,
            internal_sample_rate: initial_sr,
            buffer_size: initial_bs,
            last_latency: 0.0,
            last_cpu: 0.0,
            monitoring_enabled: Arc::new(AtomicBool::new(false)),
            rb_occupancy: Arc::new(AtomicUsize::new(0)),
            module_latency_samples: Arc::new(AtomicUsize::new(0)),
            chain_state: Arc::new(Mutex::new(EngineState {
                modules: Vec::new(),
                is_running: false,
                monitoring_enabled: false,
                sample_rate: initial_sr,
                buffer_size: initial_bs,
                input_device: Some("Default".to_string()),
                output_device: Some("Default".to_string()),
                positions: HashMap::new(),
                heights: HashMap::new(),
                widths: HashMap::new(),
            })),
            last_cpu_update: Instant::now(),
            vis_thread: None,
            vis_running: Arc::new(AtomicBool::new(false)),
            garbage_rx: None,
            offline_chain,
            prefill_samples: 0,
            input_device_name: Arc::new(Mutex::new(Some("Default".to_string()))),
            output_device_name: Arc::new(Mutex::new(Some("Default".to_string()))),
            positions: Arc::new(Mutex::new(HashMap::new())),
            heights: Arc::new(Mutex::new(HashMap::new())),
            widths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_persistence_config(&self) -> crate::core::persistence::AppConfig {
        let in_dev = self.input_device_name.lock().unwrap().clone();
        let out_dev = self.output_device_name.lock().unwrap().clone();
        let mon_en = self.monitoring_enabled.load(Ordering::Relaxed);
        let running = self.is_running();

        let (modules, layout) = if let Ok(offline) = self.offline_chain.lock() {
            let current_layout = crate::core::persistence::LayoutConfig {
                positions: self.positions.lock().unwrap().clone(),
                heights: self.heights.lock().unwrap().clone(),
                widths: self.widths.lock().unwrap().clone(),
            };
            let state = offline.get_state(
                running,
                self.buffer_size,
                mon_en,
                in_dev.clone(),
                out_dev.clone(),
                current_layout.clone(),
            );
            (state.modules, current_layout)
        } else {
            (
                Vec::new(),
                crate::core::persistence::LayoutConfig::default(),
            )
        };

        crate::core::persistence::AppConfig {
            input_device: in_dev,
            output_device: out_dev,
            monitoring_enabled: mon_en,
            engine_running: running,
            modules,
            layout,
        }
    }

    pub fn apply_persistence_config(&self, config: crate::core::persistence::AppConfig) {
        *self.input_device_name.lock().unwrap() = config.input_device.clone();
        *self.output_device_name.lock().unwrap() = config.output_device.clone();
        *self.positions.lock().unwrap() = config.layout.positions.clone();
        *self.heights.lock().unwrap() = config.layout.heights.clone();
        *self.widths.lock().unwrap() = config.layout.widths.clone();
        self.monitoring_enabled
            .store(config.monitoring_enabled, Ordering::Relaxed);

        if let Ok(mut offline) = self.offline_chain.lock() {
            // Clear current chain
            let current_ids: Vec<String> = offline
                .modules()
                .iter()
                .map(|m| m.id().to_string())
                .collect();
            for id in current_ids {
                offline.remove_module(&id);
            }

            // Add modules from config
            for m_info in config.modules {
                if let Some(mut m) = ModuleFactory::create_with_id(
                    &m_info.name,
                    m_info.id,
                    self.internal_sample_rate,
                ) {
                    m.update_config(&m_info.config);
                    offline.add_module(m);
                }
            }

            // Sync state back to UI
            if !self.is_running() {
                if let Ok(mut state_lock) = self.chain_state.lock() {
                    *state_lock = offline.get_state(
                        false,
                        self.buffer_size,
                        config.monitoring_enabled,
                        config.input_device,
                        config.output_device,
                        config.layout,
                    );
                    self.metrics.state_version.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.input_stream.is_some()
    }

    pub fn set_monitoring(&mut self, enabled: bool) {
        self.monitoring_enabled.store(enabled, Ordering::Relaxed);

        if let Ok(mut state_lock) = self.chain_state.lock() {
            state_lock.monitoring_enabled = enabled;
            self.metrics.state_version.fetch_add(1, Ordering::Relaxed);
        }

        // If the engine is running, we need to restart it to actually
        // open or close the hardware monitor stream.
        if self.is_running() {
            let in_dev = self.input_device_name.lock().unwrap().clone();
            let out_dev = self.output_device_name.lock().unwrap().clone();
            let _ = self.start(in_dev, out_dev, None);
        }

        if let Ok(tx_lock) = self.command_tx.lock() {
            if let Some(tx) = tx_lock.as_ref() {
                let _ = tx.send(InternalEngineCommand::SetMonitoring(enabled));
            }
        }
    }

    pub fn get_total_latency_ms(&mut self) -> f32 {
        if !self.is_running() {
            return 0.0;
        }

        let processing_ms = f32::from_bits(self.metrics.latency_ms.load(Ordering::Relaxed));

        // 10ms internal processing chunk duration
        let chunk_latency_ms = 10.0;

        // Correctly subtract the safety pre-fill from the reported latency
        let rb_samples = self
            .rb_occupancy
            .load(Ordering::Relaxed)
            .saturating_sub(self.prefill_samples);
        let rb_latency_ms = (rb_samples as f32 / self.sample_rate.max(1.0)) * 1000.0;

        let mod_samples = self.module_latency_samples.load(Ordering::Relaxed);
        let mod_latency_ms = (mod_samples as f32 / self.internal_sample_rate.max(1.0)) * 1000.0;

        let current_latency =
            (processing_ms + chunk_latency_ms + rb_latency_ms + mod_latency_ms).max(0.0);

        if self.last_latency == 0.0 {
            self.last_latency = current_latency;
        } else {
            // Very smooth alpha for a steady UI display
            self.last_latency = self.last_latency * 0.98 + current_latency * 0.02;
        }

        self.last_latency.round()
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
        // Track persistence in module_configs (legacy, but keeping for now)
        if let EngineCommand::UpdateConfig(config) = &command {
            let type_name = match config {
                ModuleConfig::Expander { .. } => "Expander",
                ModuleConfig::RNNoise { .. } => "RNNoise",
                ModuleConfig::Gain { .. } => "Gain",
                ModuleConfig::Compressor { .. } => "Compressor",
                ModuleConfig::Filter { .. } => "Filter",
                ModuleConfig::ParametricEQ { .. } => "ParametricEQ",
                ModuleConfig::Deesser { .. } => "Deesser",
                ModuleConfig::Saturation { .. } => "Saturation",
                ModuleConfig::Limiter { .. } => "Limiter",
                ModuleConfig::Dereverb { .. } => "Dereverb",
                ModuleConfig::FX { .. } => "FX",
                _ => "Unknown",
            };
            if let Ok(mut configs) = self.module_configs.lock() {
                configs.insert(type_name.to_string(), config.clone());
            }
        }

        // Apply to offline chain (Always keep in sync)
        if let Ok(mut offline) = self.offline_chain.lock() {
            match &command {
                EngineCommand::UpdateConfig(c) => offline.update_config(c),
                EngineCommand::AddModule { module_type } => {
                    if let Some(m) = ModuleFactory::create(module_type, self.internal_sample_rate) {
                        offline.add_module(m);
                    }
                }
                EngineCommand::RemoveModule { id } => {
                    offline.remove_module(id);
                }
                EngineCommand::SetParam { id, param, value } => {
                    offline.update_module_param(id, param, *value);
                }
                EngineCommand::Reorder { order } => {
                    offline.reorder(order);
                }
                _ => {}
            }

            // If engine is NOT running, sync state back to UI immediately.
            // If running, the audio thread will sync state back.
            if !self.is_running() {
                if let Ok(mut state_lock) = self.chain_state.lock() {
                    let is_mon = self.monitoring_enabled.load(Ordering::Relaxed);
                    let in_dev = self.input_device_name.lock().unwrap().clone();
                    let out_dev = self.output_device_name.lock().unwrap().clone();
                    let layout = crate::core::persistence::LayoutConfig {
                        positions: self.positions.lock().unwrap().clone(),
                        heights: self.heights.lock().unwrap().clone(),
                        widths: self.widths.lock().unwrap().clone(),
                    };
                    *state_lock =
                        offline.get_state(false, self.buffer_size, is_mon, in_dev, out_dev, layout);
                    self.metrics.state_version.fetch_add(1, Ordering::Relaxed);
                }
            }
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
                    log::warn!("Failed to create module type: {module_type}");
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
        if let Some(host) = Self::get_host() {
            match host.input_devices() {
                Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn get_output_devices(&self) -> Vec<String> {
        if let Some(host) = Self::get_host() {
            match host.output_devices() {
                Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn get_engine_state(&self) -> EngineState {
        if let Ok(state) = self.chain_state.lock() {
            state.clone()
        } else {
            EngineState {
                modules: Vec::new(),
                is_running: false,
                monitoring_enabled: self.monitoring_enabled.load(Ordering::Relaxed),
                sample_rate: self.sample_rate,
                buffer_size: self.buffer_size,
                input_device: self.input_device_name.lock().unwrap().clone(),
                output_device: self.output_device_name.lock().unwrap().clone(),
                positions: self.positions.lock().unwrap().clone(),
                heights: self.heights.lock().unwrap().clone(),
                widths: self.widths.lock().unwrap().clone(),
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
            while let Ok(module) = rx.try_recv() {
                let m_name = module.name();
                let m_id = module.id();
                log::info!("Garbage collecting module: {m_name} ({m_id})");
                drop(module);
            }
        }
    }

    pub fn start(
        &mut self,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
        _unused_monitor: Option<String>,
    ) -> EngineResult<()> {
        if self.is_running() {
            log::info!("Restarting engine for device swap...");
            self.stop();
        }

        let host = Self::get_host().ok_or_else(|| {
            EngineError::DeviceError("No audio host available (drivers missing or CI)".to_string())
        })?;

        log::info!("Using audio host: {:?}", host.id());

        let input_device = if let Some(ref name) = input_device_name {
            if name == "Default" {
                host.default_input_device()
            } else {
                host.input_devices()?
                    .find(|d| d.name().ok().as_ref() == Some(name))
            }
        } else {
            host.default_input_device()
        }
        .ok_or_else(|| EngineError::DeviceError("Input device not found".to_string()))?;

        let output_device = if let Some(ref name) = output_device_name {
            if name == "Default" {
                host.default_output_device()
            } else {
                host.output_devices()?
                    .find(|d| d.name().ok().as_ref() == Some(name))
            }
        } else {
            host.default_output_device()
        }
        .ok_or_else(|| EngineError::DeviceError("Output device not found".to_string()))?;

        let actual_in = input_device
            .name()
            .unwrap_or_else(|_| "Unknown".to_string());
        let actual_out = output_device
            .name()
            .unwrap_or_else(|_| "Unknown".to_string());

        // Loud diagnostics to identify if we are hitting System Default by accident
        println!("!!! [ENGINE START] !!!");
        println!("Requested Input:   {input_device_name:?}");
        println!("Requested Output:  {output_device_name:?}");
        println!("Actual Output:     {actual_out}");
        println!("Primary Input:     {actual_in}");

        log::info!("Selected Input Device: {actual_in}");
        log::info!("Selected Output Device: {actual_out}");

        // Update tracked device names for persistence
        *self.input_device_name.lock().unwrap() = input_device_name.clone();
        *self.output_device_name.lock().unwrap() = output_device_name.clone();

        let input_default = input_device.default_input_config()?;
        let output_default = output_device.default_output_config()?;

        // Prefer F32 natively, but fall back to any supported integer format.
        // Clamp to the device's default sample rate, not the maximum, to avoid
        // virtual devices (e.g. VB-Audio CABLE) rejecting the stream.
        let find_f32_config = |supported: &cpal::SupportedStreamConfigRange,
                               default_rate: cpal::SampleRate| {
            let clamped = supported
                .max_sample_rate()
                .0
                .min(default_rate.0)
                .max(supported.min_sample_rate().0);
            (*supported).with_sample_rate(cpal::SampleRate(clamped))
        };

        let in_default_rate = input_default.sample_rate();
        let out_default_rate = output_default.sample_rate();

        let input_config_supported = if input_default.sample_format() == cpal::SampleFormat::F32 {
            input_default
        } else {
            match input_device.supported_input_configs() {
                Ok(mut cfgs) => cfgs
                    .find(|c| c.sample_format() == cpal::SampleFormat::F32)
                    .map(|c| find_f32_config(&c, in_default_rate))
                    .unwrap_or_else(|| {
                        log::warn!(
                            "Input device '{}' has no F32 config; using native {:?}",
                            input_device.name().unwrap_or_default(),
                            input_default.sample_format()
                        );
                        input_default
                    }),
                Err(_) => input_default,
            }
        };

        let output_config_supported = if output_default.sample_format() == cpal::SampleFormat::F32 {
            output_default
        } else {
            match output_device.supported_output_configs() {
                Ok(mut cfgs) => cfgs
                    .find(|c| c.sample_format() == cpal::SampleFormat::F32)
                    .map(|c| find_f32_config(&c, out_default_rate))
                    .unwrap_or_else(|| {
                        log::warn!(
                            "Output device '{}' has no F32 config; using native {:?}",
                            output_device.name().unwrap_or_default(),
                            output_default.sample_format()
                        );
                        output_default
                    }),
                Err(_) => output_default,
            }
        };

        let in_sample_format = input_config_supported.sample_format();
        let out_sample_format = output_config_supported.sample_format();

        log::info!("Final Input Format: {in_sample_format:?}");
        log::info!("Final Output Format: {out_sample_format:?}");

        let mut input_config: cpal::StreamConfig = input_config_supported.into();
        let mut output_config: cpal::StreamConfig = output_config_supported.into();

        // Use Default for maximum compatibility on WASAPI
        input_config.buffer_size = cpal::BufferSize::Default;
        output_config.buffer_size = cpal::BufferSize::Default;

        log::info!("Input Config: {input_config:?}");
        log::info!("Output Config: {output_config:?}");

        let in_sample_rate = input_config.sample_rate.0 as f64;
        let out_sample_rate = output_config.sample_rate.0 as f64;
        let in_channels = input_config.channels as usize;
        // Track formats so the stream builders can dispatch conversion logic.
        let in_fmt = in_sample_format;
        let out_fmt = out_sample_format;

        self.sample_rate = out_sample_rate as f32;

        // Internal processing sample rate setup
        // We normalize processing to a single rate (usually 48kHz, or highest requirement) to simplify DSP
        let mut internal_sample_rate = self.sample_rate;
        let mut chain = SignalChain::new(internal_sample_rate);

        // Populate from offline chain
        if let Ok(offline) = self.offline_chain.lock() {
            let is_mon = self.monitoring_enabled.load(Ordering::Relaxed);
            let in_dev = self.input_device_name.lock().unwrap().clone();
            let out_dev = self.output_device_name.lock().unwrap().clone();
            let layout = crate::core::persistence::LayoutConfig {
                positions: self.positions.lock().unwrap().clone(),
                heights: self.heights.lock().unwrap().clone(),
                widths: self.widths.lock().unwrap().clone(),
            };

            for m_info in offline
                .get_state(false, 0, is_mon, in_dev, out_dev, layout)
                .modules
            {
                // Use the module's name and original ID to re-create it
                if let Some(mut m) =
                    ModuleFactory::create_with_id(&m_info.name, m_info.id, internal_sample_rate)
                {
                    m.update_config(&m_info.config);
                    chain.add_module(m);
                }
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

        let monitoring_enabled_flag = self.monitoring_enabled.clone();

        if let Ok(mut state_lock) = self.chain_state.lock() {
            let is_mon = monitoring_enabled_flag.load(Ordering::Relaxed);
            let layout = crate::core::persistence::LayoutConfig {
                positions: self.positions.lock().unwrap().clone(),
                heights: self.heights.lock().unwrap().clone(),
                widths: self.widths.lock().unwrap().clone(),
            };
            *state_lock = chain.get_state(
                true,
                0,
                is_mon,
                input_device_name.clone(),
                output_device_name.clone(),
                layout,
            ); // Start with 0, will be updated by callback
            state_lock.monitoring_enabled = is_mon;
            self.metrics.state_version.fetch_add(1, Ordering::Relaxed);
        }

        // Setup Audio Ring Buffer
        let rb = HeapRb::<f32>::new((self.sample_rate as usize * 2).max(4800 * 2));
        let (mut producer, mut consumer) = rb.split();

        // Pre-fill with 2 chunks of silence (~20ms) so the output stream never underruns
        // on its very first callback. Virtual devices (e.g. VB-Audio CABLE) fire their
        // output callback almost immediately, before the input side has produced anything.
        let prefill_samples = (internal_sample_rate as usize / 100) * 2; // ~20ms
        self.prefill_samples = prefill_samples;
        for _ in 0..prefill_samples {
            let _ = producer.try_push(0.0);
        }
        log::info!(
            "Ring buffer pre-filled with {} samples (~{:.1}ms) of silence",
            prefill_samples,
            prefill_samples as f32 / out_sample_rate as f32 * 1000.0
        );

        // Secondary ring buffer for the optional monitor/headphone stream.
        // No pre-fill needed; the monitor stream is non-critical and tolerates brief startup silence.
        let mon_rb = HeapRb::<f32>::new((self.sample_rate as usize * 2).max(4800 * 2));
        let (mut mon_producer, mut mon_consumer) = mon_rb.split();
        let monitoring_flag_for_tee = self.monitoring_enabled.clone();

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
        let in_dev_captured = input_device_name.clone();
        let out_dev_captured = output_device_name.clone();
        let pos_captured = self.positions.clone();
        let h_captured = self.heights.clone();
        let w_captured = self.widths.clone();

        // --- INPUT STREAM (Audio Thread) ---
        // This closure runs on the high-priority audio thread.
        // MALLOC/FREE FORBIDDEN. LOCKING FORBIDDEN.
        //
        // # Format Dispatch
        // Virtual devices (e.g. VB-Audio CABLE) commonly expose I16 or I32 in WASAPI
        // shared mode, not F32. We convert to F32 at the boundary so the DSP chain
        // always works in float. The conversion is branchless per-sample arithmetic.

        let monitoring_enabled_flag_for_cb = monitoring_enabled_flag.clone();

        // Shared inner processing logic, called from each format arm below.
        let mut run_input_processing = {
            move |data_f32: &[f32], in_ch: usize| {
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
                            chain.add_module(module);
                            chain_changed = true;
                        }
                        InternalEngineCommand::RemoveModule(id) => {
                            if let Some(module) = chain.remove_module(&id) {
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
                        InternalEngineCommand::SetMonitoring(enabled) => {
                            monitoring_enabled_flag_for_cb.store(enabled, Ordering::Relaxed);
                            chain_changed = true;
                        }
                    }
                }

                if chain_changed {
                    let total_lat = chain.latency_samples();
                    mod_lat_atomic.store(total_lat, Ordering::Relaxed);
                    if let Ok(mut state_lock) = chain_state_clone.try_lock() {
                        let current_buf = metrics.buffer_size.load(Ordering::Relaxed);
                        let is_mon = monitoring_enabled_flag_for_cb.load(Ordering::Relaxed);
                        let layout = crate::core::persistence::LayoutConfig {
                            positions: pos_captured.lock().unwrap().clone(),
                            heights: h_captured.lock().unwrap().clone(),
                            widths: w_captured.lock().unwrap().clone(),
                        };
                        *state_lock = chain.get_state(
                            true,
                            current_buf,
                            is_mon,
                            in_dev_captured.clone(),
                            out_dev_captured.clone(),
                            layout,
                        );
                        state_lock.monitoring_enabled = is_mon;
                        metrics.state_version.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // Downmix to mono (take channel 0 only)
                for frame in data_f32.chunks(in_ch) {
                    input_accumulator.push(frame[0]);
                }

                // 2. Resample In
                if let Some(ref mut rs) = resampler_in {
                    while input_accumulator.len() >= rs.input_frames_next() {
                        let len = rs.input_frames_next();
                        resample_buf[0].clear();
                        resample_buf[0].extend(input_accumulator.drain(..len));
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

                    chain.process(&mut working_chunk);
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

                    // 5. Monitor tee: Always push the raw processed audio to the monitor buffer
                    // before any primary-path muting happens.
                    if monitoring_flag_for_tee.load(Ordering::Relaxed) {
                        mon_producer.push_slice(&output_accumulator);
                    } else {
                        for _ in 0..output_accumulator.len() {
                            let _ = mon_producer.try_push(0.0);
                        }
                    }

                    // Primary Output Buffer
                    producer.push_slice(&output_accumulator);

                    output_accumulator.clear();

                    rb_occ_in.store(producer.occupied_len(), Ordering::Relaxed);
                    let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
                    metrics.update_latency(elapsed);
                }
            }
        };

        // Each match arm owns its own clone of the metrics arc for hardware latency tracking,
        // since a single closure cannot be moved into multiple arms simultaneously.
        let metrics_in_f32 = self.metrics.clone();
        let metrics_in_i16 = self.metrics.clone();
        let metrics_in_i32 = self.metrics.clone();

        let input_stream = match in_fmt {
            cpal::SampleFormat::F32 => input_device.build_input_stream(
                &input_config,
                move |data: &[f32], info: &cpal::InputCallbackInfo| {
                    if let Some(diff) = info
                        .timestamp()
                        .callback
                        .duration_since(&info.timestamp().capture)
                    {
                        metrics_in_f32
                            .input_latency_ms
                            .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                    }
                    run_input_processing(data, in_channels);
                },
                |err| log::error!("Input stream error: {err}"),
                None,
            )?,
            cpal::SampleFormat::I16 => {
                // Pre-allocate once; the callback reuses (and resizes only on first call).
                let mut conv_buf: Vec<f32> = Vec::with_capacity(4096);
                input_device.build_input_stream(
                    &input_config,
                    move |data: &[i16], info: &cpal::InputCallbackInfo| {
                        if let Some(diff) = info
                            .timestamp()
                            .callback
                            .duration_since(&info.timestamp().capture)
                        {
                            metrics_in_i16
                                .input_latency_ms
                                .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                        }
                        conv_buf.clear();
                        conv_buf.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                        run_input_processing(&conv_buf, in_channels);
                    },
                    |err| log::error!("Input stream error: {err}"),
                    None,
                )?
            }
            cpal::SampleFormat::I32 => {
                let mut conv_buf: Vec<f32> = Vec::with_capacity(4096);
                input_device.build_input_stream(
                    &input_config,
                    move |data: &[i32], info: &cpal::InputCallbackInfo| {
                        if let Some(diff) = info
                            .timestamp()
                            .callback
                            .duration_since(&info.timestamp().capture)
                        {
                            metrics_in_i32
                                .input_latency_ms
                                .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                        }
                        conv_buf.clear();
                        conv_buf.extend(data.iter().map(|&s| s as f32 / i32::MAX as f32));
                        run_input_processing(&conv_buf, in_channels);
                    },
                    |err| log::error!("Input stream error: {err}"),
                    None,
                )?
            }
            fmt => {
                return Err(EngineError::DeviceError(format!(
                    "Input device '{}' uses unsupported sample format {:?}",
                    input_device.name().unwrap_or_default(),
                    fmt
                )));
            }
        };

        // --- OUTPUT STREAM ---
        // Dispatch on the output device's actual format so that virtual devices
        // receiving integer PCM (e.g. VB-Audio CABLE in I16 mode) get correctly
        // converted samples instead of raw f32 bit-patterns.
        let out_channels_out = output_config.channels as usize;

        let actual_out_name = output_device.name().unwrap_or_default();
        let is_virtual_out = actual_out_name.to_lowercase().contains("cable")
            || actual_out_name.to_lowercase().contains("virtual")
            || actual_out_name.to_lowercase().contains("blackhole")
            || actual_out_name.to_lowercase().contains("monitor")
            || actual_out_name.to_lowercase().contains("pipewire");

        let monitoring_enabled_for_out = self.monitoring_enabled.clone();

        let output_stream = match out_fmt {
            cpal::SampleFormat::F32 => output_device.build_output_stream(
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
                    let frames_needed = data.len() / out_channels_out;
                    metrics_out
                        .buffer_size
                        .store(frames_needed as u32, Ordering::Relaxed);

                    let is_unmuted =
                        monitoring_enabled_for_out.load(Ordering::Relaxed) || is_virtual_out;

                    for i in 0..frames_needed {
                        let sample = consumer.try_pop().unwrap_or(0.0);
                        for c in 0..out_channels_out {
                            data[i * out_channels_out + c] = if is_unmuted { sample } else { 0.0 };
                        }
                    }
                    rb_occ_out.store(consumer.occupied_len(), Ordering::Relaxed);
                },
                |err| log::error!("Output stream error: {err}"),
                None,
            )?,
            cpal::SampleFormat::I16 => {
                let monitoring_enabled_i16 = self.monitoring_enabled.clone();
                output_device.build_output_stream(
                    &output_config,
                    move |data: &mut [i16], info: &cpal::OutputCallbackInfo| {
                        if let Some(diff) = info
                            .timestamp()
                            .playback
                            .duration_since(&info.timestamp().callback)
                        {
                            metrics_out
                                .output_latency_ms
                                .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                        }
                        let frames_needed = data.len() / out_channels_out;
                        metrics_out
                            .buffer_size
                            .store(frames_needed as u32, Ordering::Relaxed);

                        let is_unmuted =
                            monitoring_enabled_i16.load(Ordering::Relaxed) || is_virtual_out;

                        for i in 0..frames_needed {
                            let sample = consumer.try_pop().unwrap_or(0.0);
                            let s16 = if is_unmuted {
                                (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
                            } else {
                                0
                            };
                            for c in 0..out_channels_out {
                                data[i * out_channels_out + c] = s16;
                            }
                        }
                        rb_occ_out.store(consumer.occupied_len(), Ordering::Relaxed);
                    },
                    |err| log::error!("Output stream error: {err}"),
                    None,
                )?
            }
            cpal::SampleFormat::I32 => {
                let monitoring_enabled_i32 = self.monitoring_enabled.clone();
                output_device.build_output_stream(
                    &output_config,
                    move |data: &mut [i32], info: &cpal::OutputCallbackInfo| {
                        if let Some(diff) = info
                            .timestamp()
                            .playback
                            .duration_since(&info.timestamp().callback)
                        {
                            metrics_out
                                .output_latency_ms
                                .store((diff.as_secs_f32() * 1000.0).to_bits(), Ordering::Relaxed);
                        }
                        let frames_needed = data.len() / out_channels_out;
                        metrics_out
                            .buffer_size
                            .store(frames_needed as u32, Ordering::Relaxed);

                        let is_unmuted =
                            monitoring_enabled_i32.load(Ordering::Relaxed) || is_virtual_out;

                        for i in 0..frames_needed {
                            let sample = consumer.try_pop().unwrap_or(0.0);
                            let s32 = if is_unmuted {
                                (sample.clamp(-1.0, 1.0) * i32::MAX as f32) as i32
                            } else {
                                0
                            };
                            for c in 0..out_channels_out {
                                data[i * out_channels_out + c] = s32;
                            }
                        }
                        rb_occ_out.store(consumer.occupied_len(), Ordering::Relaxed);
                    },
                    |err| log::error!("Output stream error: {err}"),
                    None,
                )?
            }

            fmt => {
                return Err(EngineError::DeviceError(format!(
                    "Output device '{}' uses unsupported sample format {:?}",
                    output_device.name().unwrap_or_default(),
                    fmt
                )));
            }
        };

        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(StreamWrapper(input_stream));
        self.output_stream = Some(StreamWrapper(output_stream));

        // --- AUTOMATIC MONITOR STARTUP ---
        // If monitoring is enabled, we determine if a secondary stream is actually needed:
        // 1. If output is a virtual cable, we MUST open a second stream to the System Default Output
        //    so the user can hear what's happening.
        // 2. If output is already a physical device (Speakers), we DO NOT open a second stream
        //    because the primary output stream already handles the audible audio.
        self.monitor_stream = if monitoring_enabled_flag.load(Ordering::Relaxed) {
            let is_virtual = actual_out.to_lowercase().contains("cable")
                || actual_out.to_lowercase().contains("virtual")
                || actual_out.to_lowercase().contains("blackhole")
                || actual_out.to_lowercase().contains("monitor")
                || actual_out.to_lowercase().contains("pipewire");

            if is_virtual {
                let mon_device_opt = host.default_output_device();

                if let Some(mon_device) = mon_device_opt {
                    let mon_name = mon_device.name().unwrap_or_else(|_| "Unknown".to_string());
                    log::info!("Opening Automatic Monitor Stream on: {mon_name}");

                    let mon_default = mon_device.default_output_config().unwrap_or_else(|_| {
                        host.default_output_device()
                            .unwrap()
                            .default_output_config()
                            .unwrap()
                    });

                    let mon_fmt = mon_default.sample_format();
                    let mon_cfg: cpal::StreamConfig = mon_default.into();
                    let mon_channels = mon_cfg.channels as usize;

                    let stream = match mon_fmt {
                        cpal::SampleFormat::F32 => mon_device
                            .build_output_stream(
                                &mon_cfg,
                                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / mon_channels;
                                    for i in 0..frames {
                                        let s = mon_consumer.try_pop().unwrap_or(0.0);
                                        for c in 0..mon_channels {
                                            data[i * mon_channels + c] = s;
                                        }
                                    }
                                },
                                |err| log::error!("Monitor stream error: {err}"),
                                None,
                            )
                            .ok(),
                        cpal::SampleFormat::I16 => mon_device
                            .build_output_stream(
                                &mon_cfg,
                                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / mon_channels;
                                    for i in 0..frames {
                                        let s = mon_consumer.try_pop().unwrap_or(0.0);
                                        let s16 = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                                        for c in 0..mon_channels {
                                            data[i * mon_channels + c] = s16;
                                        }
                                    }
                                },
                                |err| log::error!("Monitor stream error: {err}"),
                                None,
                            )
                            .ok(),
                        cpal::SampleFormat::I32 => mon_device
                            .build_output_stream(
                                &mon_cfg,
                                move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / mon_channels;
                                    for i in 0..frames {
                                        let s = mon_consumer.try_pop().unwrap_or(0.0);
                                        let s32 = (s.clamp(-1.0, 1.0) * i32::MAX as f32) as i32;
                                        for c in 0..mon_channels {
                                            data[i * mon_channels + c] = s32;
                                        }
                                    }
                                },
                                |err| log::error!("Monitor stream error: {err}"),
                                None,
                            )
                            .ok(),
                        _ => None,
                    };

                    if let Some(ref s) = stream {
                        if s.play().is_ok() {
                            log::info!("Automatic Monitor started successfully");
                        }
                    }
                    stream.map(StreamWrapper)
                } else {
                    None
                }
            } else {
                // Physical device selected as Output; primary stream handles everything.
                None
            }
        } else {
            None
        };
        if self.monitor_stream.is_some() {
            println!("!!! [ENGINE START] Monitor Active !!!");
        }

        log::info!(
            "Audio engine started successfully (Internal Rate: {}Hz, Monitor: {})",
            internal_sample_rate,
            if self.monitor_stream.is_some() {
                "on"
            } else {
                "off"
            }
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.is_running() {
            self.input_stream = None;
            self.output_stream = None;
            self.monitor_stream = None;
            if let Ok(mut tx_lock) = self.command_tx.lock() {
                *tx_lock = None;
            }

            if let Ok(mut state_lock) = self.chain_state.lock() {
                state_lock.is_running = false;
                self.metrics.state_version.fetch_add(1, Ordering::Relaxed);
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
        // Test the channel-drain GC contract without instantiating AudioEngine.
        // AudioEngine::new() calls cpal::default_host() which initializes
        // WASAPI COM on Windows. Repeated init/teardown across sequential
        // tests can corrupt COM state, causing a native ACCESS_VIOLATION.
        let (tx, rx): (
            Sender<Box<dyn crate::core::traits::AudioModule>>,
            Receiver<Box<dyn crate::core::traits::AudioModule>>,
        ) = unbounded();

        // Create a dummy module and send it to the garbage channel
        let m = ModuleFactory::create("Gain", 48000.0).unwrap();
        tx.send(m).unwrap();

        // Channel should not be empty
        assert!(!rx.is_empty());

        // Simulate the same drain loop used in AudioEngine::process_garbage
        while let Ok(module) = rx.try_recv() {
            assert_eq!(module.name(), "Gain");
            drop(module);
        }

        // Channel should be empty now
        assert!(rx.is_empty());
    }

    #[test]
    fn test_persistence_sync() {
        let engine = AudioEngine::new();

        // Setup some state
        *engine.input_device_name.lock().unwrap() = Some("Test Input".to_string());
        engine.monitoring_enabled.store(true, Ordering::SeqCst);

        // Add a module - this adds to offline_chain automatically
        engine.send_command(EngineCommand::AddModule {
            module_type: "Gain".to_string(),
        });

        // Get config
        let config = engine.get_persistence_config();
        assert_eq!(config.input_device, Some("Test Input".to_string()));
        assert_eq!(config.monitoring_enabled, true);
        assert_eq!(config.modules.len(), 1);
        assert_eq!(config.modules[0].config.type_name(), "Gain");

        // Create new engine and apply config
        let engine2 = AudioEngine::new();
        engine2.apply_persistence_config(config);

        assert_eq!(
            *engine2.input_device_name.lock().unwrap(),
            Some("Test Input".to_string())
        );
        assert_eq!(engine2.monitoring_enabled.load(Ordering::SeqCst), true);

        // Apply should have added module to offline_chain
        let chain2 = engine2.offline_chain.lock().unwrap();
        assert_eq!(chain2.modules().len(), 1);
    }
}
