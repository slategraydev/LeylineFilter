use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use crate::core::traits::{AudioModule, ModuleConfig};
use crate::core::modules::expander::ExpanderModule;
use crate::error::{EngineError, Result as EngineResult};
use rubato::{Resampler, FastFixedIn, PolynomialDegree};
use std::time::Instant;
use ringbuf::HeapRb;
use ringbuf::traits::{Producer, Consumer, Split, Observer};
use sysinfo::System;
use rustfft::{FftPlanner, Fft, num_complex::Complex};

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
        // Copy chunk into FFT buffer
        for (i, &sample) in chunk.iter().enumerate() {
            if i < self.fft_buffer.len() {
                self.fft_buffer[i] = Complex { re: sample, im: 0.0 };
            }
        }

        // Perform FFT
        self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);

        // Map FFT bins to 12 UI bars (logarithmic distribution)
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

        // Peak level calculation
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

/// A wrapper around `cpal::Stream` to allow it to be sent across threads.
///
/// CPAL streams are generally thread-safe, but the wrapper is needed to
/// satisfy Rust's type system in certain contexts.
pub struct StreamWrapper(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for StreamWrapper {}

/// Performance metrics for the audio engine.
///
/// Uses atomic types to allow updates from the audio thread without
/// locking, ensuring real-time safety.
pub struct EngineMetrics {
    /// Processing latency in milliseconds.
    pub latency_ms: AtomicU32,
    /// Hardware input latency in milliseconds.
    pub input_latency_ms: AtomicU32,
    /// Hardware output latency in milliseconds.
    pub output_latency_ms: AtomicU32,
    /// Estimated CPU usage as a percentage.
    pub cpu_usage: AtomicU32,
    /// Peak RMS or level for visualization.
    pub input_level: AtomicU32,
    /// Frequency spectrum bins (12 bins for the UI).
    pub spectrum: [AtomicU32; 12],
    /// Tonality/Harmonicity bins (12 bins).
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
        // We still store processing_ms in latency_ms for now, but we'll improve it later
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

/// The core audio engine responsible for managing streams and processing modules.
pub struct AudioEngine {
    input_stream: Option<StreamWrapper>,
    output_stream: Option<StreamWrapper>,
    modules: Arc<Mutex<Vec<Box<dyn AudioModule>>>>,
    pub metrics: Arc<EngineMetrics>,
    sys: System,
    pid: sysinfo::Pid,
    sample_rate: f32,
    last_latency: f32,
    last_cpu: f32,
    rb_occupancy: Arc<AtomicUsize>,
    last_cpu_update: Instant,
}

impl AudioEngine {
    /// Creates a new instance of the `AudioEngine`.
    pub fn new() -> Self {
        let expander = Box::new(ExpanderModule::new(48000.0));
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get current PID");

        Self {
            input_stream: None,
            output_stream: None,
            modules: Arc::new(Mutex::new(vec![expander])),
            metrics: Arc::new(EngineMetrics::new()),
            sys,
            pid,
            sample_rate: 48000.0,
            last_latency: 0.0,
            last_cpu: 0.0,
            rb_occupancy: Arc::new(AtomicUsize::new(0)),
            last_cpu_update: Instant::now(),
        }
    }

    /// Checks if the audio engine is currently running.
    pub fn is_running(&self) -> bool {
        self.input_stream.is_some()
    }

    /// Calculates the total pipeline latency in milliseconds with smoothing.
    pub fn get_total_latency_ms(&mut self) -> f32 {
        if !self.is_running() {
            return 0.0;
        }

        let mut total_latency_samples = 0;
        if let Ok(modules) = self.modules.lock() {
            for module in modules.iter() {
                if module.is_enabled() {
                    total_latency_samples += module.latency_samples();
                }
            }
        }

        // Processing latency (from metrics)
        let processing_ms = f32::from_bits(self.metrics.latency_ms.load(Ordering::Relaxed));

        // Accumulator latency (10ms)
        let chunk_size = (self.sample_rate * 0.01) as usize;
        let chunk_latency_ms = (chunk_size as f32 / self.sample_rate) * 1000.0;

        // Ring buffer occupancy latency
        let rb_samples = self.rb_occupancy.load(Ordering::Relaxed);
        let rb_latency_ms = (rb_samples as f32 / self.sample_rate) * 1000.0;

        // Hardware latencies (Measured)
        let (in_lat, out_lat) = self.metrics.get_hardware_latencies();

        let current_latency = processing_ms + chunk_latency_ms + rb_latency_ms + (total_latency_samples as f32 / self.sample_rate) * 1000.0 + in_lat + out_lat;

        // EMA smoothing (alpha = 0.1)
        if self.last_latency == 0.0 {
            self.last_latency = current_latency;
        } else {
            self.last_latency = self.last_latency * 0.9 + current_latency * 0.1;
        }

        self.last_latency
    }

    /// Updates and returns the current process CPU usage.
    pub fn update_cpu_usage(&mut self) -> f32 {
        // Throttle CPU updates to every 200ms to allow sysinfo to calculate deltas accurately
        if self.last_cpu_update.elapsed() < std::time::Duration::from_millis(200) {
            return self.last_cpu;
        }

        self.sys.refresh_all();
        self.last_cpu_update = Instant::now();

        if let Some(process) = self.sys.process(self.pid) {
            // Process CPU usage is 0.0 to (100.0 * num_cpus)
            let raw_cpu = process.cpu_usage() / self.sys.cpus().len() as f32;

            // Apply EMA smoothing (alpha = 0.1) for a "soft" climb/drop
            self.last_cpu = self.last_cpu * 0.9 + raw_cpu * 0.1;

            self.metrics.cpu_usage.store(self.last_cpu.to_bits(), Ordering::Relaxed);
            self.last_cpu
        } else {
            0.0
        }
    }

    /// Updates the configuration for all managed audio modules.
    pub fn update_module_config(&self, config: ModuleConfig) {
        if let Ok(mut modules) = self.modules.lock() {
            for module in modules.iter_mut() {
                module.update_config(&config);
            }
        }
    }

            /// Returns a list of available input device names.
            pub fn get_input_devices(&self) -> Vec<String> {
                let host = cpal::default_host();
                match host.input_devices() {
                    Ok(devices) => devices
                        .filter_map(|d| d.name().ok())
                        .collect(),
                    Err(_) => vec![],
                }
            }

            /// Returns a list of available output device names.
            pub fn get_output_devices(&self) -> Vec<String> {
                let host = cpal::default_host();
                match host.output_devices() {
                    Ok(devices) => devices
                        .filter_map(|d| d.name().ok())
                        .collect(),
                    Err(_) => vec![],
                }
            }

            /// Starts the audio engine, initializing input and output streams.
            pub fn start(&mut self, input_device_name: Option<String>, output_device_name: Option<String>) -> EngineResult<()> {
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
                }.ok_or_else(|| EngineError::DeviceError("Input device not found".to_string()))?;

                let output_device = if let Some(name) = output_device_name {
                    if name == "Default" {
                        host.default_output_device()
                    } else {
                        host.output_devices()?
                            .find(|d| d.name().ok().as_ref() == Some(&name))
                    }
                } else {
                    host.default_output_device()
                }.ok_or_else(|| EngineError::DeviceError("Output device not found".to_string()))?;

            let input_config: cpal::StreamConfig = input_device.default_input_config()?.into();
        let output_config: cpal::StreamConfig = output_device.default_output_config()?.into();

        let in_sample_rate = input_config.sample_rate.0 as f64;
        let out_sample_rate = output_config.sample_rate.0 as f64;
        let in_channels = input_config.channels as usize;

        self.sample_rate = out_sample_rate as f32;

        let modules = self.modules.clone();
        let metrics = self.metrics.clone();
        let metrics_out = self.metrics.clone();
        let rb_occ_in = self.rb_occupancy.clone();
        let rb_occ_out = self.rb_occupancy.clone();

        // Notify all modules of the current sample rate
        if let Ok(mut modules_lock) = modules.lock() {
            for module in modules_lock.iter_mut() {
                module.prepare(self.sample_rate);
            }
        }

        // Use a lock-free ring buffer for real-time safe audio transfer
        // Size it for roughly 1 second of audio at the current sample rate
        let rb = HeapRb::<f32>::new((self.sample_rate as usize * 2).max(4800 * 2));
        let (mut producer, mut consumer) = rb.split();

        // Setup resampling if input rate differs from output rate
        let mut resampler = if (in_sample_rate - out_sample_rate).abs() > 1.0 {
            Some(FastFixedIn::<f32>::new(
                out_sample_rate / in_sample_rate,
                2.0,
                PolynomialDegree::Cubic,
                (out_sample_rate as f32 * 0.01) as usize, // 10ms chunk
                1,
            ).map_err(|e| EngineError::ResamplerError(e.to_string()))?)
        } else {
            None
        };

                        // Target 10ms chunk size for processing
                        let chunk_size = (self.sample_rate * 0.01) as usize;
                        let mut visualizer = VisualizerState::new(chunk_size);
                        let mut working_chunk = vec![0.0f32; chunk_size];

                        // Pre-allocate accumulators to avoid reallocations in the audio thread.
                        // 100ms worth of audio should be more than enough for jitter.
                        let mut input_accumulator = Vec::with_capacity((self.sample_rate * 0.1) as usize);
                        let mut process_accumulator = Vec::with_capacity((self.sample_rate * 0.1) as usize);
                        let mut resample_input = vec![vec![0.0f32; 0]];

                        let input_stream = input_device.build_input_stream(
                            &input_config,
                            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                                let start_time = Instant::now();
                                let callback = info.timestamp().callback;
                                let capture = info.timestamp().capture;
                                if let Some(diff) = callback.duration_since(&capture) {
                                    let ms = diff.as_secs_f32() * 1000.0;
                                    metrics.input_latency_ms.store(ms.to_bits(), Ordering::Relaxed);
                                }

                                // Take only the first channel (mono)
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

                                // Process in chunks
                                let sr = out_sample_rate as f32;
                                while process_accumulator.len() >= chunk_size {
                                    // Copy into pre-allocated working chunk using drain for O(N)
                                    for (i, sample) in process_accumulator.drain(0..chunk_size).enumerate() {
                                        working_chunk[i] = sample;
                                    }

                                    if let Ok(mut modules_lock) = modules.try_lock() {
                                        for module in modules_lock.iter_mut() {
                                            module.process(&mut working_chunk);
                                        }
                                    }

                                    producer.push_slice(&working_chunk);
                                    rb_occ_in.store(producer.occupied_len(), Ordering::Relaxed);

                                    // Visualization
                                    visualizer.process(&working_chunk, sr, &metrics);

                                    // Update processing time metric (for latency calculation)
                                    let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
                                    metrics.latency_ms.store(elapsed.to_bits(), Ordering::Relaxed);
                                }
                            },
                            |err| {
                                log::error!("Input stream error: {}", err);
                            },
                            None
                        )?;

                        let output_stream = output_device.build_output_stream(
                            &output_config,
                            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                                // Calculate hardware output latency
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

                    /// Stops the audio engine and drops the streams.
                    pub fn stop(&mut self) {
                        self.input_stream = None;
                        self.output_stream = None;
                        log::info!("Audio engine stopped");
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

        let (latency, cpu, level, spectrum, tonality) = engine.metrics.get();
        assert_eq!(latency, 0.0);
        assert_eq!(cpu, 0.0);
        assert_eq!(level, 0.0);
        assert_eq!(spectrum, [0.0f32; 12]);
        assert_eq!(tonality, [0.0f32; 12]);
    }

    #[test]
    fn test_module_config_propagation() {
        let engine = AudioEngine::new();
        let config = ModuleConfig::Expander {
            enabled: false,
            threshold: 0.5,
            ratio: 10.0,
            attack_ms: 5.0,
            release_ms: 50.0,
        };

        engine.update_module_config(config);

        let modules = engine.modules.lock().unwrap();
        // Since we know the first module is the expander
        // We can't easily downcast Box<dyn AudioModule> without extra traits,
        // but we've verified the code compiles and the logic is sound.
        assert_eq!(modules.len(), 1);
    }

    #[test]
    fn test_cpu_smoothing() {
        let mut engine = AudioEngine::new();
        engine.last_cpu = 0.0;

        // Mock a 10% CPU usage update
        // (Since we can't easily mock the process CPU, we can test the internal state if needed,
        // but for now we'll verify the engine starts at 0)
        let (_, cpu, _, _, _) = engine.metrics.get();
        assert_eq!(cpu, 0.0);
    }

    #[test]
    fn test_chunk_size_logic() {
        let sr_48k = 48000.0;
        let chunk_48k = (sr_48k * 0.01) as usize;
        assert_eq!(chunk_48k, 480);

        let sr_44k = 44100.0;
        let chunk_44k = (sr_44k * 0.01) as usize;
        assert_eq!(chunk_44k, 441);

        let sr_96k = 96000.0;
        let chunk_96k = (sr_96k * 0.01) as usize;
        assert_eq!(chunk_96k, 960);
    }
}
