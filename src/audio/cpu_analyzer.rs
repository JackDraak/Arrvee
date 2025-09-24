use super::{AudioAnalyzer, RawAudioFeatures};
use super::fft::AudioAnalyzer as CpuAnalyzer;
use anyhow::Result;
use async_trait::async_trait;

/// CPU-based audio analyzer that implements the common AudioAnalyzer trait
/// This wraps the existing CPU FFT analyzer and outputs raw features
pub struct CpuAudioAnalyzer {
    inner: CpuAnalyzer,
    sample_rate: f32,
    chunk_size: usize,
}

impl CpuAudioAnalyzer {
    /// Create a new CPU-based audio analyzer
    pub fn new(sample_rate: f32, chunk_size: usize) -> Result<Self> {
        let inner = CpuAnalyzer::new(sample_rate, chunk_size);
        Ok(Self {
            inner,
            sample_rate,
            chunk_size,
        })
    }
}

#[async_trait]
impl AudioAnalyzer for CpuAudioAnalyzer {
    async fn analyze_chunk(&mut self, audio_data: &[f32]) -> Result<RawAudioFeatures> {
        // Use the existing CPU analyzer but extract raw features before normalization
        let raw_features = self.extract_raw_features(audio_data);
        Ok(raw_features)
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn analyzer_type(&self) -> &'static str {
        "CPU"
    }
}

impl CpuAudioAnalyzer {
    /// Extract raw features before normalization by replicating CPU analyzer logic
    fn extract_raw_features(&mut self, audio_data: &[f32]) -> RawAudioFeatures {
        // Apply the same windowing and FFT as the inner analyzer
        let windowed_data = self.apply_window(audio_data);
        let spectrum = self.compute_fft(&windowed_data);
        let raw_frequency_bands = self.extract_raw_frequency_bands(&spectrum);

        // Calculate volume (RMS) - raw value
        let volume = (audio_data.iter().map(|x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();

        // Advanced analysis features - raw values
        let spectral_centroid = self.calculate_spectral_centroid(&spectrum);
        let spectral_rolloff = self.calculate_spectral_rolloff(&spectrum);
        let zero_crossing_rate = self.calculate_zero_crossing_rate(audio_data);
        let spectral_flux = self.calculate_spectral_flux(&spectrum);
        let onset_strength = self.calculate_onset_strength(&spectrum);
        let pitch_confidence = self.calculate_pitch_confidence(&spectrum);

        // Update volume history for dynamic range calculation
        let dynamic_range = self.calculate_dynamic_range(volume);

        // Run beat detection on raw frequency bands
        let beat_strength = self.calculate_beat_strength(&raw_frequency_bands);

        // Update BPM estimation
        let estimated_bpm = self.update_bpm_estimation(beat_strength > 0.3);

        RawAudioFeatures {
            sub_bass: raw_frequency_bands.sub_bass,
            bass: raw_frequency_bands.bass,
            mid: raw_frequency_bands.mid,
            treble: raw_frequency_bands.treble,
            presence: raw_frequency_bands.presence,
            spectral_centroid,
            spectral_rolloff,
            spectral_flux,
            zero_crossing_rate,
            onset_strength,
            beat_strength,
            estimated_bpm,
            volume,
            dynamic_range,
            pitch_confidence,
        }
    }

    // Helper methods that replicate the CPU analyzer's internal logic

    fn apply_window(&self, audio_data: &[f32]) -> Vec<f32> {
        let len = self.chunk_size.min(audio_data.len());
        // Hann window
        (0..len)
            .map(|i| {
                let window_val = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (len - 1) as f32).cos());
                audio_data[i] * window_val
            })
            .collect()
    }

    fn compute_fft(&self, windowed_data: &[f32]) -> Vec<f32> {
        use rustfft::{FftPlanner, num_complex::Complex};

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.chunk_size);

        let mut buffer: Vec<Complex<f32>> = windowed_data
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Pad with zeros if needed
        buffer.resize(self.chunk_size, Complex::new(0.0, 0.0));

        fft.process(&mut buffer);

