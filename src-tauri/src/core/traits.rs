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
    /// Configuration for the standard RNNoise module.
    RNNoise {
        enabled: bool,
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

    /// Returns the latency introduced by this module in samples.
    fn latency_samples(&self) -> usize { 0 }

    /// Prepares the module for processing with a specific sample rate.
    ///
    /// This is called before any processing starts or when the sample rate changes.
    /// # Arguments
    /// * `sample_rate` - The sample rate in Hz.
    #[allow(dead_code)]
    fn prepare(&mut self, sample_rate: f32);

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
