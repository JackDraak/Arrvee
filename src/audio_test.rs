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
mod audio;

use graphics::GraphicsEngine;
use audio::AudioPlayback;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Audio File Test with Real-time Visualization");

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee Audio File Test")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?);

    let mut graphics_engine = pollster::block_on(GraphicsEngine::new(&window))?;
    let mut audio_playback = AudioPlayback::new()?;

    // Load and start playing the sample file
    info!("Loading sample.wav...");
    audio_playback.load_file("sample.wav")?;
    audio_playback.play();
    info!("Audio playback started - you should hear the music!");

    info!("Audio file test initialized successfully");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");
                    audio_playback.stop();
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event,
                    ..
                } => {
                    if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                        && event.state == ElementState::Pressed {
                        info!("Escape pressed");
                        audio_playback.stop();
                        elwt.exit();
                    }
                    if event.physical_key == PhysicalKey::Code(KeyCode::Space)
                        && event.state == ElementState::Pressed {
                        if audio_playback.is_playing() {
                            audio_playback.pause();
                            info!("Audio paused");
                        } else {
                            audio_playback.play();
                            info!("Audio resumed");
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    graphics_engine.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Get real-time audio analysis from the loaded file
                    let audio_data = audio_playback.get_current_audio_frame();

                    if let Err(e) = graphics_engine.render(&audio_data, &window_clone) {
                        log::error!("Render error: {}", e);
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                // Check if audio finished
                if audio_playback.is_finished() {
                    info!("Audio finished playing");
                    elwt.exit();
                }
                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}