use anyhow::Result;
use log::info;
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

mod graphics;
mod ui;
mod audio;
mod effects;

use graphics::GraphicsEngine;
use ui::UserInterface;
use audio::AudioFrame;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Graphics Test");

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee Graphics Test")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?);

    let mut graphics_engine = pollster::block_on(GraphicsEngine::new(&window))?;
    let mut ui = UserInterface::new(&window, &graphics_engine);

    info!("Graphics test initialized successfully");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event,
                    ..
                } => {
                    if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                        && event.state == ElementState::Pressed {
                        info!("Escape pressed");
                        elwt.exit();
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    graphics_engine.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Create fake audio data for testing
                    let fake_audio = AudioFrame {
                        sample_rate: 44100.0,
                        spectrum: vec![0.1; 512],
                        time_domain: vec![0.1; 1024],
                        frequency_bands: audio::FrequencyBands {
                            bass: 0.3,
                            mid: 0.2,
                            treble: 0.1,
                            sub_bass: 0.4,
                            presence: 0.05,
                        },
                        beat_detected: true,
                        beat_strength: 0.8,
                        volume: 0.5,
                        spectral_centroid: 0.6,
                        spectral_rolloff: 0.7,
                        zero_crossing_rate: 0.3,
                        spectral_flux: 0.4,
                        onset_strength: 0.5,
                        pitch_confidence: 0.8,
                        estimated_bpm: 128.0,
                        dynamic_range: 0.6,
                    };

                    if let Err(e) = graphics_engine.render(&fake_audio, &window_clone) {
                        log::error!("Render error: {}", e);
                    }
                }
                _ => {
                    ui.handle_event(&event, &window_clone);
                }
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}