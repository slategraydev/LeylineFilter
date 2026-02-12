pub mod dynamics;
pub mod voice;
pub mod filter;
pub mod fx;
pub mod synth;
pub mod utility;

use crate::core::traits::{AudioModule};
use crate::core::modules::dynamics::expander::ExpanderModule;
use crate::core::modules::voice::rnnoise::RNNoiseModule;
use crate::core::modules::utility::gain::GainModule;

/// A factory for creating audio modules by type.
pub struct ModuleFactory;

impl ModuleFactory {
    /// Creates a new audio module instance based on its type name.
    pub fn create(module_type: &str, sample_rate: f32) -> Option<Box<dyn AudioModule>> {
        match module_type {
            "Gain" => Some(Box::new(GainModule::new(sample_rate))),
            "Expander" => Some(Box::new(ExpanderModule::new(sample_rate))),
            "RNNoise" => Some(Box::new(RNNoiseModule::new(sample_rate))),
            _ => None,
        }
    }

    /// Returns a list of all available module types.
    pub fn available_types() -> Vec<&'static str> {
        vec!["Gain", "Expander", "RNNoise"]
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

        let unknown = ModuleFactory::create("Unknown", 48000.0);
        assert!(unknown.is_none());
    }

    #[test]
    fn test_available_types() {
        let types = ModuleFactory::available_types();
        assert!(types.contains(&"Gain"));
        assert!(types.contains(&"Expander"));
        assert!(types.contains(&"RNNoise"));
    }
}
