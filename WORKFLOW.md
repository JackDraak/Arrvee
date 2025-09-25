# Arrvee Music Visualizer - Technical Workflow Documentation

## 🎯 Application Essence

**Primary Goal**: Transform musical audio streams into dynamic, synchronized visual experiences using real-time FFT analysis to drive psychedelic shader effects in 3D space.

**Core Philosophy**: Music is mathematics made audible, visualization is mathematics made visible - Arrvee bridges both worlds through intelligent audio analysis and GPU-accelerated rendering.

## 🏗️ Architecture Overview

```
Audio Input → Analysis Pipeline → Normalization → Visual Effects → 3D Rendering
     ↓              ↓                  ↓              ↓              ↓
  Multi-format   Real-time FFT    Feature Scaling  Shader Params   GPU Render
   Decoder      (15+ features)    (0.0-1.0 range)  (Effect Hooks)  (60fps sync)
```

### **Dual Operation Modes**

1. **Real-time Mode**: Live FFT analysis with <10ms latency (preferred)
2. **Pre-scan Mode**: Offline analysis with ARV binary format (97.4% compression, frame-perfect sync)

## 🎵 Audio Processing Pipeline

### **Phase 1: Audio Input & Decoding**

```rust
// Core audio playback structure
pub struct AudioPlayback {
    stream: OutputStream,                    // Audio output stream
    analyzer: Box<dyn AudioAnalyzer + Send>, // Unified GPU/CPU analyzer
    normalizer: FeatureNormalizer,           // 0.0-1.0 normalization
    audio_buffer: Vec<f32>,                  // Decoded audio samples
    sensitivity_factor: f32,                 // User-adjustable scaling (0.1-5.0)
}

// Multi-format audio loading
impl AudioPlayback {
    pub async fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // 1. Decode audio file (WAV, MP3, OGG, M4A/AAC)
        // 2. Convert to mono 44.1kHz for analysis
        // 3. Initialize unified analyzer (GPU with CPU fallback)
        // 4. Set up real-time playback sink
    }

    pub async fn get_current_audio_frame(&mut self) -> AudioFrame {
        // Real-time analysis at 60fps (735 samples per frame)
        // Returns normalized features for shader consumption
    }
}
```

### **Phase 2: Unified Analysis Architecture**

```rust
// Abstract analyzer trait for GPU/CPU transparency
#[async_trait]
pub trait AudioAnalyzer {
    async fn analyze_chunk(&mut self, audio_data: &[f32]) -> Result<RawAudioFeatures>;
    fn sample_rate(&self) -> f32;
    fn chunk_size(&self) -> usize;
    fn analyzer_type(&self) -> &'static str;  // "CPU" or "GPU"
}

// Raw audio features (natural ranges)
pub struct RawAudioFeatures {
    // Frequency bands (FFT magnitudes: ~0.00001-0.0005)
    sub_bass: f32,      // 0-60Hz
    bass: f32,          // 60-250Hz
    mid: f32,           // 250-2kHz
    treble: f32,        // 2-8kHz
    presence: f32,      // 8kHz+

    // Spectral features
    spectral_centroid: f32,    // Brightness (Hz)
    spectral_rolloff: f32,     // Spectral shape (Hz)
    spectral_flux: f32,        // Change rate

    // Temporal features
    zero_crossing_rate: f32,   // Noise vs tonal
    onset_strength: f32,       // Attack detection
    beat_strength: f32,        // Beat energy
    estimated_bpm: f32,        // Tempo (60-200)

    // Dynamic features
    volume: f32,               // RMS amplitude
    dynamic_range: f32,        // Volume variance
    pitch_confidence: f32,     // Harmonic content
}

// Automatic analyzer selection with transparent fallback
let analyzer: Box<dyn AudioAnalyzer + Send> = match NewGpuAudioAnalyzer::new().await {
    Ok(gpu) => Box::new(gpu),           // GPU acceleration
    Err(_) => Box::new(CpuAudioAnalyzer::new()?),  // CPU fallback
};
```

### **Phase 3: Feature Normalization Pipeline**

```rust
pub struct FeatureNormalizer {
    parameters: NormalizationParameters,  // Max ranges for each feature
    adaptive: bool,                       // Learning mode
}

// Normalization parameters (corrected for real FFT values)
pub struct NormalizationParameters {
    // Frequency bands (actual FFT magnitudes are tiny!)
    sub_bass_max: f32,    // 0.0001 (not 1.0!)
    bass_max: f32,        // 0.0005 (not 0.8!)
    mid_max: f32,         // 0.0002 (not 0.5!)
    treble_max: f32,      // 0.0001
    presence_max: f32,    // 0.00005

    // Dynamic ranges
    volume_max: f32,      // 0.0001 (RMS is tiny)
    beat_strength_max: f32, // 0.0005
    // ... other parameters
}

impl FeatureNormalizer {
    pub fn normalize(&mut self, raw: &RawAudioFeatures) -> NormalizedAudioFeatures {
        // Convert raw values to 0.0-1.0 range with proper scaling
        // Apply: normalized = (raw_value / max_value).clamp(0.0, 1.0)
    }
}
```

