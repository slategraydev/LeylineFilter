# LeylineFilter: Project Mandates & Architecture

This project is a high-performance biometric audio gate built with Rust, Tauri v2, and CPAL.

## Core Architecture
- **Real-Time Safety**: The audio thread (inside `src-tauri/src/core/audio.rs`) **must never block**. Do not use standard `Mutex` or `RwLock` for data transfer; use the implemented `ringbuf` (lock-free) or atomic operations.
- **Modular DSP**: All processing logic must implement the `AudioModule` trait in `traits.rs`.
- **Async/Sync Boundary**: Tauri commands are `async` and run on the tokio runtime. The `AudioEngine` state is wrapped in a `Mutex` in `AppState` for safe access from these commands.

## Engineering Standards
- **Error Handling**: Use the custom `EngineError` enum in `error.rs` for all backend failures. This ensures proper serialization for the Tauri frontend.
- **Sample Rate Independence**: All DSP modules (like the Expander) must calculate coefficients based on the current `sample_rate`. Never assume 44.1kHz or 48kHz.
- **Metrics**: Engine metrics (latency, CPU) are updated via `AtomicU32`. Read them using `f32::from_bits`.

## Security & Validation
- **CSP**: The `tauri.conf.json` contains a strict Content Security Policy. Any new external resources must be explicitly whitelisted there.
- **Parameter Sanitization**: All configuration updates from the frontend must be clamped/validated in the `update_config` implementation of the respective module.

## Critical Dependencies
- `cpal`: Cross-platform audio I/O.
- `rubato`: Used for high-quality resampling when the hardware sample rate differs from the internal 48kHz processing rate.
- `ringbuf`: Lock-free SPSC (Single Producer Single Consumer) buffers for audio samples.
