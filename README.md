# Arrvee Music Visualizer

A revolutionary music visualizer built in Rust featuring real-time audio processing, synchronized playback, and psychedelic Jeff Minter-inspired visual effects. Combines modern GPU acceleration with classic WinAmp-style visualization.

## 🌟 Key Features

### 🎵 Advanced Audio Processing
- **Multi-format Support**: WAV, MP3, OGG, M4A/AAC with high-quality decoding
- **Real-time Analysis**: 15+ audio features extracted in real-time
- **Synchronized Playback**: Frame-perfect synchronization with pre-computed analysis
- **ARV Format**: Proprietary binary format achieving 97%+ compression for instant loading

### 🌈 Psychedelic Visual Effects (Jeff Minter Inspired)
- **Llama Plasma Fields**: Multi-layered plasma driven by frequency bands
- **Geometric Kaleidoscope**: BPM-controlled kaleidoscopic patterns with mirroring
- **Psychedelic Tunnel**: Classic Minter-style tunnels with spectral control
- **Particle Swarm**: Chaos-driven particles responding to onset detection
- **Fractal Madness**: Multi-octave noise patterns modulated by dynamics
- **Spectralizer Bars**: Enhanced spectrum analyzer visualization
- **Parametric Waves**: Mathematical wave interference patterns

### 🌀 3D Surface Projection System
- **Multiple Spheres**: Effects projected onto rotating sphere grids
- **Cylindrical**: Tunnel-like wraparound projections
- **Torus**: Donut-shaped projection surfaces
- **Intelligent Auto-Selection**: Chooses optimal projection based on music characteristics

### ⚡ Performance & Technology
- **GPU-Accelerated**: Hardware-accelerated rendering with wgpu
- **Real-time FFT**: Fast Fourier Transform for frequency spectrum analysis
- **Beat Detection**: Intelligent rhythm detection with adaptive thresholds
- **Frequency Bands**: 5-band separation (sub-bass, bass, mid, treble, presence)
- **Cross-platform**: Linux, Windows, macOS support

## 🚀 Quick Start

### Audio File Visualization (Recommended)
```bash
# Clone and build
git clone https://github.com/JackDraak/Arrvee
cd Arrvee

# Run with sample file
cargo run --bin audio-test

# Run with your music file
cargo run --bin audio-test -- path/to/your/music.m4a

# Debug mode with analysis overlay
cargo run --bin audio-test -- --debug sample.wav
```

### Pre-scan for Perfect Synchronization
```bash
# Generate compressed analysis data
cargo run --bin prescan-tool sample.m4a -o sample.arv

# Run with synchronized playback (zero latency)
cargo run --bin synchronized-test sample.m4a --arv-file sample.arv --debug
```

## 🎮 Controls

### Playback & Navigation
- **Space**: Pause/resume audio playback
- **Escape**: Exit visualizer
- **+/-**: Volume control
- **S**: Show synchronization info

### Visual Effects (1-7 Keys)
- **1**: Llama Plasma Fields (frequency-driven plasma)
- **2**: Geometric Kaleidoscope (BPM-synchronized patterns)
- **3**: Psychedelic Tunnel (classic Minter tunnel)
- **4**: Particle Swarm (chaos-driven particles)
- **5**: Fractal Madness (dynamic fractal noise)
- **6**: Spectralizer Bars (spectrum analyzer)
- **7**: Parametric Waves (mathematical interference)
- **0**: Auto-Blend Mode (intelligent effect selection)

### 3D Projection Control (Q-W-E-R-T)
- **Q**: Auto Projection (intelligent selection)
- **W**: Multiple Spheres (rotating sphere grid)
- **E**: Cylinder (tunnel-like projection)
- **R**: Torus (donut-shaped surface)
- **T**: Flat (traditional 2D)

### Visual Customization
- **P**: Cycle Color Palettes (Rainbow, Neon Cyber, Warm Sunset, Deep Ocean, Purple Haze, Electric Green)
- **[/]**: Adjust smoothing/sensitivity (0.1-2.0 range)
- **D**: Toggle debug overlay (developer mode)

## 🛠️ Available Tools

### Main Visualizers
```bash
# Real-time audio file visualizer
cargo run --bin audio-test [audio_file] [--debug]

# Synchronized visualization with pre-computed data
cargo run --bin synchronized-test <audio_file> --arv-file <arv_file> [--debug]
```

### Analysis & Development Tools
```bash
# Pre-scan audio for synchronized playback
cargo run --bin prescan-tool <input_file> [-o output_file] [--format arv|json]

# Audio analysis tool for tuning parameters
cargo run --bin audio-analyzer <audio_file> [-o output_file] [--frame-log]

# Graphics pipeline test
cargo run --bin graphics-test

# GPU audio processing test
cargo run --bin gpu-audio-test
```

## 📊 Technical Architecture

### Audio Processing (`src/audio/`)
- **Real-time Analysis**: FFT-based frequency analysis with 15+ audio features
- **Beat Detection**: Adaptive threshold algorithm with BPM estimation
- **Synchronized Playback**: Frame-perfect timing using pre-computed analysis
- **ARV Format**: Proprietary binary format (97% smaller than JSON)
- **Multi-format Support**: WAV, MP3, OGG, M4A/AAC decoding

