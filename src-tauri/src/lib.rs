// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// BACKEND ENTRY POINT
// Application lifecycle management, Tauri command registration, and global state initialization.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

mod core;
mod error;
mod utils;

use crate::core::audio::AudioEngine;
use crate::core::traits::{EngineCommand, EngineState, ModuleConfig};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;

/// Global application state.
///
/// # Tauri Integration Pattern
/// The `AppState` struct holds a thread-safe reference to the `AudioEngine`.
/// Since Tauri commands are async and run on the Tokio runtime, we wrap the engine
/// in an `Arc<Mutex<>>` to allow shared access across commands.
///
/// **Note:** The Audio Thread (internal to `AudioEngine`) does NOT lock this Mutex during processing.
/// It uses its own lock-free communication channels.
pub struct AppState {
    pub engine: Arc<Mutex<AudioEngine>>,
}

use crate::core::persistence::{AppConfig, GridPosition};
use std::collections::HashMap;
use std::fs;

/// Internal helper to persist the engine state to disk.
fn save_to_disk(app: &tauri::AppHandle, engine: &AudioEngine) -> std::result::Result<(), String> {
    let config = engine.get_persistence_config();
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    let config_path = config_dir.join("session.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(config_path, json).map_err(|e| e.to_string())?;

    Ok(())
}

/// Tauri command to update the layout metadata.
#[tauri::command]
async fn update_layout(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    positions: HashMap<String, GridPosition>,
    heights: HashMap<String, u32>,
    widths: HashMap<String, u32>,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    *engine.positions.lock().unwrap() = positions;
    *engine.heights.lock().unwrap() = heights;
    *engine.widths.lock().unwrap() = widths;

    // Auto-save layout changes
    let _ = save_to_disk(&app, &engine);
    Ok(())
}

/// Tauri command to save the current session to disk.
#[tauri::command]
async fn save_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    save_to_disk(&app, &engine)
}

/// Tauri command to load the session from disk.
#[tauri::command]
async fn load_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let config_path = config_dir.join("session.json");

    if !config_path.exists() {
        return Err("No saved session found".to_string());
    }

    let json = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.apply_persistence_config(config.clone());

    // If it was running, try to restart it
    if config.engine_running {
        let _ = engine.start(config.input_device, config.output_device, None);
    }

    log::info!("Session loaded successfully");
    Ok(())
}

/// Tauri command to update a module's configuration.
#[tauri::command]
async fn update_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    config: ModuleConfig,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.update_module_config(config);

    // Auto-save config changes
    let _ = save_to_disk(&app, &engine);
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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    command: EngineCommand,
) -> std::result::Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.send_command(command);

    // Auto-save on module add/remove/reorder
    let _ = save_to_disk(&app, &engine);
    Ok(())
}

/// Tauri command to start the audio engine.
#[tauri::command]
async fn start_engine(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input_device: Option<String>,
    output_device: Option<String>,
) -> std::result::Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine
        .start(input_device, output_device, None)
        .map_err(|e| e.to_string())?;

    // Auto-save device selection and running state
    let _ = save_to_disk(&app, &engine);
    Ok(())
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
async fn stop_engine(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.stop();

    // Auto-save stopped state
    let _ = save_to_disk(&app, &engine);
    Ok(())
}

#[tauri::command]
async fn set_monitoring(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> std::result::Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_monitoring(enabled);

    // Auto-save monitoring state
    let _ = save_to_disk(&app, &engine);
    Ok(())
}

/// Metrics exported to the frontend.
#[derive(Serialize, Deserialize, Clone)]
pub struct Metrics {
    latency_ms: f32,
    cpu_usage: f32,
    input_level: f32,
    input_level_db: f32,
    buffer_size: u32,
    spectrum: Vec<f32>,
    tonality: Vec<f32>,
    waveform: Vec<f32>,
    state_version: u32,
}

impl Metrics {
    /// # Metrics Formatting Logic
    /// Extracted for testability.
    /// This allows us to unit test the data transformation without initializing the full Tauri runtime.
    pub fn from_engine(engine: &mut AudioEngine) -> Self {
        let is_running = engine.is_running();
        let (_, cpu, level, level_db, buffer_size, spectrum, tonality, waveform, version) =
            engine.metrics.get();

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
            input_level_db: level_db,
            buffer_size,
            spectrum: spectrum.to_vec(),
            tonality: tonality.to_vec(),
            waveform: waveform.to_vec(),
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

    let metrics = Metrics::from_engine(&mut engine);
    if metrics.state_version.is_multiple_of(100) {
        log::debug!(
            "Telemetry - Buffer: {} smp, Rate: {} Hz",
            metrics.buffer_size,
            engine.get_engine_state().sample_rate
        );
    }

    Ok(metrics)
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            "autostart" => {
                let autolaunch = app.autolaunch();
                if autolaunch.is_enabled().unwrap_or(false) {
                    let _ = autolaunch.disable();
                } else {
                    let _ = autolaunch.enable();
                }
            }
            _ => {}
        })
        .setup(|app| {
            // --- Auto-Load Persistence ---
            let app_handle = app.handle();
            let state = app_handle.state::<AppState>();

            // Re-use logic from load_session command for cold-start initialization
            if let Ok(config_dir) = app_handle.path().app_config_dir() {
                let config_path = config_dir.join("session.json");
                if config_path.exists() {
                    if let Ok(json) = fs::read_to_string(config_path) {
                        if let Ok(config) = serde_json::from_str::<AppConfig>(&json) {
                            if let Ok(mut engine) = state.engine.lock() {
                                engine.apply_persistence_config(config.clone());
                                if config.engine_running {
                                    let _ = engine.start(
                                        config.input_device,
                                        config.output_device,
                                        None,
                                    );
                                }
                                log::info!("Auto-loaded session from disk");
                            }
                        }
                    }
                }
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

            let is_autostart = app.autolaunch().is_enabled().unwrap_or(false);
            let autostart_i = CheckMenuItem::with_id(
                app,
                "autostart",
                "Start on Boot",
                true,
                is_autostart,
                None::<&str>,
            )?;

            // Tray Menu
            let tray_menu = Menu::with_items(app, &[&show_i, &autostart_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_engine,
            stop_engine,
            set_monitoring,
            get_metrics,
            update_config,
            get_input_devices,
            get_output_devices,
            get_engine_state,
            send_command,
            save_session,
            load_session,
            update_layout
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                log::info!("Application exiting...");

                // --- Auto-Save on Exit ---
                let state = app_handle.state::<AppState>();
                if let Ok(engine) = state.engine.lock() {
                    let config = engine.get_persistence_config();
                    if let Ok(config_dir) = app_handle.path().app_config_dir() {
                        let config_path = config_dir.join("session.json");
                        if let Ok(json) = serde_json::to_string_pretty(&config) {
                            let _ = fs::write(config_path, json);
                            log::info!("Auto-saved session on exit");
                        }
                    }
                }

                let _ = app_handle
                    .state::<AppState>()
                    .engine
                    .lock()
                    .map(|mut engine| engine.stop());
            }
        });
}
