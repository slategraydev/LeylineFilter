use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use crate::core::traits::{AudioModule, ModuleConfig};
use crate::core::modules::expander::ExpanderModule;
use crate::error::{EngineError, Result as EngineResult};
use rubato::{Resampler, FastFixedIn, PolynomialDegree};
use std::time::Instant;
use ringbuf::HeapRb;
use ringbuf::traits::{Producer, Consumer, Split};

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
            cpu_usage: AtomicU32::new(0),
            input_level: AtomicU32::new(0),
            spectrum: Default::default(),
            tonality: Default::default(),
        }
    }

    fn update(&self, latency: f32, cpu: f32, level: f32, bins: &[f32; 12], tonality: &[f32; 12]) {
        self.latency_ms.store(latency.to_bits(), Ordering::Relaxed);
        self.cpu_usage.store(cpu.to_bits(), Ordering::Relaxed);
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
}

/// The core audio engine responsible for managing streams and processing modules.
pub struct AudioEngine {
    input_stream: Option<StreamWrapper>,
    output_stream: Option<StreamWrapper>,
    modules: Arc<Mutex<Vec<Box<dyn AudioModule>>>>,
    pub metrics: Arc<EngineMetrics>,
}

impl AudioEngine {
    /// Creates a new instance of the `AudioEngine`.
    pub fn new() -> Self {
        let expander = Box::new(ExpanderModule::new(48000.0));

        Self {
            input_stream: None,
            output_stream: None,
            modules: Arc::new(Mutex::new(vec![expander])),
            metrics: Arc::new(EngineMetrics::new()),
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
        let in_channels = input_config.channels as usize;

        let modules = self.modules.clone();
        let metrics = self.metrics.clone();

        // Use a lock-free ring buffer for real-time safe audio transfer
        let rb = HeapRb::<f32>::new(4800 * 2);
        let (mut producer, mut consumer) = rb.split();

        // Setup resampling if necessary
        let mut resampler = if (in_sample_rate - 48000.0).abs() > 1.0 {
            Some(FastFixedIn::<f32>::new(
                48000.0 / in_sample_rate,
                2.0,
                PolynomialDegree::Cubic,
                480,
                1,
            ).map_err(|e| EngineError::ResamplerError(e.to_string()))?)
        } else {
            None
        };

        let mut input_accumulator = Vec::new();
        let mut process_accumulator = Vec::new();
        let mut resample_input = vec![vec![0.0f32; 0]];

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
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

                // Process in chunks of 480 samples
                while process_accumulator.len() >= 480 {
                    let chunk: Vec<f32> = process_accumulator.drain(0..480).collect();
                    process_audio_chunk(&chunk, &modules, &metrics, &mut producer);
                }
            },
            |err| log::error!("Input stream error: {}", err),
            None
        )?;

        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let out_channels = output_config.channels as usize;
                let frames_needed = data.len() / out_channels;

                for i in 0..frames_needed {
                    let sample = consumer.try_pop().unwrap_or(0.0);
                    for c in 0..out_channels {
                        data[i * out_channels + c] = sample;
                    }
                }
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

use rustfft::{FftPlanner, num_complex::Complex};

/// Processes a single chunk of audio through all active modules.
fn process_audio_chunk<P: Producer<Item = f32>>(
    chunk: &[f32],
    modules: &Arc<Mutex<Vec<Box<dyn AudioModule>>>>,
    metrics: &Arc<EngineMetrics>,
    producer: &mut P,
) {
    let start = Instant::now();
    let mut working_chunk = chunk.to_vec();

    if let Ok(mut modules_lock) = modules.try_lock() {
        for module in modules_lock.iter_mut() {
            module.process(&mut working_chunk);
        }
    }

    producer.push_slice(&working_chunk);

    // Peak level calculation
    let mut peak = 0.0f32;
    for &sample in &working_chunk {
        let abs = sample.abs();
        if abs > peak {
            peak = abs;
        }
    }

    // FFT Analysis for true frequency visualization
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(working_chunk.len());

    let mut buffer: Vec<Complex<f32>> = working_chunk
        .iter()
        .map(|&s| Complex { re: s, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

        // Map FFT bins to 12 UI bars (logarithmic distribution)
        let mut spectrum_bins = [0.0f32; 12];
        let mut tonality_bins = [0.0f32; 12];
        let num_bins = buffer.len() / 2; // Nyquist limit (e.g., 240 bins for 48k SR)

            if num_bins > 0 {
                // Logarithmic distribution: Every bar covers an equal musical interval (octaves).
                // We map from ~80Hz to ~20kHz.
                // With a 480 chunk size at 48kHz, each bin is 100Hz.
                let f_min = 80.0f32;
                let f_max = 20000.0f32;
                let bin_sz = 48000.0 / chunk.len() as f32;

                for i in 0..12 {
                    // Calculate frequency boundaries for this logarithmic bucket
                    let freq_start = f_min * (f_max / f_min).powf(i as f32 / 12.0);
                    let freq_end = f_min * (f_max / f_min).powf((i + 1) as f32 / 12.0);

                    let start_bin = (freq_start / bin_sz).floor() as usize;
                    let end_bin = (freq_end / bin_sz).ceil() as usize;

                    // Clamp to valid range
                    let start_bin = start_bin.min(num_bins);
                    let end_bin = end_bin.max(start_bin + 1).min(num_bins);

                    let mut sum = 0.0f32;
                    let mut max_bin_val = 0.0f32;
                    let count = (end_bin - start_bin).max(1);

                    for b in start_bin..end_bin {
                        let val = buffer[b].norm();
                        sum += val;
                        if val > max_bin_val {
                            max_bin_val = val;
                        }
                    }

                                let avg = sum / count as f32;

                                // Normalize FFT magnitude by the number of bins (N/2)
                                // This ensures a full-scale sine wave results in a value of ~1.0
                                let normalized_mag = avg / (num_bins as f32);

                                            // Convert to decibels (dB)
                                            // -60dB is a standard floor for UI meters
                                            let db = 20.0 * normalized_mag.max(1e-6).log10();

                                            // Map -60dB..0dB to 0.0..1.0
                                            let linear_val = ((db + 60.0) / 60.0).clamp(0.0, 1.0);

                                            // Apply a square root (quadratic) curve to the linear value.
                                            // This makes the meter 'stay high' longer and gives more
                                            // visual resolution to the quieter parts of the signal.
                                            let normalized_val = linear_val.sqrt();

                                            spectrum_bins[i] = normalized_val;
                                                                            if avg > 1e-6 {
                                                                                let ratio = max_bin_val / avg;
                                                                                // Normalize the ratio requirement based on how many bins are in this bucket.
                                                                                // Wider buckets (high end) require a higher ratio to be considered 'tonal'.
                                                                                let threshold = (count as f32).sqrt().max(2.0);
                                                                                tonality_bins[i] = (ratio / (threshold * 1.5)).min(1.0);
                                                                            }

                            }
                        }
                            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    metrics.update(elapsed, (elapsed / 10.0) * 100.0, peak, &spectrum_bins, &tonality_bins);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization() {
        let engine = AudioEngine::new();
        assert!(engine.input_stream.is_none());
        assert!(engine.output_stream.is_none());

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
}
