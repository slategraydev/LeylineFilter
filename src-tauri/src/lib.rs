mod core;
mod error;
mod utils;

use crate::core::audio::AudioEngine;
use crate::core::traits::{EngineCommand, EngineState, ModuleConfig};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

/// Global application state.
pub struct AppState {
    /// The core audio engine, protected by a Mutex for thread-safe access from Tauri commands.
    pub engine: Arc<Mutex<AudioEngine>>,
}

/// Tauri command to update a module's configuration.
#[tauri::command]
async fn update_config(
    state: State<'_, AppState>,
    config: ModuleConfig,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.update_module_config(config);
    Ok(())
}

/// Tauri command to get the full engine state.
#[tauri::command]
async fn get_engine_state(state: State<'_, AppState>) -> std::result::Result<EngineState, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_engine_state())
}

/// Tauri command to send a command to the engine.
#[tauri::command]
async fn send_command(
    state: State<'_, AppState>,
    command: EngineCommand,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.send_command(command);
    Ok(())
}

/// Tauri command to start the audio engine.
#[tauri::command]
async fn start_engine(
    state: State<'_, AppState>,
    input_device: Option<String>,
    output_device: Option<String>,
) -> std::result::Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine
        .start(input_device, output_device)
        .map_err(|e| e.to_string())
}

/// Tauri command to get available output devices.
#[tauri::command]
async fn get_output_devices(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<String>, String> {
    log::info!("Fetching output devices...");
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let devices = engine.get_output_devices();
    log::info!("Found {} output devices", devices.len());
    Ok(devices)
}

/// Tauri command to get available input devices.
#[tauri::command]
async fn get_input_devices(state: State<'_, AppState>) -> std::result::Result<Vec<String>, String> {
    log::info!("Fetching input devices...");
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let devices = engine.get_input_devices();
    log::info!("Found {} input devices", devices.len());
    Ok(devices)
}

/// Tauri command to stop the audio engine.
#[tauri::command]
async fn stop_engine(state: State<'_, AppState>) -> std::result::Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.stop();
    Ok(())
}

/// Metrics exported to the frontend.
#[derive(Serialize, Deserialize, Clone)]
pub struct Metrics {
    latency_ms: f32,
    cpu_usage: f32,
    input_level: f32,
    spectrum: [f32; 12],
    tonality: [f32; 12],
    state_version: u32,
}

impl Metrics {
    pub fn from_engine(engine: &mut AudioEngine) -> Self {
        let is_running = engine.is_running();
        let (_, cpu, level, spectrum, tonality, version) = engine.metrics.get();

        // If not running, latency should be reported as 0.0
        let latency = if !is_running {
            0.0
        } else {
            engine.get_total_latency_ms()
        };

        Metrics {
            latency_ms: latency.round(),
            cpu_usage: (cpu * 10.0).round() / 10.0,
            input_level: level,
            spectrum,
            tonality,
            state_version: version,
        }
    }
}

/// Tauri command to retrieve current engine metrics.
#[tauri::command]
async fn get_metrics(state: State<'_, AppState>) -> std::result::Result<Metrics, String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;

    // Always update CPU usage
    engine.update_cpu_usage();

    // Process garbage collected from the audio thread
    engine.process_garbage();

    Ok(Metrics::from_engine(&mut engine))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::audio::AudioEngine;

    #[test]
    fn test_metrics_formatting() {
        let mut engine = AudioEngine::new();
        // By default, engine is not running, so latency should be 0.0
        let metrics = Metrics::from_engine(&mut engine);

        assert_eq!(metrics.latency_ms, 0.0);
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.state_version, 0);
    }
}
/// The main entry point for the Tauri application.
///
/// Initializes the global state, sets up the logger, and builds the Tauri application.
/// It also handles the application lifecycle, ensuring that the audio engine is
/// stopped gracefully when the application exits.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::logger::init();
    log::info!("Initializing LeylineFilter Modular Engine...");

    let state = AppState {
        engine: Arc::new(Mutex::new(AudioEngine::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            start_engine,
            stop_engine,
            get_metrics,
            update_config,
            get_input_devices,
            get_output_devices,
            get_engine_state,
            send_command
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                log::info!("Application exiting...");
                let _ = app_handle
                    .state::<AppState>()
                    .engine
                    .lock()
                    .map(|mut engine| engine.stop());
            }
        });
}
