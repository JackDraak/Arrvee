# Arrvee Music Visualizer

A cross-platform music visualizer application inspired by classic WinAmp visualizers, built in Rust with real-time audio processing and FFT spectral analysis.

## Features

- **🎵 Audio File Processing**: Direct analysis of audio files (WAV, MP3, OGG support)
- **🧠 Advanced Musical Analysis**: 15+ real-time audio features for rich visualization control
  - **Spectral Features**: Brightness, rolloff, pitch confidence, zero-crossing rate
  - **Rhythm Analysis**: Beat detection, BPM estimation, onset strength
  - **Dynamic Analysis**: Volume tracking, dynamic range, spectral flux
- **🌈 Psychedelic Visual Effects**: Jeff Minter-inspired effect collection
  - **Llama Plasma Fields**: Multi-layered plasma driven by frequency bands
  - **Geometric Kaleidoscope**: BPM-controlled kaleidoscopic patterns
  - **Psychedelic Tunnel**: Classic Minter-style tunnels with spectral control
  - **Particle Swarm**: Chaos-driven particles responding to onset detection
  - **Fractal Madness**: Multi-octave noise patterns modulated by dynamics
- **🎛️ Intelligent Effect Management**: Automatic effect blending based on musical characteristics
- **⚡ Real-time FFT Analysis**: Fast Fourier Transform for frequency spectrum analysis
- **🥁 Beat Detection**: Intelligent rhythm detection with adaptive thresholds
- **📊 Frequency Band Analysis**: 5-band separation (sub-bass, bass, mid, treble, presence)
- **🚀 GPU-Accelerated Graphics**: Hardware-accelerated rendering with wgpu
- **🎨 Audio-Reactive Shaders**: WGSL shaders that respond to music in real-time
- **🔧 Developer Tools**: Real-time analysis overlay and volume controls
- **🌍 Cross-platform**: Works on Linux, Windows, and macOS
- **🧩 Modular Architecture**: Extensible design for adding new visual effects

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Audio file (WAV, MP3, or OGG format)

### Running the Visualizer

**Audio File Visualizer (Recommended):**
```bash
# Clone and navigate to the project
cd Arrvee

# Run with default sample file
cargo run --bin audio-test

# Run with your own audio file
cargo run --bin audio-test -- path/to/your/music.wav

# Run with developer overlay showing real-time analysis
cargo run --bin audio-test -- --debug sample.wav
```

The visualizer will:
- 🎵 Load and play your audio file through speakers
- 🧠 Analyze the audio data in real-time for 15+ musical characteristics
- 🌈 Display psychedelic, music-reactive graphics with 5 distinct effect types
- 🎛️ Automatically blend effects based on musical content (bass-heavy → plasma, harmonic → kaleidoscope, etc.)
- 🎨 Create trippy visuals that pulse, morph, and transform with the music

### Controls

**Playback:**
- **Space**: Pause/resume audio playback
- **Escape**: Exit the visualizer

**Developer Mode** (when using `--debug`):**
- **D**: Toggle debug overlay on/off
- **+/=**: Increase volume
- **-**: Decrease volume

**Debug Overlay Features:**
- Real-time frequency band analysis (sub-bass, bass, mid, treble, presence)
- Beat detection with strength and BPM estimation
- Advanced spectral features (brightness, pitch confidence, onset detection)
- Dynamic audio characteristics (volume, dynamic range, spectral flux)
- Effect blending weights and current dominant effect display

## Current Status

✅ **Working Components:**
- **Audio File Processing**: Direct WAV/MP3/OGG file analysis and playback ✓
- **Real-time FFT Analysis**: Fast frequency spectrum analysis during playback ✓
- **Beat Detection**: Smart rhythm detection with adaptive thresholds ✓
- **Frequency Analysis**: 5-band frequency separation (sub-bass, bass, mid, treble, presence) ✓
- **GPU Graphics Pipeline**: wgpu-based rendering with WGSL shaders ✓
- **Audio-Reactive Shaders**: Fragment shaders that respond to music in real-time ✓
- **Cross-platform**: Works on Linux, Windows, macOS ✓
- **Modular Architecture**: Clean separation of audio, graphics, UI, and effects ✓

🚧 **In Development:**
- **UI Integration**: egui interface for controls and presets (egui-wgpu compatibility)
- **Additional File Formats**: Enhanced MP3/OGG support beyond basic WAV
- **Advanced Shader Effects**: More complex visual patterns and presets

### Available Binaries

**Main Visualizer:**
```bash
# Audio file visualizer with psychedelic effects
cargo run --bin audio-test

# With debug overlay showing all analysis data
cargo run --bin audio-test -- --debug
```

**Development/Testing:**
```bash
# Graphics pipeline test with fake audio data (great for testing effects)
cargo run --bin graphics-test
```

## Architecture

The project is structured with the following modules:

### Audio Processing (`src/audio/`)
- **Playback**: Audio file loading, playback, and real-time analysis
- **FFT**: Fast Fourier Transform implementation for frequency analysis
- **Beat Detector**: Rhythm and beat detection algorithms

### Graphics Engine (`src/graphics/`)
- **Engine**: Core graphics rendering with wgpu (GPU-accelerated)
- **Shader**: WGSL shader management and pipeline creation
- **Vertex**: Vertex buffer management for geometry
- **Texture**: Texture management for visual effects

### User Interface (`src/ui/`)
- **Interface**: egui-based UI for controls and settings
- **Controls**: Playback controls, volume, preset selection

### Effects System (`src/effects/`)
- **Psychedelic Manager**: Intelligent effect blending and selection system
- **Presets**: Predefined visualizer configurations
- **Effect Collection**: 5 distinct Minter-inspired psychedelic effects

### Shaders (`shaders/`)
- **psychedelic_effects.wgsl**: Complete psychedelic effect collection featuring:
  - **Llama Plasma Fields**: Multi-layered plasma with frequency band control
  - **Geometric Kaleidoscope**: BPM-synchronized kaleidoscopic patterns
  - **Psychedelic Tunnel**: Classic Minter-style tunnel effects with spectral control
  - **Particle Swarm**: Chaos-driven particles responding to onset detection
  - **Fractal Madness**: Multi-octave fractal noise modulated by musical dynamics
- **simple.wgsl**: Basic shader for testing and reference

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
- Audio file processing and playback
- Real-time FFT analysis during playback
- Beat detection algorithm
- GPU-accelerated graphics rendering
- Audio-reactive shader system
- Cross-platform compatibility
- Frequency band analysis
- Modular architecture foundation

🚧 **In Progress:**
- UI integration with graphics pipeline
- Advanced shader effects and presets
- Additional audio format support

## Building from Source

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone <repository-url>
cd Arrvee

# Build the project
cargo build

# Run the main visualizer
cargo run --bin audio-test

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