### **Phase 4: Visual Enhancement Pipeline**

```rust
// Convert normalized features to shader-ready format
fn convert_to_audio_frame_static(
    normalized: &NormalizedAudioFeatures,
    sample_rate: f32,
    sensitivity: f32
) -> AudioFrame {

    // Visual responsiveness enhancement
    let baseline_boost = 0.05;  // 5% minimum activity (prevents black screen)
    let dynamic_boost = 2.0;    // 2x multiplier for better dynamic range

    AudioFrame {
        frequency_bands: FrequencyBands {
            // Apply: baseline + normalized * sensitivity * dynamic_boost
            bass: (baseline_boost + normalized.bass * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            mid: (baseline_boost + normalized.mid * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            // ... other bands
        },
        beat_detected: normalized.beat_detected,
        beat_strength: (baseline_boost + normalized.beat_strength * sensitivity * dynamic_boost).clamp(0.0, 1.0),
        // ... other features
    }
}
```

## 🎨 Visual Effects Pipeline

### **Phase 5: Psychedelic Effect Management**

```rust
pub struct PsychedelicManager {
    effect_weights: HashMap<String, f32>,     // Current effect intensities
    target_weights: HashMap<String, f32>,     // Desired effect intensities
    intensity_scalers: HashMap<String, f32>,  // Audio-driven scaling
    manual_override: Option<String>,          // User-selected effect
    config: EffectConfig,                     // Blending parameters
}

impl PsychedelicManager {
    pub fn update(&mut self, delta_time: f32, audio_frame: &AudioFrame) {
        // 1. Analyze audio characteristics
        self.analyze_and_set_targets(audio_frame);

        // 2. Update effect transitions (smooth blending)
        self.update_transitions(delta_time);

        // 3. Scale effects by audio intensity
        self.update_intensity_scalers(audio_frame);
    }

    fn analyze_and_set_targets(&mut self, audio_frame: &AudioFrame) {
        // Intelligent effect selection based on audio characteristics:

        // Bass-heavy → Llama Plasma Fields
        if audio_frame.frequency_bands.bass > 0.7 {
            self.target_weights.insert("llama_plasma".to_string(), 1.5);
        }

        // Complex harmonics → Geometric Kaleidoscope
        if audio_frame.pitch_confidence > 0.8 && audio_frame.beat_detected {
            self.target_weights.insert("geometric_kaleidoscope".to_string(),
                1.0 + audio_frame.beat_strength);
        }

        // High spectral flux → Fractal Madness
        if audio_frame.spectral_flux > 0.6 {
            self.target_weights.insert("fractal_madness".to_string(),
                audio_frame.spectral_flux * 1.5);
        }

        // ... other effect mappings
    }
}

// Available effects (7 psychedelic modes)
enum PsychedelicEffect {
    LlamaPlasma,        // Multi-layer plasma interference
    GeometricKaleidoscope, // BPM-synchronized patterns
    PsychedelicTunnel,  // Classic Minter tunnel
    ParticleSwarm,      // Chaos-driven particles
    FractalMadness,     // Dynamic fractal noise
    SpectralBars,       // Spectrum analyzer bars
    ParametricWaves,    // Mathematical wave interference
}
```

### **Phase 6: 3D Projection & GPU Rendering**

```rust
pub struct GraphicsEngine<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    psychedelic_manager: PsychedelicManager,
    projection_mode: ProjectionMode,
    palette_index: f32,
}

// 3D projection modes
pub enum ProjectionMode {
    Auto,           // Intelligent selection
    Spheres,        // Rotating sphere grid
    Cylinder,       // Tunnel-like projection
    Torus,          // Donut-shaped surface
    Flat,           // Traditional 2D
}

impl<'a> GraphicsEngine<'a> {
    pub fn render(&mut self, audio_frame: &AudioFrame, window: &Window) -> Result<()> {
        // 1. Update psychedelic effect weights
        self.psychedelic_manager.update(delta_time, audio_frame);

        // 2. Create shader uniforms from audio data
        let uniforms = self.create_uniforms(audio_frame);

        // 3. Execute GPU render pipeline
        self.execute_render_pass(&uniforms);
    }

    fn create_uniforms(&self, audio_frame: &AudioFrame) -> PsychedelicUniforms {
        PsychedelicUniforms {
            // Direct audio mappings
            bass: audio_frame.frequency_bands.bass,
            mid: audio_frame.frequency_bands.mid,
            treble: audio_frame.frequency_bands.treble,
            beat_strength: audio_frame.beat_strength,

            // Effect weights (AI-selected)
            effect_weights: self.psychedelic_manager.get_effect_weights().clone(),
            intensity_scalers: self.psychedelic_manager.get_intensity_scalers().clone(),

            // Visual parameters
            time: self.time_elapsed,
            palette_index: self.palette_index,
            projection_mode: self.projection_mode as u32,
        }
    }
}
```

