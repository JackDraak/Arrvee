use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use log::info;
use super::{fft::AudioAnalyzer, AudioFrame, FrequencyBands};

/// Pre-processed audio data for real-time synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescanData {
    /// File metadata
    pub file_info: FileInfo,

    /// Frame-by-frame audio analysis data for perfect sync
    pub frames: Vec<PrescanFrame>,

    /// Statistics for normalization and calibration
    pub statistics: AnalysisStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub duration_seconds: f32,
    pub sample_rate: f32,
    pub total_samples: usize,
    pub frame_rate: f32,
    pub chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescanFrame {
    /// Timestamp in seconds
    pub timestamp: f32,

    /// Normalized frequency bands (0.0-1.0)
    pub frequency_bands: FrequencyBands,

    /// Beat detection
    pub beat_detected: bool,
    pub beat_strength: f32,
    pub estimated_bpm: f32,

    /// Spectral features (normalized 0.0-1.0)
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub pitch_confidence: f32,

    /// Temporal features (normalized 0.0-1.0)
    pub zero_crossing_rate: f32,
    pub spectral_flux: f32,
    pub onset_strength: f32,
    pub dynamic_range: f32,

    /// Volume (RMS)
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatistics {
    /// Peak values observed for dynamic range optimization
    pub peak_bass: f32,
    pub peak_mid: f32,
    pub peak_treble: f32,
    pub peak_presence: f32,
    pub peak_volume: f32,
    pub peak_spectral_flux: f32,
    pub peak_onset: f32,

    /// Beat analysis stats
    pub total_beats: u32,
    pub average_bpm: f32,
    pub bpm_range: (f32, f32),

    /// Content classification
    pub dominant_frequency_range: String,
    pub energy_profile: String, // "Low", "Medium", "High", "Dynamic"
    pub complexity_score: f32,
}

impl From<&AudioFrame> for PrescanFrame {
    fn from(frame: &AudioFrame) -> Self {
        Self {
            timestamp: 0.0, // Will be set during processing
            frequency_bands: frame.frequency_bands.clone(),
            beat_detected: frame.beat_detected,
            beat_strength: frame.beat_strength,
            estimated_bpm: frame.estimated_bpm,
            spectral_centroid: frame.spectral_centroid,
            spectral_rolloff: frame.spectral_rolloff,
            pitch_confidence: frame.pitch_confidence,
            zero_crossing_rate: frame.zero_crossing_rate,
            spectral_flux: frame.spectral_flux,
            onset_strength: frame.onset_strength,
            dynamic_range: frame.dynamic_range,
            volume: frame.volume,
        }
    }
}

pub struct PrescanProcessor {
    chunk_size: usize,
    sample_rate: f32,
}

impl PrescanProcessor {
    pub fn new(sample_rate: f32, chunk_size: usize) -> Self {
        Self {
            chunk_size,
            sample_rate,
        }
    }

    /// Pre-scan an audio file and generate synchronization data
    pub fn prescan_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<PrescanData> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();
        info!("Pre-scanning audio file: {}", path_str);

        // Load audio file directly using the same method as AudioPlayback
        let audio_buffer = self.load_audio_file(&file_path)?;
        let total_samples = audio_buffer.len();
        let duration_seconds = total_samples as f32 / self.sample_rate;
        let frame_rate = self.sample_rate / self.chunk_size as f32;

        info!("Loaded {} samples ({:.2}s) for pre-scanning", total_samples, duration_seconds);

        // Create analyzer with normalization
        let mut analyzer = AudioAnalyzer::new(self.sample_rate, self.chunk_size);

        // Process entire file chunk by chunk
        let mut frames = Vec::new();
        let mut statistics = AnalysisStatistics::default();
        let mut sample_pos = 0;
        let mut beat_count = 0u32;
        let mut bpm_values = Vec::new();

        while sample_pos + self.chunk_size <= total_samples {
            let chunk = &audio_buffer[sample_pos..sample_pos + self.chunk_size];
            let audio_frame = analyzer.analyze(chunk);
            let timestamp = sample_pos as f32 / self.sample_rate;

            // Create prescan frame
            let mut prescan_frame = PrescanFrame::from(&audio_frame);
            prescan_frame.timestamp = timestamp;

            // Update statistics
            self.update_statistics(&mut statistics, &audio_frame, &mut beat_count, &mut bpm_values);

            frames.push(prescan_frame);
            sample_pos += self.chunk_size;

            if frames.len() % 1000 == 0 {
                info!("Pre-scanned {} frames ({:.1}s of {:.1}s)",
                      frames.len(), timestamp, duration_seconds);
            }
        }

        // Finalize statistics
        statistics.total_beats = beat_count;
        if !bpm_values.is_empty() {
            statistics.average_bpm = bpm_values.iter().sum::<f32>() / bpm_values.len() as f32;
            statistics.bpm_range = (
                bpm_values.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                bpm_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
            );
        }

        // Classify content
        self.classify_content(&mut statistics, &frames);

        info!("Pre-scan complete: {} frames, {} beats, {:.1} BPM average",
              frames.len(), beat_count, statistics.average_bpm);

        Ok(PrescanData {
            file_info: FileInfo {
                filename: path_str,
                duration_seconds,
                sample_rate: self.sample_rate,
                total_samples,
                frame_rate,
                chunk_size: self.chunk_size,
            },
            frames,
            statistics,
        })
    }

    /// Save prescan data to JSON file
    pub fn save_prescan_data<P: AsRef<Path>>(prescan_data: &PrescanData, output_path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(prescan_data)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }

    /// Load prescan data from JSON file
    pub fn load_prescan_data<P: AsRef<Path>>(input_path: P) -> Result<PrescanData> {
        let json = std::fs::read_to_string(input_path)?;
        let prescan_data: PrescanData = serde_json::from_str(&json)?;
        Ok(prescan_data)
    }

    // Private helper methods

    fn load_audio_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<f32>> {
        use rodio::{Decoder, Source};
        use std::fs::File;
        use std::io::BufReader;

        let file = BufReader::new(File::open(file_path)?);
        let source = Decoder::new(file)?;

        let channels = source.channels();
        let samples: Vec<i16> = source.convert_samples().collect();

        // Convert to f32 and mix to mono
        let audio_buffer = samples
            .chunks_exact(channels as usize)
            .map(|chunk| {
                let sum: f32 = chunk.iter().map(|&s| s as f32 / 32768.0).sum();
                sum / channels as f32
            })
            .collect();

        Ok(audio_buffer)
    }

    fn update_statistics(&self, stats: &mut AnalysisStatistics, frame: &AudioFrame,
                        beat_count: &mut u32, bpm_values: &mut Vec<f32>) {
        // Update peak values
        stats.peak_bass = stats.peak_bass.max(frame.frequency_bands.bass);
        stats.peak_mid = stats.peak_mid.max(frame.frequency_bands.mid);
        stats.peak_treble = stats.peak_treble.max(frame.frequency_bands.treble);
        stats.peak_presence = stats.peak_presence.max(frame.frequency_bands.presence);
        stats.peak_volume = stats.peak_volume.max(frame.volume);
        stats.peak_spectral_flux = stats.peak_spectral_flux.max(frame.spectral_flux);
        stats.peak_onset = stats.peak_onset.max(frame.onset_strength);

        // Track beats and BPM
        if frame.beat_detected {
            *beat_count += 1;
            if frame.estimated_bpm > 60.0 && frame.estimated_bpm < 200.0 {
                bpm_values.push(frame.estimated_bpm);
            }
        }
    }

    fn classify_content(&self, stats: &mut AnalysisStatistics, frames: &[PrescanFrame]) {
        // Determine dominant frequency range
        let avg_bass: f32 = frames.iter().map(|f| f.frequency_bands.bass).sum::<f32>() / frames.len() as f32;
        let avg_mid: f32 = frames.iter().map(|f| f.frequency_bands.mid).sum::<f32>() / frames.len() as f32;
        let avg_treble: f32 = frames.iter().map(|f| f.frequency_bands.treble).sum::<f32>() / frames.len() as f32;

        stats.dominant_frequency_range = if avg_bass > avg_mid && avg_bass > avg_treble {
            "Bass-Heavy".to_string()
        } else if avg_treble > avg_bass && avg_treble > avg_mid {
            "Treble-Focused".to_string()
        } else {
            "Balanced".to_string()
        };

        // Determine energy profile
        let avg_volume: f32 = frames.iter().map(|f| f.volume).sum::<f32>() / frames.len() as f32;
        let volume_variance: f32 = frames.iter()
            .map(|f| (f.volume - avg_volume).powi(2))
            .sum::<f32>() / frames.len() as f32;

        stats.energy_profile = if volume_variance > 0.1 {
            "Dynamic".to_string()
        } else if avg_volume > 0.3 {
            "High".to_string()
        } else if avg_volume > 0.1 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        // Calculate complexity score (0.0-1.0)
        let spectral_complexity = frames.iter().map(|f| f.spectral_flux).sum::<f32>() / frames.len() as f32;
        let harmonic_complexity = frames.iter().map(|f| f.pitch_confidence).sum::<f32>() / frames.len() as f32;
        stats.complexity_score = (spectral_complexity + harmonic_complexity + volume_variance).min(1.0);
    }
}

