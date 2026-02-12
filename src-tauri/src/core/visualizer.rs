use crate::core::audio::EngineMetrics;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

/// Pre-allocated state for audio visualization to ensure real-time safety.
pub struct VisualizerState {
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    scratch_buffer: Vec<Complex<f32>>,
}

impl VisualizerState {
    pub fn new(chunk_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(chunk_size);
        Self {
            fft_buffer: vec![Complex::default(); chunk_size],
            scratch_buffer: vec![Complex::default(); fft.get_inplace_scratch_len()],
            fft,
        }
    }

    pub fn process(&mut self, chunk: &[f32], sample_rate: f32, metrics: &EngineMetrics) {
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

        metrics.update_visualizer_metrics(peak, &spectrum_bins, &tonality_bins);
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
}
