use serde::{Deserialize, Serialize};

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
    RNNoise { enabled: bool },
    /// Configuration for a Gain module.
    Gain { enabled: bool, gain_db: f32 },
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
    /// Configuration for the Visualizer module.
    Visualizer { enabled: bool },
    /// Placeholder for no configuration.
    None,
}

/// A MIDI message for synth and parameter control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMessage {
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

/// A unified command for controlling the audio engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EngineCommand {
    /// Update the configuration of a specific module.
    UpdateConfig(ModuleConfig),
    /// Add a new module to the signal chain.
    AddModule { module_type: String },
    /// Remove a module from the signal chain.
    RemoveModule { id: String },
    /// Set a specific parameter of a module by ID.
    SetParam {
        id: String,
        param: String,
        value: f32,
    },
    /// Send a MIDI message to the engine.
    MidiEvent(MidiMessage),
    /// Reorder modules in the signal chain.
    Reorder { order: Vec<String> },
}

/// Information about a module for UI synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub category: ModuleCategory,
    pub enabled: bool,
    pub config: ModuleConfig,
}

/// The complete state of the audio engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineState {
    pub modules: Vec<ModuleInfo>,
    pub is_running: bool,
    pub sample_rate: f32,
    pub buffer_size: u32,
}

/// Categories for audio modules to help with UI organization and processing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleCategory {
    Dynamics,
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
    fn latency_samples(&self) -> usize {
        0
    }

    /// Returns the required sample rate and block size for this module, if any.
    /// (Sample Rate, Block Size)
    fn requirements(&self) -> (Option<f32>, Option<usize>) {
        (None, None)
    }

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

    /// Returns the current configuration of the module.
    fn get_config(&self) -> ModuleConfig;

    /// Returns whether the module is currently enabled.
    fn is_enabled(&self) -> bool {
        match self.get_config() {
            ModuleConfig::Expander { enabled, .. } => enabled,
            ModuleConfig::RNNoise { enabled, .. } => enabled,
            ModuleConfig::Gain { enabled, .. } => enabled,
            ModuleConfig::Compressor { enabled, .. } => enabled,
            ModuleConfig::Filter { enabled, .. } => enabled,
            ModuleConfig::FX { enabled, .. } => enabled,
            ModuleConfig::Visualizer { enabled, .. } => enabled,
            _ => true,
        }
    }
}
