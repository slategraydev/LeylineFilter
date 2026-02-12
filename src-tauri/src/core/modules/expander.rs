use crate::core::traits::{AudioModule, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;

/// An audio expander/gate module that reduces the volume of signals below a threshold.
pub struct ExpanderModule {
    threshold: ParameterSmoother,
    ratio: ParameterSmoother,
    attack_ms: f32,
    release_ms: f32,
    sample_rate: f32,
    envelope: f32,
    gain: f32,
    enabled: bool,
    attack_coeff: f32,
    release_coeff: f32,
    smoothing_coeff: f32,
}

impl ExpanderModule {
    /// Creates a new `ExpanderModule` with default settings.
    pub fn new(sample_rate: f32) -> Self {
        let mut m = Self {
            threshold: ParameterSmoother::new(0.08, 10.0, sample_rate),
            ratio: ParameterSmoother::new(2.0, 10.0, sample_rate),
            attack_ms: 10.0,
            release_ms: 100.0,
            sample_rate,
            envelope: 0.0,
            gain: 1.0,
            enabled: true,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            smoothing_coeff: 0.0,
        };
        m.update_coefficients();
        m
    }

    fn update_coefficients(&mut self) {
        self.attack_coeff = (-1.0 / (self.attack_ms * self.sample_rate / 1000.0)).exp();
        self.release_coeff = (-1.0 / (self.release_ms * self.sample_rate / 1000.0)).exp();
        // 10ms smoothing for gain changes
        self.smoothing_coeff = (-1.0 / (10.0 * self.sample_rate / 1000.0)).exp();

        self.threshold.set_smoothing_ms(10.0, self.sample_rate);
        self.ratio.set_smoothing_ms(10.0, self.sample_rate);
    }
}

impl AudioModule for ExpanderModule {
    fn name(&self) -> &str {
        "Expander"
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Expander { enabled, threshold, ratio, attack_ms, release_ms } = config {
            self.enabled = *enabled;
            self.threshold.set_target(threshold.clamp(0.0, 1.0));
            self.ratio.set_target(ratio.max(1.0));
            self.attack_ms = attack_ms.clamp(0.1, 1000.0);
            self.release_ms = release_ms.clamp(1.0, 5000.0);
            self.update_coefficients();
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // Simple envelope follower
            if input_abs > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - input_abs) + input_abs;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_abs) + input_abs;
            }

            // Get current smoothed parameters
            let current_threshold = self.threshold.next();
            let current_ratio = self.ratio.next();

            // Calculate target gain based on the ratio and threshold
            let target_gain = if self.envelope < current_threshold {
                if self.envelope < 1e-6 {
                    0.0
                } else {
                    (self.envelope / current_threshold).powf(current_ratio - 1.0)
                }
            } else {
                1.0
            };

            // Smooth gain changes to prevent clicking
            self.gain = self.smoothing_coeff * (self.gain - target_gain) + target_gain;
            *sample *= self.gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_initialization() {
        let expander = ExpanderModule::new(48000.0);
        assert_eq!(expander.name(), "Expander");
        assert!(expander.enabled);
    }

    #[test]
    fn test_expander_gain_reduction() {
        let mut expander = ExpanderModule::new(48000.0);
        let config = ModuleConfig::Expander {
            enabled: true,
            threshold: 0.5,
            ratio: 10.0,
            attack_ms: 1.0,
            release_ms: 1.0,
        };
        expander.update_config(&config);

        // Signal way below threshold
        let mut samples = vec![0.1f32; 4800]; // 100ms
        expander.process(&mut samples);

        // The last sample should be significantly reduced
        assert!(samples[4799].abs() < 0.1);
    }

    #[test]
    fn test_expander_bypass() {
        let mut expander = ExpanderModule::new(48000.0);
        let config = ModuleConfig::Expander {
            enabled: false,
            threshold: 0.5,
            ratio: 10.0,
            attack_ms: 1.0,
            release_ms: 1.0,
        };
        expander.update_config(&config);

        let mut samples = vec![0.5f32; 100];
        let original = samples.clone();
        expander.process(&mut samples);

        assert_eq!(samples, original);
    }
}
