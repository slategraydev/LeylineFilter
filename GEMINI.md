# LeylineFilter: Project Mandates & Architecture

This project is a high-performance biometric audio gate built with Rust, Tauri v2, and CPAL.

## Core Architecture
- **Real-Time Safety**: The audio thread (inside `src-tauri/src/core/audio.rs`) **must never block**. Do not use standard `Mutex` or `RwLock` for data transfer; use the implemented `ringbuf` (lock-free) or atomic operations.
- **Modular DSP**: All processing logic must implement the `AudioModule` trait in `traits.rs`.
- **Async/Sync Boundary**: Tauri commands are `async` and run on the tokio runtime. The `AudioEngine` state is wrapped in a `Mutex` in `AppState` for safe access from these commands.

## Engineering Standards
- **Error Handling**: Use the custom `EngineError` enum in `error.rs` for all backend failures. This ensures proper serialization for the Tauri frontend.
- **Sample Rate Independence**: All DSP modules must implement the `prepare` method from the `AudioModule` trait. This method is used to notify modules of the pipeline's sample rate, allowing them to pre-calculate coefficients and initialize internal resamplers if their internal processing requires a different rate (e.g., AI models).
- **Metrics**: 
    - **CPU Usage**: Tracks the application process usage using `sysinfo`, throttled to 200ms refreshes with EMA smoothing (alpha=0.1) for soft UI transitions.
    - **Latency**: Reports full pipeline latency including measured hardware I/O (via `cpal` timestamps), processing time, a dynamic 10ms chunking delay, and ring buffer occupancy. Values are EMA-smoothed and reported as whole milliseconds.

## Security & Validation
- **CSP**: The `tauri.conf.json` contains a strict Content Security Policy. Any new external resources must be explicitly whitelisted there.
- **Parameter Sanitization**: All configuration updates from the frontend must be clamped/validated in the `update_config` implementation of the respective module.

## Critical Dependencies
- `cpal`: Cross-platform audio I/O.
- `rubato`: Used for high-quality resampling when hardware input and output sample rates differ.
- `ringbuf`: Lock-free SPSC (Single Producer Single Consumer) buffers for audio samples.
- `sysinfo`: Used for cross-platform process CPU usage monitoring.
