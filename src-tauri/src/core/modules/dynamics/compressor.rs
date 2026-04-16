// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// COMPRESSOR MODULE
// Dynamics processing for controlling audio signal gain.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use uuid::Uuid;

/// A dynamic range compressor module.
pub struct CompressorModule {
    id: String,
    enabled: bool,
    threshold_db: ParameterSmoother,
    ratio: ParameterSmoother,
    attack_ms: f32,
    release_ms: f32,
    knee_db: ParameterSmoother,
    makeup_gain_db: ParameterSmoother,

    sample_rate: f32,
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl CompressorModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let mut m = Self {
            id,
            enabled: false,
            threshold_db: ParameterSmoother::new(-20.0, 10.0, sample_rate),
            ratio: ParameterSmoother::new(4.0, 10.0, sample_rate),
            attack_ms: 10.0,
            release_ms: 100.0,
            knee_db: ParameterSmoother::new(6.0, 10.0, sample_rate),
            makeup_gain_db: ParameterSmoother::new(0.0, 10.0, sample_rate),
            sample_rate,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        m.update_coefficients();
        m
    }

    fn update_coefficients(&mut self) {
        self.attack_coeff = (-1.0 / (self.attack_ms * self.sample_rate / 1000.0)).exp();
        self.release_coeff = (-1.0 / (self.release_ms * self.sample_rate / 1000.0)).exp();
    }

    fn linear_to_db(linear: f32) -> f32 {
        20.0 * linear.max(1e-6).log10()
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0f32.powf(db / 20.0)
    }
}

impl AudioModule for CompressorModule {
    fn name(&self) -> &str {
        "Compressor"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Dynamics
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Compressor {
            enabled,
            threshold_db,
            ratio,
            attack_ms,
            release_ms,
            knee_db,
            makeup_gain_db,
        } = config
        {
            self.enabled = *enabled;
            self.threshold_db.set_target(*threshold_db);
            self.ratio.set_target(ratio.max(1.0));
            self.attack_ms = attack_ms.clamp(0.1, 1000.0);
            self.release_ms = release_ms.clamp(1.0, 5000.0);
            self.knee_db.set_target(knee_db.max(0.0));
            self.makeup_gain_db.set_target(*makeup_gain_db);
            self.update_coefficients();
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Compressor {
            enabled: self.enabled,
            threshold_db: self.threshold_db.get_target(),
            ratio: self.ratio.get_target(),
            attack_ms: self.attack_ms,
            release_ms: self.release_ms,
            knee_db: self.knee_db.get_target(),
            makeup_gain_db: self.makeup_gain_db.get_target(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // Ballistics (Envelope Follower)
            if input_abs > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - input_abs) + input_abs;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_abs) + input_abs;
            }

            let input_db = Self::linear_to_db(self.envelope);

            // Get smoothed parameters
            let threshold = self.threshold_db.next();
            let ratio = self.ratio.next();
            let knee = self.knee_db.next();
            let makeup = self.makeup_gain_db.next();

            // Transfer Function with Soft Knee
            let mut output_db = input_db;
            if knee > 0.0
                && input_db > (threshold - knee / 2.0)
                && input_db < (threshold + knee / 2.0)
            {
                // Inside the knee region
                let x = input_db - threshold + knee / 2.0;
                output_db = input_db + ((1.0 / ratio - 1.0) * x * x) / (2.0 * knee);
            } else if input_db > threshold {
                // Above the knee
                output_db = threshold + (input_db - threshold) / ratio;
            }

            let gain_db = output_db - input_db + makeup;
            let gain = Self::db_to_linear(gain_db);

            *sample *= gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_initialization() {
        let comp = CompressorModule::new(48000.0);
        assert_eq!(comp.name(), "Compressor");
        assert!(!comp.enabled);
    }
}