## 🚀 Pre-scan Pipeline (ARV Format)

### **When Real-time Fails**: Offline Analysis System

```rust
// ARV (Arrvee Binary) format for frame-perfect synchronization
pub struct ArvFormat;

impl ArvFormat {
    pub fn save_arv<P: AsRef<Path>>(prescan_data: &PrescanData, path: P) -> Result<()> {
        // Ultra-efficient binary format:
        // - 16 bytes per frame (vs ~600 bytes JSON)
        // - 97.4% compression (11MB → 296KB typical)
        // - Instant loading vs 2-3 seconds JSON parsing
    }
}

// Pre-scan workflow
pub struct PrescanProcessor {
    analyzer: Box<dyn AudioAnalyzer + Send>,  // Unified GPU/CPU
    normalizer: FeatureNormalizer,
}

impl PrescanProcessor {
    pub fn prescan_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<PrescanData> {
        // 1. Load entire audio file
        let audio_samples = self.load_audio_file(file_path)?;

        // 2. Analyze in 512-sample chunks (real-time simulation)
        let mut frames = Vec::new();
        for chunk in audio_samples.chunks(512) {
            let raw_features = self.analyzer.analyze_chunk(chunk).await?;
            let normalized = self.normalizer.normalize(&raw_features);
            frames.push(PrescanFrame::from(normalized));
        }

        // 3. Create synchronized data structure
        Ok(PrescanData {
            frames,
            sample_rate: self.analyzer.sample_rate(),
            total_duration: audio_samples.len() as f32 / self.analyzer.sample_rate(),
            file_info: FileInfo::from_path(&file_path),
        })
    }
}

// Synchronized playback
pub struct SynchronizedPlayback {
    prescan_data: PrescanData,
    current_frame_index: usize,
}

impl SynchronizedPlayback {
    pub fn get_synchronized_frame(&mut self, playback_time_seconds: f32) -> Option<&PrescanFrame> {
        // Frame-perfect synchronization: time → frame_index → visual data
        let frame_rate = 60.0;  // 60fps target
        let frame_index = (playback_time_seconds * frame_rate) as usize;
        self.prescan_data.frames.get(frame_index)
    }
}
```

## 🧪 Testing & Validation

### **Unit Tests for Real-time Performance**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_realtime_latency() {
        let mut playback = AudioPlayback::new().unwrap();
        playback.load_file("test_audio.wav").await.unwrap();

        let start = std::time::Instant::now();
        let frame = playback.get_current_audio_frame().await;
        let latency = start.elapsed();

        assert!(latency.as_millis() < 10, "Real-time latency must be <10ms");
        assert!(frame.frequency_bands.bass >= 0.0 && frame.frequency_bands.bass <= 1.0);
    }

    #[tokio::test]
    async fn test_gpu_cpu_consistency() {
        // Verify GPU and CPU analyzers produce identical results
        let audio_chunk = vec![0.1, 0.2, -0.1, -0.2]; // Test signal

        let gpu_result = gpu_analyzer.analyze_chunk(&audio_chunk).await.unwrap();
        let cpu_result = cpu_analyzer.analyze_chunk(&audio_chunk).await.unwrap();

        assert_eq!(gpu_result.bass, cpu_result.bass);
        assert_eq!(gpu_result.mid, cpu_result.mid);
        // ... verify all features match
    }

    #[test]
    fn test_arv_compression() {
        let prescan_data = create_test_prescan_data();

        // Save as JSON and ARV formats
        let json_size = serde_json::to_string(&prescan_data).unwrap().len();
        let arv_data = ArvFormat::save_arv(&prescan_data, "test.arv").unwrap();
        let arv_size = std::fs::metadata("test.arv").unwrap().len();

        let compression_ratio = 1.0 - (arv_size as f64 / json_size as f64);
        assert!(compression_ratio > 0.95, "ARV compression should be >95%");
    }
}
```

## 🎮 Application Entry Points

### **Real-time Visualizer**

```bash
# Primary real-time mode (preferred)
cargo run --bin audio-test sample.m4a --debug

# Key functions:
# - AudioPlayback::load_file() → decode & setup analyzer
# - AudioPlayback::get_current_audio_frame() → real-time FFT
# - GraphicsEngine::render() → 60fps synchronized visuals
```

### **Pre-scan Tool**

```bash
# Generate ARV file for perfect sync (fallback mode)
cargo run --bin prescan-tool sample.m4a -o sample.arv

