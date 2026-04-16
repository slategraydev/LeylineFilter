// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// SATURATION MODULE
// Harmonic distortion/saturation module.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use std::f32::consts::PI;
use uuid::Uuid;

/// A Tube Saturation module with drive, tilt, and dry/wet mix.
pub struct SaturationModule {
    id: String,
    enabled: bool,
    drive: ParameterSmoother,
    tilt: ParameterSmoother, // High shelf to bias harmonics
    mix: ParameterSmoother,

    sample_rate: f32,

    // Tilt Filter (Biquad High Shelf)
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl SaturationModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let mut m = Self {
            id,
            enabled: false,
            drive: ParameterSmoother::new(1.0, 10.0, sample_rate),
            tilt: ParameterSmoother::new(0.0, 10.0, sample_rate),
            mix: ParameterSmoother::new(1.0, 10.0, sample_rate),
            sample_rate,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        m.update_tilt_coefficients();
        m
    }

    fn update_tilt_coefficients(&mut self) {
        // Simple High Shelf at 2kHz for tilt
        let freq = 2000.0;
        let gain_db = self.tilt.get_target();
        let q = 0.707;

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn process_tilt(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    fn saturate(&self, x: f32, drive: f32) -> f32 {
        // Soft clipper using tanh approximation
        // drive is 1.0 to 10.0
        let val = x * drive;

        // tanh(x) approx: (e^x - e^-x) / (e^x + e^-x)
        // For audio, we can often just use: val / (1.0 + val.abs()) for faster soft clipping
        // or a polynomial approximation. Let's use tanh for quality.
        val.tanh()
    }
}

impl AudioModule for SaturationModule {
    fn name(&self) -> &str {
        "Saturation"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::FX
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_tilt_coefficients();
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Saturation {
            enabled,
            drive,
            tilt,
            mix,
        } = config
        {
            self.enabled = *enabled;
            self.drive.set_target(drive.clamp(1.0, 10.0));
            self.tilt.set_target(tilt.clamp(-12.0, 12.0));
            self.mix.set_target(mix.clamp(0.0, 1.0));
            self.update_tilt_coefficients();
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Saturation {
            enabled: self.enabled,
            drive: self.drive.get_target(),
            tilt: self.tilt.get_target(),
            mix: self.mix.get_target(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let input = *sample;
            let drive = self.drive.next();
            let mix = self.mix.next();

            // 1. Apply Tilt (Pre-saturation EQ)
            let tilted = self.process_tilt(input);

            // 2. Saturate
            let saturated = self.saturate(tilted, drive);

            // 3. Dry/Wet Mix
            *sample = input * (1.0 - mix) + saturated * mix;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturation_initialization() {
        let sat = SaturationModule::new(48000.0);
        assert_eq!(sat.name(), "Saturation");
        assert!(!sat.enabled);
    }
}
