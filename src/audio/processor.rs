use anyhow::Result;
use cpal::{Device, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use log::{info, warn};

use super::{AudioAnalyzer, AudioFrame, BeatDetector};

pub struct AudioProcessor {
    #[allow(dead_code)]
    stream: Stream,
    audio_receiver: Receiver<Vec<f32>>,
    analyzer: AudioAnalyzer,
    beat_detector: BeatDetector,
    latest_frame: Arc<Mutex<AudioFrame>>,
}

impl AudioProcessor {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get default input config: {}", e))?;

        info!("Using audio device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
        info!("Audio config: {:?}", config);

        let sample_rate = config.sample_rate().0 as f32;
        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();
        let latest_frame = Arc::new(Mutex::new(AudioFrame::default()));

        let stream = Self::create_input_stream(&device, &config.into(), audio_sender)?;
        stream.play()?;

        let analyzer = AudioAnalyzer::new(sample_rate, 1024);
        let beat_detector = BeatDetector::new(sample_rate);

        Ok(Self {
            stream,
            audio_receiver,
            analyzer,
            beat_detector,
            latest_frame,
        })
    }

    fn create_input_stream(
        device: &Device,
        config: &StreamConfig,
        sender: Sender<Vec<f32>>,
    ) -> Result<Stream> {
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;

        info!("Creating input stream with {} channels at {} Hz", channels, sample_rate);

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mono_data: Vec<f32> = if channels == 1 {
                    data.to_vec()
                } else {
                    data.chunks(channels)
                        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                        .collect()
                };

                if sender.send(mono_data).is_err() {
                    warn!("Failed to send audio data");
                }
            },
            |err| {
                warn!("Audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    pub fn get_latest_frame(&mut self) -> AudioFrame {
        while let Ok(audio_data) = self.audio_receiver.try_recv() {
            if audio_data.len() >= 1024 {
                let mut frame = self.analyzer.analyze(&audio_data);

                let beat_info = self.beat_detector.detect_beat(&frame.frequency_bands);
                frame.beat_detected = beat_info.0;
                frame.beat_strength = beat_info.1;

                frame.volume = audio_data.iter()
                    .map(|&x| x.abs())
                    .sum::<f32>() / audio_data.len() as f32;

                if let Ok(mut latest) = self.latest_frame.try_lock() {
                    *latest = frame;
                }
            }
        }

        self.latest_frame.lock().unwrap().clone()
    }
}