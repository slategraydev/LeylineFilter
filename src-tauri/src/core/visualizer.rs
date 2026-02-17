// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

use crate::core::audio::EngineMetrics;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

struct BinRange {
    start: usize,
    end: usize,
}

/// Pre-allocated state for audio visualization to ensure real-time safety.
///
/// # Visualizer Strategy
/// We reuse the `FftPlanner` and pre-allocated scratch buffers to avoid
/// repeated allocations during the render loop.
/// The `BinRange` array caches the mapping from FFT bins to the 12 display bars
/// (logarithmic scale) so we don't recalculate `powf` every frame.
pub struct VisualizerState {
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    scratch_buffer: Vec<Complex<f32>>,
    bin_ranges: [BinRange; 12],
    sample_rate: f32,
    chunk_size: usize,
}

impl VisualizerState {
    pub fn new(chunk_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(chunk_size);

        let mut state = Self {
            fft_buffer: vec![Complex::default(); chunk_size],
            scratch_buffer: vec![Complex::default(); fft.get_inplace_scratch_len()],
            fft,
            bin_ranges: [const { BinRange { start: 0, end: 0 } }; 12],
            sample_rate: 48000.0, // Default, will update on process
            chunk_size,
        };
        state.recalculate_bins(48000.0);
        state
    }

    fn recalculate_bins(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let num_bins = self.fft_buffer.len() / 2;
        if num_bins == 0 {
            return;
        }

        let f_min = 80.0f32;
        let f_max = 20000.0f32;
        let bin_sz = sample_rate / self.chunk_size as f32;

        for i in 0..12 {
            let freq_start = f_min * (f_max / f_min).powf(i as f32 / 12.0);
            let freq_end = f_min * (f_max / f_min).powf((i + 1) as f32 / 12.0);

            let start_bin = (freq_start / bin_sz).floor() as usize;
            let end_bin = (freq_end / bin_sz).ceil() as usize;

            self.bin_ranges[i] = BinRange {
                start: start_bin.min(num_bins),
                end: end_bin.max(start_bin + 1).min(num_bins),
            };
        }
    }

    pub fn process(&mut self, chunk: &[f32], sample_rate: f32, metrics: &EngineMetrics) {
        if (sample_rate - self.sample_rate).abs() > 1.0 {
            self.recalculate_bins(sample_rate);
        }

        for (i, &sample) in chunk.iter().enumerate() {
            if i < self.fft_buffer.len() {
                self.fft_buffer[i] = Complex {
                    re: sample,
                    im: 0.0,
                };
            }
        }

        self.fft
            .process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);

        let mut spectrum_bins = [0.0f32; 12];
        let mut tonality_bins = [0.0f32; 12];
        let num_bins = self.fft_buffer.len() / 2;

        if num_bins > 0 {
            for i in 0..12 {
                let range = &self.bin_ranges[i];
                let mut sum = 0.0f32;
                let mut max_bin_val = 0.0f32;
                let count = (range.end - range.start).max(1);

                for b in range.start..range.end {
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
        let mut waveform = [0.0f32; 64];
        let step = chunk.len() / 64;

        for i in 0..64 {
            let idx = i * step;
            if idx < chunk.len() {
                // Apply Hann window to contain the wave in the center
                // Window formula: 0.5 * (1 - cos(2 * PI * i / (N - 1)))
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / 63.0).cos());
                waveform[i] = chunk[idx] * window;
            }
        }

        for &sample in chunk {
            let abs = sample.abs();
            if abs > peak {
                peak = abs;
            }
        }

        metrics.update_visualizer_metrics(peak, &spectrum_bins, &tonality_bins, &waveform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_visualizer_metrics_update() {
        let metrics = EngineMetrics::new();
        let mut visualizer = VisualizerState::new(480);
        let chunk = vec![0.5; 480]; // Flat signal

        visualizer.process(&chunk, 48000.0, &metrics);

        // Check if level was updated
        let level = f32::from_bits(metrics.input_level.load(Ordering::Relaxed));
        assert!(level > 0.0);
        assert_eq!(level, 0.5);

        // Check if latency was NOT touched (should remain 0)
        let latency = f32::from_bits(metrics.latency_ms.load(Ordering::Relaxed));
        assert_eq!(latency, 0.0);
    }

    #[test]
    fn test_bin_range_validity() {
        let chunk_size = 2048;
        let mut visualizer = VisualizerState::new(chunk_size);
        let num_bins = chunk_size / 2;

        // Test at 48kHz
        visualizer.recalculate_bins(48000.0);
        for i in 0..12 {
            let range = &visualizer.bin_ranges[i];
            assert!(
                range.start <= range.end,
                "Bin range start > end at index {}",
                i
            );
            assert!(
                range.end <= num_bins,
                "Bin range end out of bounds at index {}",
                i
            );
        }

        // Test at 96kHz
        visualizer.recalculate_bins(96000.0);
        for i in 0..12 {
            let range = &visualizer.bin_ranges[i];
            assert!(
                range.start <= range.end,
                "Bin range start > end at index {}",
                i
            );
            assert!(
                range.end <= num_bins,
                "Bin range end out of bounds at index {}",
                i
            );
        }
    }
}
