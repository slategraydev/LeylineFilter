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
    /// Configuration for a Gain module.
    Gain {
        enabled: bool,
        gain_db: f32,
    },
    /// Configuration for a Compressor module.
    #[allow(dead_code)]
    Compressor {
        enabled: bool,
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        knee_db: f32,
        makeup_gain_db: f32,
    },
    /// Configuration for a Filter module (HPF/LPF).
    #[allow(dead_code)]
    Filter {
        enabled: bool,
        filter_type: String,
        frequency: f32,
        q: f32,
    },
    /// Configuration for an FX module (Reverb/Delay).
    #[allow(dead_code)]
    FX {
        enabled: bool,
        fx_type: String,
        mix: f32,
        params: std::collections::HashMap<String, f32>,
    },
    /// Placeholder for no configuration.
    None,
}

/// Categories for audio modules to help with UI organization and processing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleCategory {
    Dynamics,
    #[allow(dead_code)]
    Filter,
    Voice,
    #[allow(dead_code)]
    FX,
    #[allow(dead_code)]
    Synth,
    Utility,
}

/// A trait defining the interface for an audio processing module.
///
/// Modules are expected to process audio in-place and handle their own
/// internal state.
pub trait AudioModule: Send + Sync {
    /// Returns the unique name of the module.
    fn name(&self) -> &str;

    /// Returns a unique identifier for the module instance.
    fn id(&self) -> &str;

    /// Returns the category of the module.
    fn category(&self) -> ModuleCategory;

    /// Returns the latency introduced by this module in samples.
    fn latency_samples(&self) -> usize { 0 }

    /// Returns the required sample rate and block size for this module, if any.
    /// (Sample Rate, Block Size)
    fn requirements(&self) -> (Option<f32>, Option<usize>) { (None, None) }

    /// Prepares the module for processing with a specific sample rate.
    ///
    /// This is called before any processing starts or when the sample rate changes.
    /// # Arguments
    /// * `sample_rate` - The sample rate in Hz.
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
