use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use log::info;
use crate::audio::{AudioFrame, AudioAnalyzer, CpuAudioAnalyzer, NewGpuAudioAnalyzer, FeatureNormalizer, NormalizedAudioFeatures};

pub struct AudioPlayback {
    #[allow(dead_code)]
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    analyzer: Option<Box<dyn AudioAnalyzer + Send>>,
    normalizer: Option<FeatureNormalizer>,
    sensitivity_factor: f32,
    sample_rate: u32,
    audio_buffer: Vec<f32>,
    buffer_position: usize,
}

impl AudioPlayback {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;

        Ok(Self {
            stream,
            stream_handle,
            sink: None,
            analyzer: None,
            normalizer: None,
            sensitivity_factor: 1.0,
            sample_rate: 44100,
            audio_buffer: Vec::new(),
            buffer_position: 0,
        })
    }

    pub async fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = BufReader::new(File::open(&path)?);
        let source = Decoder::new(file)?;

        // Get sample rate and convert to f32 samples for analysis
        self.sample_rate = source.sample_rate();
        let channels = source.channels();

        // Collect samples for analysis
        let samples: Vec<i16> = source.convert_samples().collect();

        // Convert to f32 and mix to mono for analysis
        self.audio_buffer = samples
            .chunks_exact(channels as usize)
            .map(|chunk| {
                let sum: f32 = chunk.iter().map(|&s| s as f32 / 32768.0).sum();
                sum / channels as f32
            })
            .collect();

        // Create unified analyzer with GPU/CPU fallback
        let chunk_size = 512;
        let sample_rate_f32 = self.sample_rate as f32;

        info!("Initializing audio analyzer with unified architecture...");
        let analyzer: Box<dyn AudioAnalyzer + Send> = match NewGpuAudioAnalyzer::new_standalone(sample_rate_f32, chunk_size).await {
            Ok(gpu_analyzer) => {
                info!("✅ GPU analyzer initialized successfully");
                Box::new(gpu_analyzer)
            }
            Err(e) => {
                info!("⚠️  GPU initialization failed: {}. Falling back to CPU.", e);
                Box::new(CpuAudioAnalyzer::new(sample_rate_f32, chunk_size)?)
            }
        };

        self.analyzer = Some(analyzer);
        self.normalizer = Some(FeatureNormalizer::new());
        self.buffer_position = 0;

        // Load file again for playback (since we consumed the decoder above)
        let file = BufReader::new(File::open(&path)?);
        let source = Decoder::new(file)?;
        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.pause();

        info!("Loaded audio file: {:?} ({}Hz, {} samples)", path.as_ref(), self.sample_rate, self.audio_buffer.len());
        self.sink = Some(sink);

        Ok(())
    }

    pub fn play(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
            info!("Audio playback started");
        }
    }

    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            info!("Audio playback paused");
        }
    }

    pub fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
            info!("Audio playback stopped");
        }
    }

    pub fn set_volume(&self, volume: f32) {
        if let Some(sink) = &self.sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map_or(false, |sink| !sink.is_paused())
    }

    pub fn is_finished(&self) -> bool {
        self.sink.as_ref().map_or(true, |sink| sink.empty())
    }

    pub async fn get_current_audio_frame(&mut self) -> AudioFrame {
        if let Some(analyzer) = &mut self.analyzer {
            if !self.audio_buffer.is_empty() {
                // At 60fps, we should process ~735 samples per frame (44100/60)
                let samples_per_frame = 735;
                let chunk_size = 512; // Analysis window size

                let start = self.buffer_position;
                let end = (start + samples_per_frame).min(self.audio_buffer.len());

                if start < self.audio_buffer.len() {
                    // Process all accumulated samples in this frame using overlapping windows
                    let frame_data = &self.audio_buffer[start..end];

                    if frame_data.len() >= chunk_size {
                        // Average multiple overlapping analysis windows within this frame
                        let mut accumulated_frame = AudioFrame::default();
                        let mut analysis_count = 0;

                        // Analyze overlapping windows within the frame data
                        let step_size = (frame_data.len().saturating_sub(chunk_size) / 4).max(1); // 4 overlapping analyses

                        for window_start in (0..frame_data.len().saturating_sub(chunk_size)).step_by(step_size) {
                            let window_end = (window_start + chunk_size).min(frame_data.len());
                            let window = &frame_data[window_start..window_end];

                            if window.len() == chunk_size {
                                // Get raw features from unified analyzer
                                if let Ok(raw_features) = analyzer.analyze_chunk(window).await {
                                    if let Some(normalizer) = &mut self.normalizer {
                                        let normalized_features = normalizer.normalize(&raw_features);
                                        let analysis = Self::convert_to_audio_frame_static(&normalized_features, self.sample_rate as f32, self.sensitivity_factor);

                                        // Accumulate all analysis values
                                        accumulated_frame.volume += analysis.volume;
                                        accumulated_frame.beat_strength += analysis.beat_strength;
                                        accumulated_frame.spectral_centroid += analysis.spectral_centroid;
                                        accumulated_frame.spectral_rolloff += analysis.spectral_rolloff;
                                        accumulated_frame.zero_crossing_rate += analysis.zero_crossing_rate;
                                        accumulated_frame.spectral_flux += analysis.spectral_flux;
                                        accumulated_frame.onset_strength += analysis.onset_strength;
                                        accumulated_frame.pitch_confidence += analysis.pitch_confidence;
                                        accumulated_frame.dynamic_range += analysis.dynamic_range;

                                        // Accumulate frequency bands
                                        accumulated_frame.frequency_bands.bass += analysis.frequency_bands.bass;
                                        accumulated_frame.frequency_bands.mid += analysis.frequency_bands.mid;
                                        accumulated_frame.frequency_bands.treble += analysis.frequency_bands.treble;
                                        accumulated_frame.frequency_bands.sub_bass += analysis.frequency_bands.sub_bass;
                                        accumulated_frame.frequency_bands.presence += analysis.frequency_bands.presence;

                                        // Keep the most recent beat detection and BPM
                                        if analysis.beat_detected {
                                            accumulated_frame.beat_detected = true;
                                            accumulated_frame.estimated_bpm = analysis.estimated_bpm;
                                        }

                                        analysis_count += 1;
                                    }
                                }
                            }
                        }

                        // Average all accumulated values
                        if analysis_count > 0 {
                            let count_f32 = analysis_count as f32;
                            accumulated_frame.volume /= count_f32;
                            accumulated_frame.beat_strength /= count_f32;
                            accumulated_frame.spectral_centroid /= count_f32;
                            accumulated_frame.spectral_rolloff /= count_f32;
                            accumulated_frame.zero_crossing_rate /= count_f32;
                            accumulated_frame.spectral_flux /= count_f32;
                            accumulated_frame.onset_strength /= count_f32;
                            accumulated_frame.pitch_confidence /= count_f32;
                            accumulated_frame.dynamic_range /= count_f32;

                            // Average frequency bands
                            accumulated_frame.frequency_bands.bass /= count_f32;
                            accumulated_frame.frequency_bands.mid /= count_f32;
                            accumulated_frame.frequency_bands.treble /= count_f32;
                            accumulated_frame.frequency_bands.sub_bass /= count_f32;
                            accumulated_frame.frequency_bands.presence /= count_f32;

                            // Set sample rate
                            accumulated_frame.sample_rate = self.sample_rate as f32;
                        }

                        // Advance buffer position by the frame amount
                        self.buffer_position = (self.buffer_position + samples_per_frame) % self.audio_buffer.len();

                        return accumulated_frame;
                    } else {
                        // Fallback: if frame data is too small, just analyze what we have
                        let padded_chunk = if frame_data.len() < chunk_size {
                            let mut padded = vec![0.0; chunk_size];
                            padded[..frame_data.len()].copy_from_slice(frame_data);
                            padded
                        } else {
                            frame_data.to_vec()
                        };

                        self.buffer_position = (self.buffer_position + samples_per_frame) % self.audio_buffer.len();

                        // Use new async analysis with normalization
                        if let Ok(raw_features) = analyzer.analyze_chunk(&padded_chunk).await {
                            if let Some(normalizer) = &mut self.normalizer {
                                let normalized_features = normalizer.normalize(&raw_features);
                                return Self::convert_to_audio_frame_static(&normalized_features, self.sample_rate as f32, self.sensitivity_factor);
                            }
                        }
                    }
                }
            }
        }

        // Return empty frame if no data
        AudioFrame::default()
    }

    /// Static version of convert_to_audio_frame to avoid borrowing issues
    fn convert_to_audio_frame_static(normalized: &NormalizedAudioFeatures, sample_rate: f32, sensitivity: f32) -> AudioFrame {
        use crate::audio::FrequencyBands;
        use log::debug;

        // Debug logging to understand the actual values (only log occasionally to avoid spam)
        static mut DEBUG_COUNTER: u32 = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            if DEBUG_COUNTER % 120 == 0 { // Log every ~2 seconds at 60fps
                debug!("🔍 AUDIO PIPELINE VALUES (sensitivity: {:.1}x):", sensitivity);
                debug!("  📊 Normalized Input: bass={:.4}, mid={:.4}, volume={:.4}",
                    normalized.bass, normalized.mid, normalized.volume);
                debug!("  🎚️ After Sensitivity: bass={:.4}, mid={:.4}, volume={:.4}",
                    (normalized.bass * sensitivity).clamp(0.0, 1.0),
                    (normalized.mid * sensitivity).clamp(0.0, 1.0),
                    (normalized.volume * sensitivity).clamp(0.0, 1.0));
            }
        }

        // Apply baseline boost for minimum visual responsiveness
        let baseline_boost = 0.05; // Ensure minimum 5% activity even in silence
        let dynamic_boost = 2.0;   // Extra multiplier for better dynamic range

        AudioFrame {
            sample_rate,
            spectrum: Vec::new(), // Not used in current analysis
            time_domain: Vec::new(), // Not used in current analysis
            frequency_bands: FrequencyBands {
                sub_bass: (baseline_boost + normalized.sub_bass * sensitivity * dynamic_boost).clamp(0.0, 1.0),
                bass: (baseline_boost + normalized.bass * sensitivity * dynamic_boost).clamp(0.0, 1.0),
                mid: (baseline_boost + normalized.mid * sensitivity * dynamic_boost).clamp(0.0, 1.0),
                treble: (baseline_boost + normalized.treble * sensitivity * dynamic_boost).clamp(0.0, 1.0),
                presence: (baseline_boost + normalized.presence * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            },
            beat_detected: normalized.beat_detected,
            beat_strength: (baseline_boost + normalized.beat_strength * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            estimated_bpm: normalized.estimated_bpm, // BPM not affected by sensitivity
            volume: (baseline_boost + normalized.volume * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            spectral_centroid: normalized.spectral_centroid, // Keep raw for analysis
            spectral_rolloff: normalized.spectral_rolloff, // Keep raw for analysis
            pitch_confidence: normalized.pitch_confidence, // Keep raw for analysis
            zero_crossing_rate: normalized.zero_crossing_rate, // Keep raw for analysis
            spectral_flux: (baseline_boost + normalized.spectral_flux * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            onset_strength: (baseline_boost + normalized.onset_strength * sensitivity * dynamic_boost).clamp(0.0, 1.0),
            dynamic_range: (baseline_boost + normalized.dynamic_range * sensitivity * dynamic_boost).clamp(0.0, 1.0),
        }
    }

    /// Convert normalized audio features to AudioFrame format for compatibility
    fn convert_to_audio_frame(&self, normalized: &NormalizedAudioFeatures) -> AudioFrame {
        use crate::audio::FrequencyBands;

        // Apply sensitivity scaling to key visual parameters
        let sensitivity = self.sensitivity_factor;

        AudioFrame {
            sample_rate: self.sample_rate as f32,
            spectrum: Vec::new(), // Not used in current analysis
            time_domain: Vec::new(), // Not used in current analysis
            frequency_bands: FrequencyBands {
                sub_bass: (normalized.sub_bass * sensitivity).clamp(0.0, 1.0),
                bass: (normalized.bass * sensitivity).clamp(0.0, 1.0),
                mid: (normalized.mid * sensitivity).clamp(0.0, 1.0),
                treble: (normalized.treble * sensitivity).clamp(0.0, 1.0),
                presence: (normalized.presence * sensitivity).clamp(0.0, 1.0),
            },
            beat_detected: normalized.beat_detected,
            beat_strength: (normalized.beat_strength * sensitivity).clamp(0.0, 1.0),
            estimated_bpm: normalized.estimated_bpm, // BPM not affected by sensitivity
            volume: (normalized.volume * sensitivity).clamp(0.0, 1.0),
            spectral_centroid: normalized.spectral_centroid, // Keep raw for analysis
            spectral_rolloff: normalized.spectral_rolloff, // Keep raw for analysis
            pitch_confidence: normalized.pitch_confidence, // Keep raw for analysis
            zero_crossing_rate: normalized.zero_crossing_rate, // Keep raw for analysis
            spectral_flux: (normalized.spectral_flux * sensitivity).clamp(0.0, 1.0),
            onset_strength: (normalized.onset_strength * sensitivity).clamp(0.0, 1.0),
            dynamic_range: (normalized.dynamic_range * sensitivity).clamp(0.0, 1.0),
        }
    }

    /// Get raw audio data for GPU processing
    pub fn get_current_audio_chunk(&mut self) -> Vec<f32> {
        if !self.audio_buffer.is_empty() {
            let chunk_size = 512; // Same size as GPU analyzer expects
            let start = self.buffer_position;
            let end = (start + chunk_size).min(self.audio_buffer.len());

            if start < self.audio_buffer.len() {
                let chunk = self.audio_buffer[start..end].to_vec();
                // Advance at real-time rate: 44100 samples/sec = ~735 samples per frame at 60fps
                self.buffer_position = (self.buffer_position + 735) % self.audio_buffer.len();
                return chunk;
            }
        }

        // Return silence if no data
        vec![0.0; 512]
    }

    /// Get the full audio buffer for comprehensive analysis
    pub fn get_full_audio_buffer(&self) -> &Vec<f32> {
        &self.audio_buffer
    }

    /// Get current sensitivity factor
    pub fn get_sensitivity(&self) -> f32 {
        self.sensitivity_factor
    }

    /// Set sensitivity factor (expanded range: 0.1 = very low, 5.0 = very high)
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity_factor = sensitivity.clamp(0.1, 5.0);
    }

    /// Adjust sensitivity by delta (e.g., +0.1 or -0.1)
    pub fn adjust_sensitivity(&mut self, delta: f32) -> f32 {
        self.sensitivity_factor = (self.sensitivity_factor + delta).clamp(0.1, 5.0);
        self.sensitivity_factor
    }

    /// Legacy compatibility: return self for analyzer access
    pub fn analyzer(&self) -> Option<&Self> {
        Some(self)
    }

    /// Legacy compatibility: return self for mutable analyzer access
    pub fn analyzer_mut(&mut self) -> Option<&mut Self> {
        Some(self)
    }

}