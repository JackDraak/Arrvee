use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use log::info;
use crate::audio::{AudioFrame, AudioAnalyzer};

pub struct AudioPlayback {
    #[allow(dead_code)]
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    analyzer: Option<AudioAnalyzer>,
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
            sample_rate: 44100,
            audio_buffer: Vec::new(),
            buffer_position: 0,
        })
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
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

        // Create analyzer
        self.analyzer = Some(AudioAnalyzer::new(self.sample_rate as f32, 512));
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

    pub fn get_current_audio_frame(&mut self) -> AudioFrame {
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
                                let analysis = analyzer.analyze(window);

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
                        return analyzer.analyze(&padded_chunk);
                    }
                }
            }
        }

        // Return empty frame if no data
        AudioFrame::default()
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
}