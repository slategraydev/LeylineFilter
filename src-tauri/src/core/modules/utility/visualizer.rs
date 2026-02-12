use crate::core::traits::{AudioModule, ModuleCategory, ModuleConfig};
use uuid::Uuid;

/// A module that represents the Visualizer in the signal chain.
/// It doesn't process audio directly (FFT is handled by the engine)
/// but provides a handle for the UI to move it around.
pub struct VisualizerModule {
    id: String,
    enabled: bool,
}

impl VisualizerModule {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            enabled: true,
        }
    }
}

impl AudioModule for VisualizerModule {
    fn name(&self) -> &str {
        "Visualizer"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Utility
    }

    fn prepare(&mut self, _sample_rate: f32) {
        // No preparation needed
    }

    fn update_config(&mut self, config: &ModuleConfig) {
        if let ModuleConfig::Visualizer { enabled } = config {
            self.enabled = *enabled;
        }
    }

    fn get_config(&self) -> ModuleConfig {
        ModuleConfig::Visualizer {
            enabled: self.enabled,
        }
    }

    fn process(&mut self, _samples: &mut [f32]) {
        // Visualizer is a pass-through module.
        // The engine handles the actual analysis.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_initialization() {
        let viz = VisualizerModule::new();
        assert_eq!(viz.name(), "Visualizer");
        assert!(viz.enabled);
    }

    #[test]
    fn test_visualizer_config() {
        let mut viz = VisualizerModule::new();
        viz.update_config(&ModuleConfig::Visualizer { enabled: false });
        assert!(!viz.enabled);

        if let ModuleConfig::Visualizer { enabled } = viz.get_config() {
            assert!(!enabled);
        } else {
            panic!("Wrong config type returned");
        }
    }
}
