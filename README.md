# LeylineFilter

LeylineFilter is a high-performance, modular audio processing engine built with Rust and Tauri. It provides real-time noise suppression and biometric sound gating with professional-grade stability and audio quality.

## Features

- **AI Noise Suppression**: Integration of **RNNoise**, using Gated Recurrent Units (GRU) to suppress non-stationary noise in real-time.
- **Lock-Free Engine**: Strictly real-time safe audio thread using message passing via `crossbeam-channel` and lock-free ring buffers. No Mutex contention in the audio path.
- **Smart Resampling Domains**: Engine-level optimization that resamples audio to a unified internal rate (e.g., 48kHz) only when required by the module chain, minimizing CPU usage and maximizing fidelity.
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

## Project Structure

### Backend (src-tauri/)
```text
src-tauri/
├── src/
│   ├── core/           # Core audio logic and DSP modules
│   │   ├── modules/    # Individual processing modules (Expander, RNNoise)
│   │   ├── audio.rs    # Main Lock-Free Audio Engine
│   │   └── traits.rs   # AudioModule traits and config enums
│   ├── utils/          # Utilities (Resampling, Smoothing, Logger)
│   ├── error.rs        # Serialization-ready error handling
│   ├── lib.rs          # Tauri commands and state management
└── Cargo.toml          # Rust dependencies
```

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install) and [Node.js](https://nodejs.org/).
2. Install frontend dependencies: `npm install`
3. Run in development: `npm run tauri dev`

## License

MIT License - see [LICENSE](LICENSE) for details.
