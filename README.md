# LeylineFilter

LeylineFilter is a high-performance, modular audio processing engine built with Rust and Tauri. It provides real-time noise suppression and neural sound gating with professional-grade stability and audio quality.

## Features

- **Dynamic Signal Chain**: A modular processing chain that supports adding, removing, and reordering modules in real-time without interrupting the audio stream.
- **Unified Engine Bus**: A centralized `EngineCommand` system for controlling all aspects of the engine through a single thread-safe message channel.
- **State Synchronization**: Real-time state reporting that keeps the UI perfectly in sync with the internal engine state using snapshots.
- **AI Noise Suppression**: Integration of **RNNoise**, using Gated Recurrent Units (GRU) to suppress non-stationary noise in real-time.
- **Lock-Free Engine**: Strictly real-time safe audio thread using message passing via `crossbeam-channel` and lock-free ring buffers.
    - **No Allocations/Deallocations**: The audio thread is garbage-collected by the main thread to prevent malloc/free contention.
    - **Zero Contention**: Private state management ensures the audio thread never waits for a Mutex.
- **Synth-Ready DSP**: Integrated with **fundsp** for high-performance functional DSP composition, enabling complex synth voices and custom processing chains.
- **De-clicked Parameters**: All parameter changes (thresholds, ratios, bypass toggles) use exponential smoothing and cross-fading to prevent digital clicks and pops.
- **Sample Rate Independence**: Full support for arbitrary hardware sample rates with automatic engine adaptation and high-quality resampling via `rubato`.
- **Cross-Platform**: Built on Tauri v2 for a lightweight experience on Windows, macOS, and Linux.

## Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/)
- **Frontend**: [React](https://reactjs.org/) with [TypeScript](https://www.typescriptlang.org/)
- **Framework**: [Tauri v2](https://tauri.app/)
- **Audio I/O**: [CPAL](https://github.com/RustAudio/cpal)
- **Noise Suppression**: [nnnoiseless](https://github.com/shrit/nnnoiseless) (RNNoise)
- **DSP/Resampling**: [Rubato](https://github.com/HesselM/rubato)
- **Testing**: [Vitest](https://vitest.dev/) (Frontend) & `cargo test` (Backend)

## Project Structure

### Backend (src-tauri/)
```text
src-tauri/
├── src/
│   ├── core/           # Core audio logic and DSP modules
│   │   ├── modules/    # Registry-based processing modules
│   │   │   ├── dynamics/  # Expander, Compressor, Limiter
│   │   │   ├── voice/     # RNNoise, VAD
│   │   │   ├── filter/    # HPF, LPF, EQ
│   │   │   ├── fx/        # Reverb, Delay
│   │   │   ├── synth/     # Oscillators, Envelopes
│   │   │   └── utility/   # Gain, AGC
│   │   ├── audio.rs    # Main Lock-Free Audio Engine
│   │   ├── chain.rs    # Dynamic Signal Chain management
│   │   ├── traits.rs   # AudioModule traits and config enums
│   │   └── visualizer.rs # Real-time FFT analysis
│   ├── utils/          # Utilities (Resampling, Smoothing, Logger)
│   ├── error.rs        # Serialization-ready error handling
│   ├── lib.rs          # Tauri commands and state management
└── Cargo.toml          # Rust dependencies
```

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install) and [Node.js](https://nodejs.org/).
2. Install frontend dependencies: `npm install`
3. Run in development: `npm run tauri dev`
4. Run tests: `npm test` or `cd src-tauri && cargo test`

## License

MIT License - see [LICENSE](LICENSE) for details.