impl Default for AnalysisStatistics {
    fn default() -> Self {
        Self {
            peak_bass: 0.0,
            peak_mid: 0.0,
            peak_treble: 0.0,
            peak_presence: 0.0,
            peak_volume: 0.0,
            peak_spectral_flux: 0.0,
            peak_onset: 0.0,
            total_beats: 0,
            average_bpm: 120.0,
            bpm_range: (60.0, 180.0),
            dominant_frequency_range: "Unknown".to_string(),
            energy_profile: "Unknown".to_string(),
            complexity_score: 0.5,
        }
    }
}

/// Real-time synchronized playback using pre-scanned data
pub struct SynchronizedPlayback {
    prescan_data: PrescanData,
    current_time: f32,
    frame_index: usize,
}

impl SynchronizedPlayback {
    pub fn new(prescan_data: PrescanData) -> Self {
        Self {
            prescan_data,
            current_time: 0.0,
            frame_index: 0,
        }
    }

    /// Get audio frame for current playback time with perfect synchronization
    pub fn get_synchronized_frame(&mut self, playback_time_seconds: f32) -> Option<&PrescanFrame> {
        self.current_time = playback_time_seconds;

        // Find the frame closest to current time
        while self.frame_index < self.prescan_data.frames.len() {
            let frame = &self.prescan_data.frames[self.frame_index];

            if frame.timestamp <= playback_time_seconds {
                if self.frame_index + 1 < self.prescan_data.frames.len() {
                    let next_frame = &self.prescan_data.frames[self.frame_index + 1];
                    if next_frame.timestamp > playback_time_seconds {
                        return Some(frame);
                    } else {
                        self.frame_index += 1;
                    }
                } else {
                    return Some(frame);
                }
            } else {
                break;
            }
        }

        // If we're behind, find the correct frame
        if self.frame_index > 0 {
            while self.frame_index > 0 &&
                  self.prescan_data.frames[self.frame_index].timestamp > playback_time_seconds {
                self.frame_index -= 1;
            }
        }

        self.prescan_data.frames.get(self.frame_index)
    }

    /// Get statistics for this audio file
    pub fn get_statistics(&self) -> &AnalysisStatistics {
        &self.prescan_data.statistics
    }

    /// Get file info
    pub fn get_file_info(&self) -> &FileInfo {
        &self.prescan_data.file_info
    }
}