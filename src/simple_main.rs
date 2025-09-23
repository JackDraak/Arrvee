use anyhow::Result;
use log::info;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

struct SimpleVisualizer {
    audio_receiver: Receiver<Vec<f32>>,
    spectrum_data: Arc<Mutex<Vec<f32>>>,
}

impl SimpleVisualizer {
    fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get default input config: {}", e))?;

        info!("Using audio device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
        info!("Audio config: {:?}", config);

        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();
        let spectrum_data = Arc::new(Mutex::new(vec![0.0; 256]));

        let channels = config.channels() as usize;
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mono_data: Vec<f32> = if channels == 1 {
                    data.to_vec()
                } else {
                    data.chunks(channels)
                        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                        .collect()
                };

                if audio_sender.send(mono_data).is_err() {
                    log::warn!("Failed to send audio data");
                }
            },
            |err| {
                log::warn!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;

        // Keep the stream alive
        std::mem::forget(stream);

        Ok(Self {
            audio_receiver,
            spectrum_data,
        })
    }

    fn process_audio(&self) {
        while let Ok(audio_data) = self.audio_receiver.try_recv() {
            if audio_data.len() >= 512 {
                let spectrum = self.compute_fft(&audio_data[..512]);
                if let Ok(mut data) = self.spectrum_data.try_lock() {
                    *data = spectrum;
                }
            }
        }
    }

    fn compute_fft(&self, audio_data: &[f32]) -> Vec<f32> {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(512);

        let mut buffer: Vec<Complex<f32>> = audio_data
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        fft.process(&mut buffer);

        buffer[..256]
            .iter()
            .map(|c| c.norm() * 2.0 / 512.0)
            .collect()
    }

    fn print_visualization(&self) {
        if let Ok(spectrum) = self.spectrum_data.try_lock() {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top

            println!("Arrvee Music Visualizer - Simple Audio Spectrum");
            println!("===============================================");
            println!();

            // Create a simple ASCII bar visualization
            for (i, &magnitude) in spectrum.iter().enumerate().take(32) {
                let bar_height = (magnitude * 50.0) as usize;
                let freq = (i as f32 * 44100.0) / 512.0;

                print!("{:6.0}Hz |", freq);
                for _ in 0..bar_height.min(50) {
                    print!("█");
                }
                println!(" {:.3}", magnitude);
            }

            println!();
            println!("Bass: {:.3}", spectrum[0..8].iter().sum::<f32>() / 8.0);
            println!("Mid:  {:.3}", spectrum[8..32].iter().sum::<f32>() / 24.0);
            println!("High: {:.3}", spectrum[32..128].iter().sum::<f32>() / 96.0);
            println!();
            println!("Press Ctrl+C to exit");
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Simple Arrvee Music Visualizer");

    let visualizer = SimpleVisualizer::new()?;

    info!("Visualizer initialized successfully");
    info!("Listening for audio input...");

    loop {
        visualizer.process_audio();
        visualizer.print_visualization();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}