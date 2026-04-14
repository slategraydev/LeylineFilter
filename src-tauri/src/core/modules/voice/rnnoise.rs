use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::resampling::AudioBlockProcessor;
use crate::utils::smoothing::ParameterSmoother;
use nnnoiseless::DenoiseState;
use uuid::Uuid;

/// An audio noise suppression module using the RNNoise algorithm.
pub struct RNNoiseModule {
    id: String,
    enabled: bool,
    sample_rate: f32,
    denoiser: Box<DenoiseState<'static>>,
    processor: AudioBlockProcessor,
    /// A smoother to cross-fade between wet and dry signal for de-clicking bypass.
    bypass_smoother: ParameterSmoother,
}

impl RNNoiseModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        Self {
            id,
            enabled: false,
            sample_rate,
            denoiser: DenoiseState::new(),
            processor: AudioBlockProcessor::new(sample_rate, 48000.0, 480),
            // 20ms cross-fade for smooth transitions
            bypass_smoother: ParameterSmoother::new(0.0, 20.0, sample_rate),
        }
    }
}

impl AudioModule for RNNoiseModule {
    fn name(&self) -> &str {
        "RNNoise"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Voice
    }

    fn requirements(&self) -> (Option<f32>, Option<usize>) {
        // RNNoise strictly requires 48kHz and 480 sample blocks (10ms)
        (Some(48000.0), Some(480))
    }

    fn latency_samples(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        480
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.processor.prepare(sample_rate);
        // RNNoise always processes at 48kHz internally/via processor, so the smoother
        // must be timed for 48kHz regardless of the external sample rate.
        self.bypass_smoother.set_smoothing_ms(20.0, 48000.0);
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::RNNoise { enabled } = config {
            if self.enabled != *enabled {
                log::info!("RNNoise module toggled to: {enabled}");
                self.enabled = *enabled;
                self.bypass_smoother
                    .set_target(if self.enabled { 1.0 } else { 0.0 });
            }
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::RNNoise {
            enabled: self.enabled,
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        let current_mix = self.bypass_smoother.current();

        // If we are fully disabled and the ramp has finished, just skip
        if !self.enabled && current_mix < 1e-4 {
            return;
        }

        let denoiser = &mut self.denoiser;
        let bypass = &mut self.bypass_smoother;

        // Optimization: If the engine is already providing 48kHz and 480 sample blocks,
        // we can skip the internal processor and its redundant resampling/buffering.
        if (self.sample_rate - 48000.0).abs() < 1.0 && samples.len() == 480 {
            // Scale input from [-1.0, 1.0] to [-32768.0, 32767.0]
            let mut scaled_input = [0.0f32; 480];
            for (i, &s) in samples.iter().enumerate() {
                scaled_input[i] = s * 32768.0;
            }

            let mut output = [0.0f32; 480];
            denoiser.process_frame(&mut output, &scaled_input);

            // Scale output back and apply bypass mix
            for i in 0..480 {
                let wet = output[i] / 32768.0;
                let dry = samples[i];
                let mix = bypass.next();
                samples[i] = wet * mix + dry * (1.0 - mix);
            }
        } else {
            // Fallback for non-optimal environments (unlikely with our new engine logic)
            self.processor.process(samples, |input, output| {
                let mut scaled_input = [0.0f32; 480];
                for (i, &s) in input.iter().enumerate() {
                    scaled_input[i] = s * 32768.0;
                }

                denoiser.process_frame(output, &scaled_input);

                for i in 0..output.len() {
                    let wet = output[i] / 32768.0;
                    let dry = input[i];
                    let mix = bypass.next();
                    output[i] = wet * mix + dry * (1.0 - mix);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnnoise_requirements() {
        let module = RNNoiseModule::new(48000.0);
        let (rate, size) = module.requirements();
        assert_eq!(rate, Some(48000.0));
        assert_eq!(size, Some(480));
    }

    #[test]
    fn test_rnnoise_latency() {
        let mut module = RNNoiseModule::new(48000.0);
        assert_eq!(module.latency_samples(), 0); // Disabled by default

        module.update_config(&ModuleConfig::RNNoise { enabled: true });
        assert_eq!(module.latency_samples(), 480);
    }

    #[test]
    fn test_rnnoise_process_optimal() {
        let mut module = RNNoiseModule::new(48000.0);
        module.update_config(&ModuleConfig::RNNoise { enabled: true });

        // Fast-path test (48kHz, 480 samples)
        let mut samples = vec![0.1f32; 480];
        module.process(&mut samples);

        // Should not be silence immediately due to bypass ramp
        assert!(samples[0].abs() > 0.0);
    }
}
