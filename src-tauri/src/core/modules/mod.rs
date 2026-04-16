// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// MODULE REGISTRY
// Dynamic instantiation of audio modules via the Registry/Factory pattern.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub mod dynamics;
pub mod filter;
pub mod fx;
pub mod synth;
pub mod utility;
pub mod voice;

use crate::core::modules::dynamics::compressor::CompressorModule;
use crate::core::modules::dynamics::deesser::DeesserModule;
use crate::core::modules::dynamics::expander::ExpanderModule;
use crate::core::modules::dynamics::limiter::LimiterModule;
use crate::core::modules::filter::biquad::BiquadModule;
use crate::core::modules::filter::parametric_eq::ParametricEQModule;
use crate::core::modules::fx::saturation::SaturationModule;
use crate::core::modules::utility::gain::GainModule;
use crate::core::modules::voice::dereverb::DereverbModule;
use crate::core::modules::voice::rnnoise::RNNoiseModule;
use crate::core::traits::AudioModule;

/// A factory for creating audio modules by type.
///
/// # Registry Pattern
/// This central registry allows the Engine to instantiate modules dynamically
/// via string identifiers (e.g., from a JSON config or UI selection).
/// It decouples the core engine loop from specific module implementations.
pub struct ModuleFactory;

impl ModuleFactory {
    /// Creates a new audio module instance based on its type name.
    pub fn create(module_type: &str, sample_rate: f32) -> Option<Box<dyn AudioModule>> {
        match module_type {
            "Gain" => Some(Box::new(GainModule::new(sample_rate))),
            "Expander" => Some(Box::new(ExpanderModule::new(sample_rate))),
            "RNNoise" => Some(Box::new(RNNoiseModule::new(sample_rate))),
            "Filter" => Some(Box::new(BiquadModule::new(sample_rate))),
            "Compressor" => Some(Box::new(CompressorModule::new(sample_rate))),
            "ParametricEQ" => Some(Box::new(ParametricEQModule::new(sample_rate))),
            "Deesser" => Some(Box::new(DeesserModule::new(sample_rate))),
            "Saturation" => Some(Box::new(SaturationModule::new(sample_rate))),
            "Limiter" => Some(Box::new(LimiterModule::new(sample_rate))),
            "Dereverb" => Some(Box::new(DereverbModule::new(sample_rate))),
            _ => None,
        }
    }

    /// Creates a new audio module instance with a specific ID.
    pub fn create_with_id(
        module_type: &str,
        id: String,
        sample_rate: f32,
    ) -> Option<Box<dyn AudioModule>> {
        match module_type {
            "Gain" => Some(Box::new(GainModule::with_id(id, sample_rate))),
            "Expander" => Some(Box::new(ExpanderModule::with_id(id, sample_rate))),
            "RNNoise" => Some(Box::new(RNNoiseModule::with_id(id, sample_rate))),
            "Filter" => Some(Box::new(BiquadModule::with_id(id, sample_rate))),
            "Compressor" => Some(Box::new(CompressorModule::with_id(id, sample_rate))),
            "ParametricEQ" => Some(Box::new(ParametricEQModule::with_id(id, sample_rate))),
            "Deesser" => Some(Box::new(DeesserModule::with_id(id, sample_rate))),
            "Saturation" => Some(Box::new(SaturationModule::with_id(id, sample_rate))),
            "Limiter" => Some(Box::new(LimiterModule::with_id(id, sample_rate))),
            "Dereverb" => Some(Box::new(DereverbModule::with_id(id, sample_rate))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let gain = ModuleFactory::create("Gain", 48000.0);
        assert!(gain.is_some());
        assert_eq!(gain.unwrap().name(), "Gain");

        let expander = ModuleFactory::create("Expander", 48000.0);
        assert!(expander.is_some());
        assert_eq!(expander.unwrap().name(), "Expander");

        let rnnoise = ModuleFactory::create("RNNoise", 48000.0);
        assert!(rnnoise.is_some());
        assert_eq!(rnnoise.unwrap().name(), "RNNoise");

        let filter = ModuleFactory::create("Filter", 48000.0);
        assert!(filter.is_some());
        assert_eq!(filter.unwrap().name(), "Filter");

        let unknown = ModuleFactory::create("Unknown", 48000.0);
        assert!(unknown.is_none());
    }
}
