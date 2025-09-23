# Arrvee Music Visualizer

A cross-platform music visualizer application inspired by classic WinAmp visualizers, built in Rust with real-time audio processing and FFT spectral analysis.

## Features

- **Real-time Audio Processing**: Live microphone input with FFT-based spectral analysis
- **Beat Detection**: Intelligent rhythm detection for visual synchronization
- **Frequency Band Analysis**: Bass, mid, and treble frequency separation
- **Cross-platform**: Works on Linux, Windows, and macOS
- **Modular Architecture**: Extensible design for adding new visual effects

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Audio input device (microphone)

### Running the Visualizers

**Basic Enhanced Visualizer (Recommended):**
```bash
# Clone and navigate to the project
cd Arrvee

# Run the enhanced ASCII-based visualizer
cargo run --bin basic
```

**Simple Frequency Spectrum:**
```bash
# Run the simple frequency spectrum display
cargo run --bin simple
```

The visualizers will display real-time audio analysis in your terminal, showing:
- **Basic**: Enhanced display with frequency bands, beat detection, volume meters, and sample spectrum
- **Simple**: Individual frequency bins with ASCII bar visualization and basic stats
- Real-time response to audio input from your microphone

### Controls

- **Ctrl+C**: Exit the visualizer
- Make some noise or play music near your microphone to see the visualization respond!

## Current Status

✅ **Working Components:**
- **Real-time Audio Processing**: Live microphone input with FFT analysis ✓
- **Beat Detection**: Smart rhythm detection with adaptive thresholds ✓
- **Frequency Analysis**: 5-band frequency separation (sub-bass, bass, mid, treble, presence) ✓
- **Terminal Visualizers**: Two working ASCII-based demos ✓
- **Cross-platform Audio**: Works on Linux, Windows, macOS ✓
- **Modular Architecture**: Clean separation of audio, graphics, UI, and effects ✓
- **GPU Graphics Pipeline**: wgpu-based rendering with WGSL shaders ✓
- **Shader System**: Audio-reactive fragment shaders with real-time parameters ✓

🚧 **In Development:**
- **UI Integration**: egui interface for controls and presets (egui-wgpu compatibility)
- **File Playback**: MP3/WAV/OGG audio file support

### Available Demos

**Audio-only Visualizers:**
```bash
# Enhanced ASCII visualizer with beat detection
cargo run --bin basic

# Simple frequency spectrum display
cargo run --bin simple
```

**Graphics Pipeline Tests:**
```bash
# GPU-accelerated shader rendering (no UI)
cargo run --bin minimal
```

## Architecture

The project is structured with the following modules:

### Audio Processing (`src/audio/`)
- **Processor**: Real-time audio capture and processing pipeline
- **FFT**: Fast Fourier Transform implementation for frequency analysis
- **Beat Detector**: Rhythm and beat detection algorithms
- **Playback**: Audio file loading and playback system (for future use)

### Graphics Engine (`src/graphics/`)
- **Engine**: Core graphics rendering with wgpu (GPU-accelerated)
- **Shader**: WGSL shader management and pipeline creation
- **Vertex**: Vertex buffer management for geometry
- **Texture**: Texture management for visual effects

### User Interface (`src/ui/`)
- **Interface**: egui-based UI for controls and settings
- **Controls**: Playback controls, volume, preset selection

### Effects System (`src/effects/`)
- **Presets**: Predefined visualizer configurations
- **Manager**: Dynamic effect switching and combination

### Shaders (`shaders/`)
- **visualizer.wgsl**: Main visualization shader with:
  - Plasma effects that respond to audio frequency bands
  - Spectrum bar visualization
  - Radial wave patterns
  - Beat-synchronized visual pulses

## Technical Features

### Audio Analysis
- 44.1kHz sampling rate support
- 512-point FFT for real-time frequency analysis
- Frequency band separation:
  - Sub-bass: 0-60 Hz
  - Bass: 60-250 Hz
  - Mid: 250-2000 Hz
  - Treble: 2000-8000 Hz
  - Presence: 8000+ Hz

### Beat Detection
- Adaptive threshold-based beat detection
- Bass and kick drum emphasis
- Minimum beat interval filtering to prevent false positives
- Beat strength calculation for visual intensity control

### Graphics Rendering
- Modern wgpu-based rendering pipeline
- Cross-platform GPU acceleration
- Real-time shader parameter updates based on audio analysis
- Fullscreen quad rendering for maximum visual impact

## Development Status

✅ **Working Components:**
- Real-time audio capture and FFT analysis
- Beat detection algorithm
- Simple ASCII-based visualizer
- Cross-platform audio input
- Frequency band analysis
- Modular architecture foundation

🚧 **In Progress:**
- Full graphics pipeline integration
- UI compatibility fixes for latest dependency versions
- Advanced shader effects
- File-based audio playback integration

## Building from Source

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone <repository-url>
cd Arrvee

# Build the project
cargo build

# Run the simple visualizer
cargo run --bin simple

# Run tests
cargo test
```

## Dependencies

- **Audio**: `cpal` for cross-platform audio I/O, `rustfft` for frequency analysis
- **Graphics**: `wgpu` for GPU rendering, `winit` for windowing
- **UI**: `egui` for immediate mode UI
- **Math**: `glam` for linear algebra operations
- **Utilities**: `crossbeam-channel` for threading, `anyhow` for error handling

## Future Enhancements

- [ ] Complete graphics pipeline integration
- [ ] File-based audio playback (MP3, WAV, OGG)
- [ ] Additional visual presets and effects
- [ ] Real-time parameter adjustment UI
- [ ] System audio capture (desktop audio)
- [ ] Fullscreen mode with multiple monitor support
- [ ] Preset saving and loading
- [ ] Plugin system for custom effects

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

---

*Arrvee - Bringing the classic WinAmp visualizer experience to the modern era with Rust performance and cross-platform compatibility.*