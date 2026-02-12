# LeylineFilter

LeylineFilter is a high-performance, modular audio processing engine built with Rust and Tauri. It provides real-time biometric sound gating and expansion with low latency and minimal CPU overhead.

## Features

- **Modular Architecture**: Easily extensible audio processing pipeline.
- **Real-time Safety**: Lock-free audio threading for glitch-free performance.
- **Low Latency**: Optimized DSP routines and efficient resampling.
- **Cross-Platform**: Built on Tauri for a lightweight, native experience on Windows, macOS, and Linux.
- **Biometric Sound Gate**: Advanced expander module designed for biometric audio filtering.

## Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/)
- **Frontend**: [React](https://reactjs.org/) with [TypeScript](https://www.typescriptlang.org/)
- **Framework**: [Tauri v2](https://tauri.app/)
- **Audio I/O**: [CPAL](https://github.com/RustAudio/cpal)
- **DSP/Resampling**: [Rubato](https://github.com/HesselM/rubato)
- **Monitoring**: [sysinfo](https://github.com/GuillaumeGomez/sysinfo)

## Project Structure

### Backend (src-tauri/)
```text
src-tauri/
├── src/
│   ├── core/           # Core audio logic and DSP modules
│   │   ├── modules/    # Individual processing modules (Expander, etc.)
│   │   ├── audio.rs    # Main Audio Engine implementation
│   │   └── traits.rs   # Common traits and configurations
│   ├── utils/          # Utility functions and logging
│   ├── error.rs        # Custom error handling
│   ├── lib.rs          # Tauri command handlers and state management
│   └── main.rs         # Application entry point
└── Cargo.toml          # Rust dependencies and configuration
```

### Frontend (src/)
```text
src/
├── components/         # Modular UI components
│   ├── Engine/         # Engine control and device selection
│   ├── Modules/        # Audio processing module UI (Expander, etc.)
│   └── Visualizer/     # Real-time status visualization
├── hooks/              # Custom React hooks (useEngine, etc.)
├── types/              # Shared TypeScript definitions
├── App.tsx             # Main layout and module orchestration
└── App.css             # Global and layout-specific styling
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) and `npm`
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1. Clone the repository.
2. Install frontend dependencies:
   ```bash
   npm install
   ```
3. Run the application in development mode:
   ```bash
   npm run tauri dev
   ```

## Configuration

The engine supports dynamic configuration of audio modules via the Tauri IPC. The `Expander` module can be tuned for threshold and ratio to filter out background noise or focus on specific biometric audio signatures.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
