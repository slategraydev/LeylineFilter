use crate::core::traits::{AudioModule, EQBandConfig, ModuleCategory, ModuleConfig};
use crate::utils::smoothing::ParameterSmoother;
use std::f32::consts::PI;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BandFilterType {
    LowShelf,
    HighShelf,
    Peaking,
}

impl From<&str> for BandFilterType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "lowshelf" => BandFilterType::LowShelf,
            "highshelf" => BandFilterType::HighShelf,
            _ => BandFilterType::Peaking,
        }
    }
}

struct EQBand {
    enabled: bool,
    filter_type: BandFilterType,
    freq_smoother: ParameterSmoother,
    q_smoother: ParameterSmoother,
    gain_db_smoother: ParameterSmoother,

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

impl EQBand {
    fn new(sample_rate: f32, freq: f32, q: f32, gain_db: f32, filter_type: BandFilterType) -> Self {
        let mut band = Self {
            enabled: true,
            filter_type,
            freq_smoother: ParameterSmoother::new(freq, 10.0, sample_rate),
            q_smoother: ParameterSmoother::new(q, 10.0, sample_rate),
            gain_db_smoother: ParameterSmoother::new(gain_db, 10.0, sample_rate),
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
        band.update_coefficients(sample_rate, freq, q, gain_db);
        band
    }

    fn update_coefficients(&mut self, sample_rate: f32, freq: f32, q: f32, gain_db: f32) {
        let f0 = freq.clamp(20.0, sample_rate * 0.45);
        let q = q.max(0.1);
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * f0 / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            BandFilterType::Peaking => (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            ),
            BandFilterType::LowShelf => {
                let sqrt_a = a.sqrt();
                let beta = 2.0 * sqrt_a * alpha;
                (
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 + beta),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 - beta),
                    (a + 1.0) + (a - 1.0) * cos_w0 + beta,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                    (a + 1.0) + (a - 1.0) * cos_w0 - beta,
                )
            }
            BandFilterType::HighShelf => {
                let sqrt_a = a.sqrt();
                let beta = 2.0 * sqrt_a * alpha;
                (
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 + beta),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 - beta),
                    (a + 1.0) - (a - 1.0) * cos_w0 + beta,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                    (a + 1.0) - (a - 1.0) * cos_w0 - beta,
                )
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn process_sample(&mut self, sample: f32, sample_rate: f32) -> f32 {
        if !self.enabled {
            return sample;
        }

        let freq = self.freq_smoother.next();
        let q = self.q_smoother.next();
        let gain_db = self.gain_db_smoother.next();

        self.update_coefficients(sample_rate, freq, q, gain_db);

        let x = sample;
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;

        y
    }
}

pub struct ParametricEQModule {
    id: String,
    enabled: bool,
    bands: Vec<EQBand>,
    sample_rate: f32,
}

impl ParametricEQModule {
    pub fn new(sample_rate: f32) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), sample_rate)
    }

    pub fn with_id(id: String, sample_rate: f32) -> Self {
        let bands = vec![
            EQBand::new(sample_rate, 100.0, 0.707, 0.0, BandFilterType::LowShelf),
            EQBand::new(sample_rate, 1000.0, 1.0, 0.0, BandFilterType::Peaking),
            EQBand::new(sample_rate, 8000.0, 0.707, 0.0, BandFilterType::HighShelf),
        ];

        Self {
            id,
            enabled: false,
            bands,
            sample_rate,
        }
    }
}

impl AudioModule for ParametricEQModule {
    fn name(&self) -> &str {
        "ParametricEQ"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn category(&self) -> ModuleCategory {
        ModuleCategory::Filter
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for band in &mut self.bands {
            band.freq_smoother.set_smoothing_ms(10.0, sample_rate);
            band.q_smoother.set_smoothing_ms(10.0, sample_rate);
            band.gain_db_smoother.set_smoothing_ms(10.0, sample_rate);
        }
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::ParametricEQ { enabled, bands } = config {
            self.enabled = *enabled;
            for (i, band_config) in bands.iter().enumerate() {
                if i < self.bands.len() {
                    let band = &mut self.bands[i];
                    band.enabled = band_config.enabled;
                    band.filter_type = BandFilterType::from(band_config.filter_type.as_str());
                    band.freq_smoother.set_target(band_config.frequency);
                    band.q_smoother.set_target(band_config.q);
                    band.gain_db_smoother.set_target(band_config.gain_db);
                }
            }
        }
    }

    fn get_config(&self) -> ModuleConfig {
        let bands = self
            .bands
            .iter()
            .map(|b| EQBandConfig {
                enabled: b.enabled,
                filter_type: format!("{:?}", b.filter_type),
                frequency: b.freq_smoother.get_target(),
                q: b.q_smoother.get_target(),
                gain_db: b.gain_db_smoother.get_target(),
            })
            .collect();

        ModuleConfig::ParametricEQ {
            enabled: self.enabled,
            bands,
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let mut out = *sample;
            for band in &mut self.bands {
                out = band.process_sample(out, self.sample_rate);
            }
            *sample = out;
        }
    }
}
