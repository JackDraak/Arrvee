use anyhow::Result;
use log::info;
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

mod audio;
mod graphics;
mod ui;
mod effects;

use audio::AudioPlayback;
use graphics::GraphicsEngine;
use ui::UserInterface;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Arrvee Music Visualizer");

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee Music Visualizer")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?);

    let mut graphics_engine = pollster::block_on(GraphicsEngine::new(&window))?;
    let mut audio_playback = AudioPlayback::new()?;
    let mut ui = UserInterface::new(&window, &graphics_engine);

    // Load sample audio file
    audio_playback.load_file("sample.wav")?;
    audio_playback.play();

    info!("Visualizer initialized successfully");

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
                    let audio_data = audio_playback.get_current_audio_frame();
                    if let Err(e) = graphics_engine.render(&audio_data, &window_clone) {
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
