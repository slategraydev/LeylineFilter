use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use std::f32::consts::PI;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

impl From<&str> for FilterType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "highpass" | "hpf" => FilterType::HighPass,
            "bandpass" | "bpf" => FilterType::BandPass,
            "notch" => FilterType::Notch,
            _ => FilterType::LowPass,
        }
    }
}

/// A Biquad filter module.
pub struct BiquadModule {
    id: String,
    enabled: bool,
    filter_type: FilterType,
    freq_smoother: ParameterSmoother,
    q_smoother: ParameterSmoother,
    sample_rate: f32,

    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // State
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let mut module = Self {
            id,
            enabled: false,
            filter_type: FilterType::LowPass,
            freq_smoother: ParameterSmoother::new(1000.0, 10.0, sample_rate),
            q_smoother: ParameterSmoother::new(0.707, 10.0, sample_rate),
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
        module.update_coefficients(1000.0, 0.707);
        module
    }

    fn update_coefficients(&mut self, freq: f32, q: f32) {
        let f0 = freq.clamp(20.0, self.sample_rate * 0.45);
        let q = q.max(0.01);

        let w0 = 2.0 * PI * f0 / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            FilterType::LowPass => (
                (1.0 - cos_w0) / 2.0,
                1.0 - cos_w0,
                (1.0 - cos_w0) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            FilterType::HighPass => (
                (1.0 + cos_w0) / 2.0,
                -(1.0 + cos_w0),
                (1.0 + cos_w0) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            FilterType::BandPass => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha),
            FilterType::Notch => (
                1.0,
                -2.0 * cos_w0,
                1.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl AudioModule for BiquadModule {
    fn name(&self) -> &str {
        "Filter"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Filter
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.freq_smoother.set_smoothing_ms(10.0, sample_rate);
        self.q_smoother.set_smoothing_ms(10.0, sample_rate);
        let f = self.freq_smoother.current();
        let q = self.q_smoother.current();
        self.update_coefficients(f, q);
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Filter {
            enabled,
            filter_type,
            frequency,
            q,
        } = config
        {
            self.enabled = *enabled;
            self.filter_type = FilterType::from(filter_type.as_str());
            self.freq_smoother.set_target(*frequency);
            self.q_smoother.set_target(*q);
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Filter {
            enabled: self.enabled,
            filter_type: format!("{:?}", self.filter_type),
            frequency: self.freq_smoother.get_target(),
            q: self.q_smoother.get_target(),
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let freq = self.freq_smoother.next();
            let q = self.q_smoother.next();

            // For now, update every sample for maximum smoothness.
            // If CPU becomes an issue, we can throttle this.
            self.update_coefficients(freq, q);

            let x = *sample;
            let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1
                - self.a2 * self.y2;

            self.x2 = self.x1;
            self.x1 = x;
            self.y2 = self.y1;
            self.y1 = y;

            *sample = y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_initialization() {
        let filter = BiquadModule::new(48000.0);
        assert_eq!(filter.name(), "Filter");
        assert!(!filter.enabled);
    }

    #[test]
    fn test_lpf_processing() {
        let mut filter = BiquadModule::new(48000.0);
        filter.update_config(&ModuleConfig::Filter {
            enabled: true,
            filter_type: "LowPass".to_string(),
            frequency: 100.0, // Low cutoff
            q: 0.707,
        });

        let mut samples = vec![0.0; 1000];
        // Impulse
        samples[0] = 1.0;

        filter.process(&mut samples);

        // Output should be smoothed impulse
        assert!(samples[0] < 1.0);
        assert!(samples[1] > 0.0);
    }
}