### Graphics Engine (`src/graphics/`)
- **wgpu Rendering**: Modern GPU-accelerated graphics pipeline
- **WGSL Shaders**: Audio-reactive fragment shaders with real-time parameters
- **3D Projections**: Sphere, cylinder, torus projection mathematics
- **Effect Blending**: Intelligent effect selection based on musical characteristics

### Effects System (`src/effects/`)
- **Psychedelic Manager**: AI-driven effect selection and blending
- **Jeff Minter Inspiration**: Classic demoscene and Llamasoft aesthetics
- **Real-time Parameters**: All effects respond to live audio analysis

### Synchronized System
- **Pre-computation**: Offline analysis generates compressed data files
- **Frame-perfect Sync**: Zero-latency synchronized playback
- **Compression**: 97%+ size reduction (11MB → 300KB typical)

## 🎯 Audio Features Analyzed

### Frequency Analysis
- **5-Band Separation**: Sub-bass (0-60Hz), Bass (60-250Hz), Mid (250-2kHz), Treble (2-8kHz), Presence (8kHz+)
- **Spectral Features**: Centroid, rolloff, flux for brightness and texture analysis
- **Harmonic Analysis**: Pitch confidence and zero-crossing rate

### Rhythm & Dynamics
- **Beat Detection**: Onset detection with adaptive thresholds
- **BPM Estimation**: Real-time tempo analysis with range validation
- **Dynamic Range**: Volume variance and energy profiling
- **Complexity Scoring**: Musical complexity for intelligent effect selection

### Visual Mapping
- **Bass → Plasma**: Heavy bass drives plasma field intensity
- **Harmonics → Kaleidoscope**: Harmonic content controls geometric patterns
- **Beats → Particles**: Beat detection triggers particle bursts
- **Complexity → Fractals**: Musical complexity modulates fractal noise

## 🔧 Building from Source

### Prerequisites
- Rust (latest stable)
- Audio libraries (automatically managed by Cargo)

### Installation
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/JackDraak/Arrvee
cd Arrvee
cargo build --release

# Run with optimized performance
cargo run --release --bin audio-test
```

### Dependencies
- **Audio**: `cpal`, `rodio`, `symphonia`, `rustfft`, `hound`
- **Graphics**: `wgpu`, `winit`, `egui`, `glam`
- **Threading**: `tokio`, `crossbeam-channel`, `futures-intrusive`
- **Utilities**: `anyhow`, `serde`, `clap`, `log`

## 📈 Performance Benchmarks

### ARV Format Compression
- **Sample 3-minute song**: 11.4MB JSON → 296KB ARV (97.4% compression)
- **Load time**: Instant vs 2-3 seconds for JSON parsing
- **Memory usage**: 15MB → 500KB in-memory representation

### Real-time Performance
- **Audio latency**: <10ms audio processing pipeline
- **Visual latency**: 16ms (60fps) frame-perfect synchronization
- **CPU usage**: 5-10% on modern hardware
- **GPU usage**: Efficient fragment shader rendering

## 🎨 Shader Effects Details

### Llama Plasma Fields
- Multi-layered interference patterns
- Frequency band amplitude mapping
- Dynamic color temperature based on spectral content
- Mathematical wave equations: `sin(radius * frequency - time * speed)`

### Geometric Kaleidoscope
- BPM-synchronized rotation and mirroring
- Harmonic content drives geometric complexity
- 6-fold symmetry with audio-reactive distortion
- Color cycling based on pitch confidence

### Parametric Waves
- Mathematical interference patterns using trigonometric functions
- Multiple wave sources with audio-reactive parameters
- Amplitude and frequency modulation from frequency bands
- Complex mathematical patterns: `wave1 * wave2 + interference`

## 🚧 Development Status

### ✅ Complete Features
- Multi-format audio file processing and playback
- Real-time FFT analysis with 15+ audio features
- GPU-accelerated psychedelic visual effects (7 unique effects)
- 3D surface projection system (4 projection modes)
- Synchronized playback with ARV binary format
- Cross-platform compatibility (Linux, Windows, macOS)
- Intelligent effect selection and blending
- Developer tools and analysis overlays

### 🔄 In Progress
- Additional visual effects and presets
- Advanced UI controls and settings persistence
- Plugin system for custom effects
- Performance optimization and profiling

## 🎵 Supported Formats

- **WAV**: Uncompressed audio (best quality)
- **MP3**: MPEG audio layer 3
- **OGG**: Ogg Vorbis compressed audio
- **M4A/AAC**: Advanced Audio Coding (iTunes format)
- **Sample rates**: 44.1kHz, 48kHz, 96kHz
- **Bit depths**: 16-bit, 24-bit, 32-bit

## 🌟 Future Enhancements

- [ ] Live audio input (microphone/line-in)
- [ ] System audio capture (desktop audio visualization)
- [ ] VR/AR support for immersive experiences
- [ ] Network streaming and remote control
- [ ] Machine learning-driven effect selection
- [ ] Preset sharing and community features
- [ ] Multi-monitor fullscreen support
- [ ] MIDI controller integration

## 🤝 Contributing

Contributions welcome! Areas of interest:
- New visual effects and shaders
- Audio format support expansion
- Performance optimizations
- Cross-platform compatibility
- Documentation and examples

## 📄 License

MIT License - See LICENSE file for details.

---

*Arrvee - Real-time synchronized music visualization with authentic demoscene aesthetics and modern performance.*

**🎵 "Music is the universal language, visualization is its visual poetry" 🎵**