# Key functions:
# - PrescanProcessor::prescan_file() → offline analysis
# - ArvFormat::save_arv() → binary compression
# - File size: ~97% smaller than JSON equivalent
```

### **Synchronized Player**

```bash
# Frame-perfect playback with pre-generated ARV
cargo run --bin synchronized-test sample.m4a --arv-file sample.arv

# Key functions:
# - SynchronizedPlayback::get_synchronized_frame() → time-indexed lookup
# - Zero analysis latency (pre-computed)
# - Perfect A/V sync guaranteed
```

## 📊 Performance Characteristics

### **Real-time Mode Benchmarks**
- **Audio Latency**: <10ms processing pipeline
- **Visual Latency**: 16ms (60fps) frame-perfect synchronization
- **CPU Usage**: 5-10% on modern hardware (optimized builds)
- **Memory Usage**: ~50MB typical runtime

### **Pre-scan Mode Benchmarks**
- **ARV Compression**: 97.4% (11.4MB → 296KB for 3-minute song)
- **Load Time**: Instant vs 2-3 seconds for JSON parsing
- **Frame Storage**: 16 bytes per frame vs ~600 bytes JSON
- **Sync Accuracy**: Perfect (pre-computed, time-indexed)

## 🎯 Key Function Signatures for LLM Reference

```rust
// === AUDIO PIPELINE ===
AudioPlayback::new() -> Result<Self>
AudioPlayback::load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()>
AudioPlayback::get_current_audio_frame(&mut self) -> AudioFrame  // 60fps real-time

// === ANALYSIS ARCHITECTURE ===
AudioAnalyzer::analyze_chunk(&mut self, audio_data: &[f32]) -> Result<RawAudioFeatures>
FeatureNormalizer::normalize(&mut self, raw: &RawAudioFeatures) -> NormalizedAudioFeatures

// === VISUAL EFFECTS ===
PsychedelicManager::update(&mut self, delta_time: f32, audio_frame: &AudioFrame)
GraphicsEngine::render(&mut self, audio_frame: &AudioFrame, window: &Window) -> Result<()>

// === PRE-SCAN PIPELINE ===
PrescanProcessor::prescan_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<PrescanData>
ArvFormat::save_arv<P: AsRef<Path>>(prescan_data: &PrescanData, path: P) -> Result<()>
SynchronizedPlayback::get_synchronized_frame(&mut self, playback_time_seconds: f32) -> Option<&PrescanFrame>
```

## 🌈 Shader Hook Integration

### **Audio → Shader Parameter Mapping**

```wgsl
// Psychedelic effects shader uniforms
struct PsychedelicUniforms {
    // Direct audio features (0.0-1.0 normalized)
    bass: f32,              // Controls plasma density, particle count
    mid: f32,               // Controls kaleidoscope complexity
    treble: f32,            // Controls high-frequency details
    beat_strength: f32,     // Controls flash intensity, transitions

    // Effect weights (AI-determined)
    effect_weights: array<f32, 7>,    // [llama_plasma, kaleidoscope, tunnel, ...]
    intensity_scalers: array<f32, 7>, // Audio-driven intensity scaling

    // Visual parameters
    time: f32,              // Animation timeline
    palette_index: f32,     // Color scheme (0-5)
    projection_mode: u32,   // 3D projection type
}

// Example effect implementation
fn llama_plasma(uv: vec2<f32>, uniforms: PsychedelicUniforms) -> vec4<f32> {
    let frequency = 10.0 + uniforms.bass * 20.0;        // Bass drives frequency
    let amplitude = 0.5 + uniforms.beat_strength * 0.5;  // Beat drives amplitude

    let plasma1 = sin(distance(uv, vec2(0.5)) * frequency + uniforms.time * 2.0);
    let plasma2 = cos(atan2(uv.y - 0.5, uv.x - 0.5) * 6.0 + uniforms.time);

    return vec4<f32>(plasma1 * plasma2 * amplitude);  // Audio-reactive plasma
}
```

---

## 🎪 Summary for LLM Understanding

**Arrvee** is a **dual-mode music visualizer** that:

1. **Primary Path**: Real-time FFT analysis (15+ features) → Normalization (0.0-1.0) → GPU shaders (60fps)
2. **Fallback Path**: Pre-scan analysis → ARV binary format → Frame-perfect synchronized playback

**Core Innovation**: Unified analysis architecture with transparent GPU/CPU fallback, ensuring identical results while maximizing performance.

**Visual Magic**: AI-driven effect selection maps musical characteristics to appropriate psychedelic effects, creating engaging, music-reactive 3D visualizations that mirror the emotional and energetic content of the audio.

The system prioritizes **real-time operation** but gracefully falls back to **pre-scan mode** when latency constraints cannot be met, ensuring optimal user experience regardless of hardware capabilities.