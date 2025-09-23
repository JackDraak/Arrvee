use rustfft::{FftPlanner, num_complex::Complex};
use super::{AudioFrame, FrequencyBands, BeatDetector};

pub struct AudioAnalyzer {
    sample_rate: f32,
    fft_size: usize,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    beat_detector: BeatDetector,
}

impl AudioAnalyzer {
    pub fn new(sample_rate: f32, fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let window = Self::hann_window(fft_size);

        Self {
            sample_rate,
            fft_size,
            fft,
            window,
            beat_detector: BeatDetector::new(sample_rate),
        }
    }

    fn hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                let phase = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
                0.5 * (1.0 - phase.cos())
            })
            .collect()
    }

    pub fn analyze(&mut self, audio_data: &[f32]) -> AudioFrame {
        let windowed_data = self.apply_window(audio_data);
        let spectrum = self.compute_fft(&windowed_data);
        let frequency_bands = self.extract_frequency_bands(&spectrum);

        // Calculate volume (RMS)
        let volume = (audio_data.iter().map(|x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();

        // Run beat detection
        let (beat_detected, beat_strength) = self.beat_detector.detect_beat(&frequency_bands);

        AudioFrame {
            sample_rate: self.sample_rate,
            spectrum: spectrum.clone(),
            time_domain: audio_data[..self.fft_size.min(audio_data.len())].to_vec(),
            frequency_bands,
            beat_detected,
            beat_strength,
            volume,
        }
    }

    fn apply_window(&self, audio_data: &[f32]) -> Vec<f32> {
        let len = self.fft_size.min(audio_data.len());
        (0..len)
            .map(|i| audio_data[i] * self.window[i])
            .collect()
    }

    fn compute_fft(&self, windowed_data: &[f32]) -> Vec<f32> {
        let mut buffer: Vec<Complex<f32>> = windowed_data
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        if buffer.len() < self.fft_size {
            buffer.resize(self.fft_size, Complex::new(0.0, 0.0));
        }

        self.fft.process(&mut buffer);

        buffer[..self.fft_size / 2]
            .iter()
            .map(|c| c.norm() * 2.0 / self.fft_size as f32)
            .collect()
    }

    fn extract_frequency_bands(&self, spectrum: &[f32]) -> FrequencyBands {
        let bin_width = self.sample_rate / self.fft_size as f32;
        let len = spectrum.len();

        let sub_bass_end = (60.0 / bin_width) as usize;
        let bass_end = (250.0 / bin_width) as usize;
        let mid_end = (2000.0 / bin_width) as usize;
        let treble_end = (8000.0 / bin_width) as usize;

        let sub_bass = Self::average_range(spectrum, 0, sub_bass_end.min(len));
        let bass = Self::average_range(spectrum, sub_bass_end, bass_end.min(len));
        let mid = Self::average_range(spectrum, bass_end, mid_end.min(len));
        let treble = Self::average_range(spectrum, mid_end, treble_end.min(len));
        let presence = Self::average_range(spectrum, treble_end, len);

        FrequencyBands {
            sub_bass,
            bass,
            mid,
            treble,
            presence,
        }
    }

    fn average_range(data: &[f32], start: usize, end: usize) -> f32 {
        if start >= end || start >= data.len() {
            return 0.0;
        }

        let end = end.min(data.len());
        let sum: f32 = data[start..end].iter().sum();
        sum / (end - start) as f32
    }
}