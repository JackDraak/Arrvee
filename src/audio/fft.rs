use rustfft::{FftPlanner, num_complex::Complex};
use super::{AudioFrame, FrequencyBands, BeatDetector};

pub struct AudioAnalyzer {
    sample_rate: f32,
    fft_size: usize,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    beat_detector: BeatDetector,

    // For advanced analysis
    previous_spectrum: Vec<f32>,
    volume_history: Vec<f32>,
    tempo_detector: TempoDetector,
}

struct TempoDetector {
    beat_intervals: Vec<f32>,
    last_beat_time: f32,
    current_time: f32,
    estimated_bpm: f32,
}

impl TempoDetector {
    fn new() -> Self {
        Self {
            beat_intervals: Vec::new(),
            last_beat_time: 0.0,
            current_time: 0.0,
            estimated_bpm: 120.0,
        }
    }

    fn update(&mut self, beat_detected: bool, time_delta: f32) {
        self.current_time += time_delta;

        if beat_detected {
            if self.last_beat_time > 0.0 {
                let interval = self.current_time - self.last_beat_time;
                if interval > 0.3 && interval < 2.0 { // Reasonable beat interval (30-200 BPM)
                    self.beat_intervals.push(interval);
                    if self.beat_intervals.len() > 8 {
                        self.beat_intervals.remove(0);
                    }

                    // Calculate average interval and convert to BPM
                    let avg_interval: f32 = self.beat_intervals.iter().sum::<f32>() / self.beat_intervals.len() as f32;
                    self.estimated_bpm = 60.0 / avg_interval;
                }
            }
            self.last_beat_time = self.current_time;
        }
    }
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
            previous_spectrum: vec![0.0; fft_size / 2 + 1],
            volume_history: Vec::with_capacity(100),
            tempo_detector: TempoDetector::new(),
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

        // Advanced analysis features
        let spectral_centroid = self.calculate_spectral_centroid(&spectrum);
        let spectral_rolloff = self.calculate_spectral_rolloff(&spectrum);
        let zero_crossing_rate = self.calculate_zero_crossing_rate(audio_data);
        let spectral_flux = self.calculate_spectral_flux(&spectrum);
        let onset_strength = self.calculate_onset_strength(&spectrum);
        let pitch_confidence = self.calculate_pitch_confidence(&spectrum);

        // Update volume history for dynamic range calculation
        self.volume_history.push(volume);
        if self.volume_history.len() > 100 {
            self.volume_history.remove(0);
        }
        let dynamic_range = self.calculate_dynamic_range();

        // Run beat detection
        let (beat_detected, beat_strength) = self.beat_detector.detect_beat(&frequency_bands);

        // Update tempo detection (assuming ~60fps for time delta)
        self.tempo_detector.update(beat_detected, 1.0 / 60.0);

        // Store current spectrum for next frame's spectral flux calculation
        self.previous_spectrum = spectrum.clone();

        AudioFrame {
            sample_rate: self.sample_rate,
            spectrum: spectrum.clone(),
            time_domain: audio_data[..self.fft_size.min(audio_data.len())].to_vec(),
            frequency_bands,
            beat_detected,
            beat_strength,
            volume,
            spectral_centroid,
            spectral_rolloff,
            zero_crossing_rate,
            spectral_flux,
            onset_strength,
            pitch_confidence,
            estimated_bpm: self.tempo_detector.estimated_bpm,
            dynamic_range,
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

    // Advanced analysis methods
    fn calculate_spectral_centroid(&self, spectrum: &[f32]) -> f32 {
        let total_energy: f32 = spectrum.iter().sum();
        if total_energy == 0.0 {
            return 0.0;
        }

        let weighted_sum: f32 = spectrum
            .iter()
            .enumerate()
            .map(|(i, &magnitude)| i as f32 * magnitude)
            .sum();

        (weighted_sum / total_energy) * (self.sample_rate / 2.0) / spectrum.len() as f32
    }

    fn calculate_spectral_rolloff(&self, spectrum: &[f32]) -> f32 {
        let total_energy: f32 = spectrum.iter().sum();
        let rolloff_threshold = total_energy * 0.85; // 85% of energy

        let mut cumulative_energy = 0.0;
        for (i, &magnitude) in spectrum.iter().enumerate() {
            cumulative_energy += magnitude;
            if cumulative_energy >= rolloff_threshold {
                return (i as f32 / spectrum.len() as f32) * (self.sample_rate / 2.0);
            }
        }
        self.sample_rate / 2.0
    }

    fn calculate_zero_crossing_rate(&self, audio_data: &[f32]) -> f32 {
        if audio_data.len() < 2 {
            return 0.0;
        }

        let zero_crossings = audio_data
            .windows(2)
            .filter(|window| window[0] * window[1] < 0.0)
            .count();

        zero_crossings as f32 / (audio_data.len() - 1) as f32
    }

    fn calculate_spectral_flux(&self, spectrum: &[f32]) -> f32 {
        if self.previous_spectrum.len() != spectrum.len() {
            return 0.0;
        }

        spectrum
            .iter()
            .zip(self.previous_spectrum.iter())
            .map(|(&current, &previous)| (current - previous).abs())
            .sum::<f32>()
            / spectrum.len() as f32
    }

    fn calculate_onset_strength(&self, spectrum: &[f32]) -> f32 {
        let low_bands = &spectrum[1..10.min(spectrum.len())];
        let energy: f32 = low_bands.iter().sum();

        let prev_energy: f32 = if self.previous_spectrum.len() >= 10 {
            self.previous_spectrum[1..10.min(self.previous_spectrum.len())].iter().sum()
        } else {
            0.0
        };

        (energy - prev_energy).max(0.0) / low_bands.len() as f32
    }

    fn calculate_pitch_confidence(&self, spectrum: &[f32]) -> f32 {
        if spectrum.len() < 10 {
            return 0.0;
        }

        let fundamental_region = &spectrum[2..50.min(spectrum.len())];
        let high_freq_region = &spectrum[100..spectrum.len().min(200)];

        let fundamental_energy: f32 = fundamental_region.iter().sum();
        let high_freq_energy: f32 = high_freq_region.iter().sum();

        if fundamental_energy + high_freq_energy == 0.0 {
            return 0.0;
        }

        fundamental_energy / (fundamental_energy + high_freq_energy)
    }

    fn calculate_dynamic_range(&self) -> f32 {
        if self.volume_history.len() < 2 {
            return 0.0;
        }

        let max_volume = self.volume_history.iter().fold(0.0f32, |a, &b| a.max(b));
        let min_volume = self.volume_history.iter().fold(1.0f32, |a, &b| a.min(b));

        max_volume - min_volume
    }
}