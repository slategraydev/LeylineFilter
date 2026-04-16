// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DEESSER MODULE
// Frequency-selective compression for reducing sibilance.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use std::f32::consts::PI;
use uuid::Uuid;

/// A professional De-Esser module using sidechained high-pass compression.
pub struct DeesserModule {
    id: String,
    enabled: bool,
    threshold_db: ParameterSmoother,
    ratio: ParameterSmoother,
    attack_ms: f32,
    release_ms: f32,
    frequency: ParameterSmoother,

    sample_rate: f32,
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,

    // Sidechain HPF (One-Pole)
    hpf_x1: f32,
    hpf_y1: f32,
}

impl DeesserModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let mut m = Self {
            id,
            enabled: false,
            threshold_db: ParameterSmoother::new(-30.0, 10.0, sample_rate),
            ratio: ParameterSmoother::new(4.0, 10.0, sample_rate),
            attack_ms: 1.0,   // Fast attack for sibilance
            release_ms: 50.0, // Natural release
            frequency: ParameterSmoother::new(6000.0, 10.0, sample_rate),
            sample_rate,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            hpf_x1: 0.0,
            hpf_y1: 0.0,
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

    /// Simple One-Pole HPF state calculation
    fn process_hpf(&mut self, x: f32, cutoff: f32) -> f32 {
        // Simple RC HPF approximation
        let dt = 1.0 / self.sample_rate;
        let rc = 1.0 / (2.0 * PI * cutoff);
        let alpha = rc / (rc + dt);

        let y = alpha * (self.hpf_y1 + x - self.hpf_x1);
        self.hpf_x1 = x;
        self.hpf_y1 = y;
        y
    }
}

impl AudioModule for DeesserModule {
    fn name(&self) -> &str {
        "Deesser"
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
        if let ModuleConfig::Deesser {
            enabled,
            threshold_db,
            ratio,
            attack_ms,
            release_ms,
            frequency,
        } = config
        {
            self.enabled = *enabled;
            self.threshold_db.set_target(*threshold_db);
            self.ratio.set_target(ratio.max(1.0));
            self.attack_ms = attack_ms.clamp(0.1, 1000.0);
            self.release_ms = release_ms.clamp(1.0, 5000.0);
            self.frequency.set_target(frequency.clamp(1000.0, 15000.0));
            self.update_coefficients();
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Deesser {
            enabled: self.enabled,
            threshold_db: self.threshold_db.get_target(),
            ratio: self.ratio.get_target(),
            attack_ms: self.attack_ms,
            release_ms: self.release_ms,
            frequency: self.frequency.get_target(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let threshold = self.threshold_db.next();
            let ratio = self.ratio.next();
            let freq = self.frequency.next();

            // 1. High-pass filter the signal for sibilance detection
            let sidechain = self.process_hpf(*sample, freq);
            let sidechain_abs = sidechain.abs();

            // 2. Ballistics (fast for de-essing)
            if sidechain_abs > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - sidechain_abs) + sidechain_abs;
            } else {
                self.envelope =
                    self.release_coeff * (self.envelope - sidechain_abs) + sidechain_abs;
            }

            // 3. Gain Calculation
            let input_db = Self::linear_to_db(self.envelope);
            let mut gain_db = 0.0;

            if input_db > threshold {
                gain_db = (threshold - input_db) * (1.0 - 1.0 / ratio);
            }

            // 4. Apply gain reduction to the ORIGINAL (unsidelined) signal
            let gain = Self::db_to_linear(gain_db);
            *sample *= gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deesser_initialization() {
        let deesser = DeesserModule::new(48000.0);
        assert_eq!(deesser.name(), "Deesser");
        assert!(!deesser.enabled);
    }
}
