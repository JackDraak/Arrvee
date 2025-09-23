pub mod fft;
pub mod beat_detector;
pub mod playback;

pub use fft::AudioAnalyzer;
pub use beat_detector::BeatDetector;
pub use playback::AudioPlayback;

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub sample_rate: f32,
    pub spectrum: Vec<f32>,
    pub time_domain: Vec<f32>,
    pub frequency_bands: FrequencyBands,
    pub beat_detected: bool,
    pub beat_strength: f32,
    pub volume: f32,

    // Advanced analysis features
    pub spectral_centroid: f32,    // "Brightness" - where most energy is concentrated
    pub spectral_rolloff: f32,     // High frequency content
    pub zero_crossing_rate: f32,   // Noisiness vs tonality
    pub spectral_flux: f32,        // Rate of change in spectrum
    pub onset_strength: f32,       // Note attack detection
    pub pitch_confidence: f32,     // How tonal vs noisy
    pub estimated_bpm: f32,        // Current tempo estimate
    pub dynamic_range: f32,        // Loudness variation
}

#[derive(Debug, Clone)]
pub struct FrequencyBands {
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub sub_bass: f32,
    pub presence: f32,
}

impl Default for AudioFrame {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            spectrum: vec![0.0; 512],
            time_domain: vec![0.0; 1024],
            frequency_bands: FrequencyBands::default(),
            beat_detected: false,
            beat_strength: 0.0,
            volume: 0.0,
            spectral_centroid: 0.0,
            spectral_rolloff: 0.0,
            zero_crossing_rate: 0.0,
            spectral_flux: 0.0,
            onset_strength: 0.0,
            pitch_confidence: 0.0,
            estimated_bpm: 120.0,
            dynamic_range: 0.0,
        }
    }
}

impl Default for FrequencyBands {
    fn default() -> Self {
        Self {
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            sub_bass: 0.0,
            presence: 0.0,
        }
    }
}