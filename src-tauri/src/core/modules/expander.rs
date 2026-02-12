use crate::core::traits::{AudioModule, ModuleConfig};

/// An audio expander/gate module that reduces the volume of signals below a threshold.
///
/// This implementation uses a simple envelope follower with attack and release
/// characteristics to smooth gain changes.
pub struct ExpanderModule {
    threshold: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    sample_rate: f32,
    envelope: f32,
    gain: f32,
    enabled: bool,
}

impl ExpanderModule {
    /// Creates a new `ExpanderModule` with default settings.
    ///
    /// # Arguments
    /// * `sample_rate` - The sample rate at which the audio will be processed.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 0.08,
            ratio: 2.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            sample_rate,
            envelope: 0.0,
            gain: 1.0,
            enabled: true,
        }
    }
}

impl AudioModule for ExpanderModule {
    fn name(&self) -> &str {
        "Expander"
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Expander { enabled, threshold, ratio, attack_ms, release_ms } = config {
            self.enabled = *enabled;
            // Ensure threshold is between 0 and 1
            self.threshold = threshold.clamp(0.0, 1.0);
            // Ratio must be at least 1.0
            self.ratio = ratio.max(1.0);

            self.attack_ms = attack_ms.clamp(0.1, 1000.0);
            self.release_ms = release_ms.clamp(1.0, 5000.0);

            log::debug!(
                "Expander updated: enabled={}, threshold={}, ratio={}, attack={}ms, release={}ms",
                self.enabled, self.threshold, self.ratio, self.attack_ms, self.release_ms
            );
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let attack_coeff = (-1.0 / (self.attack_ms * self.sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * self.sample_rate / 1000.0)).exp();

        // 10ms smoothing for gain changes
        let smoothing_coeff = (-1.0 / (10.0 * self.sample_rate / 1000.0)).exp();

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // Simple envelope follower
            if input_abs > self.envelope {
                self.envelope = attack_coeff * (self.envelope - input_abs) + input_abs;
            } else {
                self.envelope = release_coeff * (self.envelope - input_abs) + input_abs;
            }

            // Calculate target gain based on the ratio and threshold
            let target_gain = if self.envelope < self.threshold {
                if self.envelope < 1e-6 {
                    0.0
                } else {
                    (self.envelope / self.threshold).powf(self.ratio - 1.0)
                }
            } else {
                1.0
            };

            // Smooth gain changes to prevent clicking, sample-rate dependent
            self.gain = smoothing_coeff * (self.gain - target_gain) + target_gain;
            *sample *= self.gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_initialization() {
        let expander = ExpanderModule::new(44100.0);
        assert_eq!(expander.name(), "Expander");
        assert!(expander.enabled);
    }

    #[test]
    fn test_expander_gain_reduction() {
        let mut expander = ExpanderModule::new(44100.0);
        expander.threshold = 0.5;
        expander.ratio = 10.0;

        // Signal way below threshold
        let mut samples = vec![0.1f32; 1000];
        expander.process(&mut samples);

        // The last sample should be significantly reduced
        assert!(samples[999].abs() < 0.1);
    }

    #[test]
    fn test_expander_bypass() {
        let mut expander = ExpanderModule::new(44100.0);
        expander.enabled = false;

        let mut samples = vec![0.5f32; 100];
        let original = samples.clone();
        expander.process(&mut samples);

        assert_eq!(samples, original);
    }
}
