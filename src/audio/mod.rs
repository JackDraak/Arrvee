pub mod processor;
pub mod fft;
pub mod beat_detector;
pub mod playback;

pub use processor::AudioProcessor;
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