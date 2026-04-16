// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// GAIN MODULE
// Simple gain control module.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use uuid::Uuid;

/// A simple gain module.
pub struct GainModule {
    id: String,
    gain: ParameterSmoother,
    enabled: bool,
    sample_rate: f32,
}

impl GainModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        Self {
            id,
            gain: ParameterSmoother::new(1.0, 10.0, sample_rate),
            enabled: false,
            sample_rate,
        }
    }
}

impl AudioModule for GainModule {
    fn name(&self) -> &str {
        "Gain"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Utility
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.gain.set_smoothing_ms(10.0, sample_rate);
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Gain { enabled, gain_db } = config {
            self.enabled = *enabled;
            // Convert dB to linear gain: 10^(dB/20)
            let linear_gain = 10.0f32.powf(*gain_db / 20.0);
            self.gain.set_target(linear_gain);
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Gain {
            enabled: self.enabled,
            gain_db: 20.0 * self.gain.get_target().log10(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            *sample *= self.gain.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_initialization() {
        let gain = GainModule::new(48000.0);
        assert_eq!(gain.name(), "Gain");
        assert!(!gain.enabled);
    }

    #[test]
    fn test_gain_processing() {
        let mut gain = GainModule::new(48000.0);
        gain.update_config(&ModuleConfig::Gain {
            enabled: true,
            gain_db: 6.0,
        }); // ~2x gain

        let mut samples = vec![1.0f32; 5000]; // ~100ms
        gain.process(&mut samples);

        // After 100ms (10 time constants), it should be very close to 2.0
        assert!(samples[4999] > 1.9);
        assert!((samples[4999] - 2.0).abs() < 0.01);
    }
}
