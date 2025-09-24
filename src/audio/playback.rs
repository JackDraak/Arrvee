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
                // Calculate current position based on playback time
                // Use smaller chunks for better real-time responsiveness
                let chunk_size = 512; // Smaller chunks = better responsiveness (11.6ms at 44.1kHz)
                let start = self.buffer_position;
                let end = (start + chunk_size).min(self.audio_buffer.len());

                if start < self.audio_buffer.len() {
                    let chunk = &self.audio_buffer[start..end];
                    // Advance at real-time rate: 44100 samples/sec = ~735 samples per frame at 60fps
                    self.buffer_position = (self.buffer_position + 735) % self.audio_buffer.len();

                    return analyzer.analyze(chunk);
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
}