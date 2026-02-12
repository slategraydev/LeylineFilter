use crate::core::traits::{AudioModule, ModuleConfig};
use crate::utils::resampling::AudioBlockProcessor;
use nnnoiseless::DenoiseState;

/// An audio noise suppression module using the RNNoise algorithm.
pub struct RNNoiseModule {
    enabled: bool,
    denoiser: Box<DenoiseState<'static>>,
    processor: AudioBlockProcessor,
}

impl RNNoiseModule {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            enabled: false,
            denoiser: DenoiseState::new(),
            // RNNoise strictly requires 48kHz and 480 sample blocks (10ms)
            processor: AudioBlockProcessor::new(sample_rate, 48000.0, 480),
        }
    }
}

impl AudioModule for RNNoiseModule {
    fn name(&self) -> &str {
        "RNNoise"
    }

    fn latency_samples(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        // RNNoise has a 10ms windowing latency
        480
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.processor.prepare(sample_rate);
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::RNNoise { enabled } = config {
            if self.enabled != *enabled {
                log::info!("RNNoise module toggled to: {}", enabled);
                self.enabled = *enabled;
            }
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let denoiser = &mut self.denoiser;
        self.processor.process(samples, |input, output| {
            // Scale input from [-1.0, 1.0] to [-32768.0, 32767.0]
            // This is critical because RNNoise is trained on 16-bit integer PCM scale.
            let mut scaled_input = [0.0f32; 480];
            for (i, &s) in input.iter().enumerate() {
                scaled_input[i] = s * 32768.0;
            }

            denoiser.process_frame(output, &scaled_input);

            // Scale output back to [-1.0, 1.0]
            for s in output.iter_mut() {
                *s /= 32768.0;
            }
        });
    }
}
