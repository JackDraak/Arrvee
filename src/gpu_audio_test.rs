use anyhow::Result;
use clap::Parser;
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
mod effects;

use graphics::GraphicsEngine;
use audio::{AudioPlayback, AudioFrame};

#[derive(Parser)]
#[command(name = "arrvee-gpu-audio-test")]
#[command(about = "Arrvee Music Visualizer - GPU-Accelerated Audio Analysis Test")]
struct Args {
    /// Audio file to visualize (WAV, MP3, OGG, M4A)
    #[arg(default_value = "sample.wav")]
    audio_file: String,

    /// Use GPU compute shaders for audio analysis
    #[arg(long, short)]
    gpu: bool,

    /// Show developer overlay with analysis stats
    #[arg(long, short)]
    debug: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Starting GPU Audio Analysis Test");
    info!("Audio file: {}", args.audio_file);
    info!("GPU acceleration: {}", args.gpu);
    info!("Debug overlay: {}", args.debug);

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee GPU Audio Analysis Test")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?);

    let mut graphics_engine = pollster::block_on(GraphicsEngine::new(&window))?;
    let mut shutdown_requested = false;
    let mut audio_playback = AudioPlayback::new()?;

    // Load and start playing the specified audio file
    info!("Loading {}...", args.audio_file);
    audio_playback.load_file(&args.audio_file)?;
    audio_playback.play();
    info!("Audio playback started");

    // Initialize and test GPU analyzer availability
    if args.gpu {
        info!("Initializing GPU audio analysis capabilities...");
        match pollster::block_on(graphics_engine.init_gpu_analyzer()) {
            Ok(_) => {
                info!("✅ GPU audio analyzer initialized successfully!");
                // Test with a small chunk of silence
                let test_chunk = vec![0.0f32; 512];
                match pollster::block_on(graphics_engine.analyze_audio_gpu(&test_chunk)) {
                    Some(_) => info!("✅ GPU audio analysis is working!"),
                    None => info!("❌ GPU audio analysis test failed"),
                }
            }
            Err(e) => {
                info!("❌ GPU audio analyzer initialization failed: {}", e);
                info!("Falling back to CPU analysis");
            }
        }
    }

    info!("GPU Audio test initialized successfully");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested - cleaning up...");
                    shutdown_requested = true;
                    audio_playback.stop();
                    graphics_engine.cleanup();
                    info!("Cleanup complete");
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event,
                    ..
                } => {
                    if event.state == ElementState::Pressed {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Escape) => {
                                info!("Escape pressed - cleaning up...");
                                shutdown_requested = true;
                                audio_playback.stop();
                                graphics_engine.cleanup();
                                info!("Cleanup complete");
                                elwt.exit();
                            }
                            PhysicalKey::Code(KeyCode::Space) => {
                                if audio_playback.is_playing() {
                                    audio_playback.pause();
                                    info!("Audio paused");
                                } else {
                                    audio_playback.play();
                                    info!("Audio resumed");
                                }
                            }
                            PhysicalKey::Code(KeyCode::KeyG) => {
                                // Test GPU analysis on demand
                                let audio_chunk = audio_playback.get_current_audio_chunk();
                                match pollster::block_on(graphics_engine.analyze_audio_gpu(&audio_chunk)) {
                                    Some(gpu_features) => {
                                        info!("🎵 GPU Analysis Results:");
                                        info!("  Bass: {:.3}, Mid: {:.3}, Treble: {:.3}",
                                              gpu_features.bass, gpu_features.mid, gpu_features.treble);
                                        info!("  Beat Strength: {:.3}, BPM: {:.1}",
                                              gpu_features.beat_strength, gpu_features.estimated_bpm);
                                        info!("  Volume: {:.3}, Spectral Centroid: {:.1}",
                                              gpu_features.volume, gpu_features.spectral_centroid);
                                    }
                                    None => {
                                        info!("❌ GPU analysis failed");
                                    }
                                }
                            }
                            // Effect switching controls
                            PhysicalKey::Code(KeyCode::Digit1) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("llama_plasma".to_string()));
                                info!("🌈 Effect: Llama Plasma Fields");
                            }
                            PhysicalKey::Code(KeyCode::Digit2) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("geometric_kaleidoscope".to_string()));
                                info!("🌈 Effect: Geometric Kaleidoscope");
                            }
                            PhysicalKey::Code(KeyCode::Digit3) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("psychedelic_tunnel".to_string()));
                                info!("🌈 Effect: Psychedelic Tunnel");
                            }
                            PhysicalKey::Code(KeyCode::Digit4) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("particle_swarm".to_string()));
                                info!("🌈 Effect: Particle Swarm");
                            }
                            PhysicalKey::Code(KeyCode::Digit5) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("fractal_madness".to_string()));
                                info!("🌈 Effect: Fractal Madness");
                            }
                            PhysicalKey::Code(KeyCode::Digit6) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("spectralizer_bars".to_string()));
                                info!("🌈 Effect: Spectralizer Bars");
                            }
                            PhysicalKey::Code(KeyCode::Digit0) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(None);
                                info!("🌈 Effect: Auto-Blend Mode");
                            }
                            // Palette switching
                            PhysicalKey::Code(KeyCode::KeyP) => {
                                graphics_engine.palette_index = (graphics_engine.palette_index + 1.0) % 6.0;
                                let palette_names = ["Rainbow", "Neon Cyber", "Warm Sunset", "Deep Ocean", "Purple Haze", "Electric Green"];
                                let palette_name = palette_names[graphics_engine.palette_index as usize];
                                info!("🎨 Palette: {}", palette_name);
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    graphics_engine.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    if shutdown_requested {
                        return; // Don't render after shutdown requested
                    }

                    // Get audio data for analysis
                    let audio_data = if args.gpu {
                        // Try GPU analysis first
                        let audio_chunk = audio_playback.get_current_audio_chunk();
                        match pollster::block_on(graphics_engine.analyze_audio_gpu(&audio_chunk)) {
                            Some(gpu_features) => {
                                // Convert GPU features to AudioFrame format
                                graphics_engine.gpu_features_to_audio_frame(&gpu_features)
                            }
                            None => {
                                // Fallback to CPU analysis
                                audio_playback.get_current_audio_frame()
                            }
                        }
                    } else {
                        // Use CPU analysis
                        audio_playback.get_current_audio_frame()
                    };

                    // Print performance info every 60 frames (1 second at 60fps)
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 60 == 0 && args.debug {
                            info!("📊 Analysis Mode: {} | Bass: {:.3} | Beat: {:.3} | BPM: {:.1}",
                                  if args.gpu { "GPU" } else { "CPU" },
                                  audio_data.frequency_bands.bass,
                                  audio_data.beat_strength,
                                  audio_data.estimated_bpm);
                        }
                    }

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