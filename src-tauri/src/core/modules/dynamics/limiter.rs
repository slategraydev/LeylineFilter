// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use std::collections::VecDeque;
use uuid::Uuid;

/// A look-ahead brickwall limiter module.
pub struct LimiterModule {
    id: String,
    enabled: bool,
    threshold_db: ParameterSmoother,
    release_ms: f32,

    sample_rate: f32,
    delay_line: VecDeque<f32>,
    envelope: f32,
    release_coeff: f32,
    lookahead_samples: usize,
}

impl LimiterModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        // Fixed 5ms lookahead
        let lookahead_ms = 5.0;
        let lookahead_samples = (lookahead_ms * sample_rate / 1000.0) as usize;

        let mut m = Self {
            id,
            enabled: false,
            threshold_db: ParameterSmoother::new(-0.1, 10.0, sample_rate),
            release_ms: 100.0,
            sample_rate,
            delay_line: VecDeque::from(vec![0.0; lookahead_samples]),
            envelope: 0.0,
            release_coeff: 0.0,
            lookahead_samples,
        };
        m.update_coefficients();
        m
    }

    fn update_coefficients(&mut self) {
        self.release_coeff = (-1.0 / (self.release_ms * self.sample_rate / 1000.0)).exp();
    }

    #[allow(dead_code)]
    fn linear_to_db(linear: f32) -> f32 {
        20.0 * linear.max(1e-6).log10()
    }

    #[allow(dead_code)]
    fn db_to_linear(db: f32) -> f32 {
        10.0f32.powf(db / 20.0)
    }
}

impl AudioModule for LimiterModule {
    fn name(&self) -> &str {
        "Limiter"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Dynamics
    }

    fn prepare(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() > 1.0 {
            self.sample_rate = sample_rate;
            let lookahead_ms = 5.0;
            self.lookahead_samples = (lookahead_ms * sample_rate / 1000.0) as usize;
            self.delay_line = VecDeque::from(vec![0.0; self.lookahead_samples]);
            self.update_coefficients();
            self.threshold_db.set_smoothing_ms(10.0, sample_rate);
        }
    }

    fn latency_samples(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        self.lookahead_samples
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Limiter {
            enabled,
            threshold_db,
            release_ms,
        } = config
        {
            self.enabled = *enabled;
            self.threshold_db.set_target(threshold_db.clamp(-60.0, 0.0));
            self.release_ms = release_ms.clamp(1.0, 1000.0);
            self.update_coefficients();
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Limiter {
            enabled: self.enabled,
            threshold_db: self.threshold_db.get_target(),
            release_ms: self.release_ms,
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            // Drain delay line smoothly if we have remaining samples
            for sample in samples.iter_mut() {
                self.delay_line.push_back(*sample);
                *sample = self.delay_line.pop_front().unwrap_or(0.0);
            }
            return;
        }

        for sample in samples.iter_mut() {
            let threshold_db = self.threshold_db.next();
            let threshold_linear = Self::db_to_linear(threshold_db);

            let input = *sample;
            let input_abs = input.abs();

            // Envelope follower (Instant attack, slow release)
            if input_abs > self.envelope {
                self.envelope = input_abs;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_abs) + input_abs;
            }

            // Calculate gain reduction
            let gain = if self.envelope > threshold_linear {
                threshold_linear / self.envelope
            } else {
                1.0
            };

            // Apply gain to the sample coming OUT of the delay line (look-ahead)
            self.delay_line.push_back(input);
            let delayed_sample = self.delay_line.pop_front().unwrap_or(0.0);

            *sample = delayed_sample * gain;

            // Final hard clip safety
            if *sample > threshold_linear {
                *sample = threshold_linear;
            } else if *sample < -threshold_linear {
                *sample = -threshold_linear;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_clamping() {
        let mut limiter = LimiterModule::new(48000.0);
        limiter.update_config(&ModuleConfig::Limiter {
            enabled: true,
            threshold_db: -6.0,
            release_ms: 100.0,
        });

        // 0dB input (1.0)
        let mut samples = vec![1.0; 480];

        // Process enough to let smoothing settle (10ms smoothing @ 48kHz = 480 samples time constant)
        // 4800 samples = 10 time constants, definitely settled.
        let mut settle = vec![0.0; 4800];
        limiter.process(&mut settle);

        limiter.process(&mut samples);
        // Threshold -6dB is approx 0.501
        for &s in samples.iter() {
            assert!(s.abs() <= 0.51, "Sample {} exceeded threshold", s);
        }
    }
}
