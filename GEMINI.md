# LeylineFilter: Project Mandates & Architecture

This project is a high-performance biometric audio gate built with Rust, Tauri v2, and CPAL.

## Core Architecture
- **Real-Time Safety (Lock-Free)**: The audio thread (inside `src-tauri/src/core/audio.rs`) is strictly **lock-free**. 
    - Configuration updates are sent from the UI via `crossbeam-channel`. 
    - The audio thread maintains its own private state and never waits for a Mutex held by the UI thread.
- **Engine-Level Resampling**: The engine optimizes the processing chain by choosing a unified "Internal Sample Rate."
    - Modules report requirements via the `AudioModule::requirements()` trait method.
    - If RNNoise is enabled, the engine resamples the entire input to 48kHz once, processes all modules, and resamples back to the system rate once.
- **Modular DSP**: All processing logic implements the `AudioModule` trait in `traits.rs`.
- **Async/Sync Boundary**: Tauri commands are `async` and run on the tokio runtime. The `AudioEngine` is managed as global state in `AppState`.
- **Lifecycle & Cleanup**: The `AudioEngine` implements the `Drop` trait to ensure audio streams are released when the engine is destroyed. Additionally, the application explicitly stops the engine during the `RunEvent::Exit` event in `lib.rs` to guarantee a graceful shutdown and consistent logging.

## Engineering Standards
- **Parameter Smoothing**: All audible parameter changes (threshold, ratio, bypass) must use the `ParameterSmoother` utility to prevent digital clicks and pops.
- **Sample Rate Independence**: Modules are notified of the internal sample rate via `prepare()`. They should favor the engine-level resampling but must remain functional at any rate.
- **Error Handling**: Use the custom `EngineError` enum in `error.rs` for all backend failures.
- **Metrics**: 
    - **CPU Usage**: Process-specific usage tracked via `sysinfo`.
    - **Latency**: Comprehensive pipeline latency including hardware I/O, engine buffering (10ms chunks), and module-specific lookahead (e.g., 480 samples for RNNoise).

## Security & Validation
- **CSP**: Strict Content Security Policy in `tauri.conf.json`.
- **Parameter Sanitization**: All configuration updates must be clamped/validated in the `update_config` implementation of the respective module.

## Critical Dependencies
- `cpal`: Cross-platform audio I/O.
- `nnnoiseless`: Pure-rust RNNoise implementation for voice noise suppression.
- `rubato`: High-quality resampling.
- `crossbeam-channel`: Lock-free MPMC channels for real-time safe communication.
- `ringbuf`: Lock-free SPSC buffers for audio sample transfer.
