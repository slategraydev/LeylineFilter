// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

use serde::{Deserialize, Serialize};
use crate::core::traits::ModuleInfo;
use std::collections::HashMap;

/// # Grid Position
/// A logical position in the module grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridPosition {
    pub gx: i32,
    pub gy: i32,
}

/// # App Configuration
/// This struct represents the full state of the application that should be persisted.
/// It includes both the signal chain and the hardware selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub monitoring_enabled: bool,
    pub engine_running: bool,
    pub modules: Vec<ModuleInfo>,
    // UI Layout Persistence
    pub positions: HashMap<String, GridPosition>,
    pub heights: HashMap<String, u32>,
    pub widths: HashMap<String, u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_device: Some("Default".to_string()),
            output_device: Some("Default".to_string()),
            monitoring_enabled: false,
            engine_running: false,
            modules: Vec::new(),
            positions: HashMap::new(),
            heights: HashMap::new(),
            widths: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.input_device, Some("Default".to_string()));
        assert!(!config.engine_running);
        assert!(!config.monitoring_enabled);
        assert!(config.modules.is_empty());
    }

    #[test]
    fn test_app_config_serialization() {
        let mut config = AppConfig::default();
        config.input_device = Some("Test Mic".to_string());
        config.positions.insert("mod1".to_string(), GridPosition { gx: 10, gy: 20 });

        let json = serde_json::to_string(&config).unwrap();
        let de_config: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(de_config.input_device, Some("Test Mic".to_string()));
        assert_eq!(de_config.positions.get("mod1").unwrap().gx, 10);
    }
}
