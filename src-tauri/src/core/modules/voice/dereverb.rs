// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use uuid::Uuid;

/// A Room De-Reverb module based on transient enhancement and sustain suppression.
pub struct DereverbModule {
    id: String,
    enabled: bool,
    reduction: ParameterSmoother,
    sensitivity: ParameterSmoother,

    sample_rate: f32,
    fast_envelope: f32, // Attack focused
    slow_envelope: f32, // Tail focused

    fast_attack: f32,
    fast_release: f32,
    slow_attack: f32,
    slow_release: f32,
}

impl DereverbModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let mut m = Self {
            id,
            enabled: false,
            reduction: ParameterSmoother::new(0.5, 10.0, sample_rate),
            sensitivity: ParameterSmoother::new(0.5, 10.0, sample_rate),
            sample_rate,
            fast_envelope: 0.0,
            slow_envelope: 0.0,
            fast_attack: 0.0,
            fast_release: 0.0,
            slow_attack: 0.0,
            slow_release: 0.0,
        };
        m.update_coefficients();
        m
    }

    fn update_coefficients(&mut self) {
        // Fast follower for transients
        self.fast_attack = (-1.0 / (2.0 * self.sample_rate / 1000.0)).exp();
        self.fast_release = (-1.0 / (10.0 * self.sample_rate / 1000.0)).exp();

        // Slow follower for room tone/sustain
        self.slow_attack = (-1.0 / (20.0 * self.sample_rate / 1000.0)).exp();
        self.slow_release = (-1.0 / (500.0 * self.sample_rate / 1000.0)).exp();
    }
}

impl AudioModule for DereverbModule {
    fn name(&self) -> &str {
        "De-Reverb"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Voice
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
        self.reduction.set_smoothing_ms(10.0, sample_rate);
        self.sensitivity.set_smoothing_ms(10.0, sample_rate);
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Dereverb {
            enabled,
            reduction,
            sensitivity,
        } = config
        {
            self.enabled = *enabled;
            self.reduction.set_target(reduction.clamp(0.0, 1.0));
            self.sensitivity.set_target(sensitivity.clamp(0.0, 1.0));
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Dereverb {
            enabled: self.enabled,
            reduction: self.reduction.get_target(),
            sensitivity: self.sensitivity.get_target(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let reduction_target = self.reduction.next();
        let sensitivity = self.sensitivity.next();

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // Fast envelope (transients)
            if input_abs > self.fast_envelope {
                self.fast_envelope =
                    self.fast_attack * (self.fast_envelope - input_abs) + input_abs;
            } else {
                self.fast_envelope =
                    self.fast_release * (self.fast_envelope - input_abs) + input_abs;
            }

            // Slow envelope (sustain/room)
            if input_abs > self.slow_envelope {
                self.slow_envelope =
                    self.slow_attack * (self.slow_envelope - input_abs) + input_abs;
            } else {
                self.slow_envelope =
                    self.slow_release * (self.slow_envelope - input_abs) + input_abs;
            }

            // Calculate gain based on envelope ratio
            // If fast >> slow, it's a transient -> gain ~ 1.0
            // If fast ~ slow, it's sustain/reverb -> apply reduction

            let ratio = if self.fast_envelope > 1e-6 {
                (self.slow_envelope / self.fast_envelope).clamp(0.0, 1.0)
            } else {
                0.0
            };

            // Suppression amount increases as ratio increases (more sustain-like)
            // Sensitivity adjusts how early we start suppressing
            let suppression_curve = ratio.powf(2.0 - sensitivity);
            let gain = 1.0 - (suppression_curve * reduction_target);

            *sample *= gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dereverb_initialization() {
        let m = DereverbModule::new(48000.0);
        assert_eq!(m.name(), "De-Reverb");
        assert!(!m.enabled);
    }
}