        // Convert to magnitudes
        buffer.iter()
            .take(self.chunk_size / 2)
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect()
    }

    fn extract_raw_frequency_bands(&self, spectrum: &[f32]) -> RawFrequencyBands {
        let sample_rate = self.sample_rate;
        let fft_size = self.chunk_size;

        let mut bass = 0.0;
        let mut mid = 0.0;
        let mut treble = 0.0;
        let mut sub_bass = 0.0;
        let mut presence = 0.0;

        let mut bass_count = 0;
        let mut mid_count = 0;
        let mut treble_count = 0;
        let mut sub_bass_count = 0;
        let mut presence_count = 0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            let frequency = (i as f32 * sample_rate) / fft_size as f32;

            if frequency <= 60.0 {
                sub_bass += magnitude;
                sub_bass_count += 1;
            } else if frequency <= 250.0 {
                bass += magnitude;
                bass_count += 1;
            } else if frequency <= 4000.0 {
                mid += magnitude;
                mid_count += 1;
            } else if frequency <= 12000.0 {
                treble += magnitude;
                treble_count += 1;
            } else if frequency <= 20000.0 {
                presence += magnitude;
                presence_count += 1;
            }
        }

        // Average by count (raw values, not normalized)
        RawFrequencyBands {
            sub_bass: if sub_bass_count > 0 { sub_bass / sub_bass_count as f32 } else { 0.0 },
            bass: if bass_count > 0 { bass / bass_count as f32 } else { 0.0 },
            mid: if mid_count > 0 { mid / mid_count as f32 } else { 0.0 },
            treble: if treble_count > 0 { treble / treble_count as f32 } else { 0.0 },
            presence: if presence_count > 0 { presence / presence_count as f32 } else { 0.0 },
        }
    }

    fn calculate_spectral_centroid(&self, spectrum: &[f32]) -> f32 {
        let mut weighted_sum = 0.0;
        let mut magnitude_sum = 0.0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            let frequency = (i as f32 * self.sample_rate) / self.chunk_size as f32;
            weighted_sum += frequency * magnitude;
            magnitude_sum += magnitude;
        }

        if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        }
    }

    fn calculate_spectral_rolloff(&self, spectrum: &[f32]) -> f32 {
        let total_energy: f32 = spectrum.iter().map(|&x| x * x).sum();
        let threshold = total_energy * 0.85;
        let mut cumulative_energy = 0.0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            cumulative_energy += magnitude * magnitude;
            if cumulative_energy >= threshold {
                return (i as f32 * self.sample_rate) / self.chunk_size as f32;
            }
        }

        self.sample_rate / 2.0 // Nyquist frequency
    }

    fn calculate_zero_crossing_rate(&self, audio_data: &[f32]) -> f32 {
        let mut crossings = 0;
        for i in 1..audio_data.len() {
            if (audio_data[i] >= 0.0) != (audio_data[i-1] >= 0.0) {
                crossings += 1;
            }
        }
        crossings as f32 / audio_data.len() as f32
    }

    fn calculate_spectral_flux(&self, spectrum: &[f32]) -> f32 {
        // Simple spectral variance as a proxy for flux
        let mean: f32 = spectrum.iter().sum::<f32>() / spectrum.len() as f32;
        let variance: f32 = spectrum.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / spectrum.len() as f32;
        variance.sqrt()
    }

    fn calculate_onset_strength(&self, spectrum: &[f32]) -> f32 {
        // Use energy in lower frequencies (attack frequencies)
        spectrum.iter()
            .take(spectrum.len() / 4)
            .map(|&x| x * x)
            .sum::<f32>()
    }

    fn calculate_pitch_confidence(&self, spectrum: &[f32]) -> f32 {
        // Simple harmonic detection - ratio of harmonic peaks
        let mut harmonic_energy = 0.0;
        let total_energy: f32 = spectrum.iter().map(|&x| x * x).sum();

        // Look for peaks that have harmonics
        for i in 1..spectrum.len()/8 {
            let fundamental_energy = spectrum[i] * spectrum[i];
            if i * 2 < spectrum.len() {
                let harmonic_energy_val = spectrum[i * 2] * spectrum[i * 2];
                if harmonic_energy_val > fundamental_energy * 0.3 {
                    harmonic_energy += fundamental_energy;
                }
            }
        }

        if total_energy > 0.0 {
            (harmonic_energy / total_energy).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn calculate_dynamic_range(&self, current_volume: f32) -> f32 {
        // Simple range calculation based on recent volume
        // In a real implementation, you'd maintain a volume history
        current_volume.clamp(0.0, 1.0)
    }

    fn calculate_beat_strength(&self, bands: &RawFrequencyBands) -> f32 {
        // Simple beat strength based on bass energy
        bands.bass + bands.sub_bass * 0.5
    }

    fn update_bpm_estimation(&self, beat_detected: bool) -> f32 {
        // Simplified BPM estimation - return a reasonable default
        120.0 // In real implementation, track beat intervals
    }
}

#[derive(Debug)]
struct RawFrequencyBands {
    sub_bass: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    presence: f32,
}