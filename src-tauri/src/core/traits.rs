use serde::{Serialize, Deserialize};

/// Configuration options for the various audio modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ModuleConfig {
    /// Configuration for the Expander/Gate module.
    Expander {
        enabled: bool,
        threshold: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
    },
    /// Placeholder for no configuration.
    None,
}

/// A trait defining the interface for an audio processing module.
///
/// Modules are expected to process audio in-place and handle their own
/// internal state.
pub trait AudioModule: Send + Sync {
    /// Returns the unique name of the module.
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Processes a block of audio samples in-place.
    ///
    /// # Arguments
    /// * `audio` - A mutable slice of audio samples.
    fn process(&mut self, audio: &mut [f32]);

    /// Updates the module's internal configuration.
    ///
    /// # Arguments
    /// * `config` - The new configuration to apply.
    fn update_config(&mut self, config: &ModuleConfig);
}
