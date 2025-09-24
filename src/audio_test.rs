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

// Enhanced terminal-based debug interface (egui integration would go here for future GUI overlay)

mod graphics;
mod audio;
mod effects;

use graphics::GraphicsEngine;
use audio::{AudioPlayback, AudioFrame};

struct DebugOverlay {
    show_overlay: bool,
    volume_control: f32,
    frame_count: u32,
}

impl DebugOverlay {
    fn new() -> Self {
        Self {
            show_overlay: true,
            volume_control: 1.0,
            frame_count: 0,
        }
    }

    fn render_debug_info(&mut self, audio_frame: &AudioFrame, graphics_engine: &graphics::GraphicsEngine) {
        if !self.show_overlay {
            return;
        }

        self.frame_count += 1;

        // Only update display every 30 frames (roughly twice per second) to reduce spam
        if self.frame_count % 30 != 0 {
            return;
        }

        // Clear screen and position cursor at top
        print!("\x1B[2J\x1B[1;1H");

        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║              🎵 ARRVEE AUDIO ANALYSIS DEBUG 🎵                ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");

        println!("║ 🎵 FREQUENCY BANDS                                            ║");
        println!("║   Sub-Bass: {:>8.3} ■{:<20}                            ║",
                 audio_frame.frequency_bands.sub_bass,
                 "█".repeat((audio_frame.frequency_bands.sub_bass * 20.0) as usize));
        println!("║   Bass:     {:>8.3} ■{:<20}                            ║",
                 audio_frame.frequency_bands.bass,
                 "█".repeat((audio_frame.frequency_bands.bass * 20.0) as usize));
        println!("║   Mid:      {:>8.3} ■{:<20}                            ║",
                 audio_frame.frequency_bands.mid,
                 "█".repeat((audio_frame.frequency_bands.mid * 20.0) as usize));
        println!("║   Treble:   {:>8.3} ■{:<20}                            ║",
                 audio_frame.frequency_bands.treble,
                 "█".repeat((audio_frame.frequency_bands.treble * 20.0) as usize));
        println!("║   Presence: {:>8.3} ■{:<20}                            ║",
                 audio_frame.frequency_bands.presence,
                 "█".repeat((audio_frame.frequency_bands.presence * 20.0) as usize));

        println!("║                                                               ║");
        println!("║ 🥁 RHYTHM ANALYSIS                                            ║");
        println!("║   Beat: {:>12} | Strength: {:>6.3} | BPM: {:>6.1}         ║",
                 if audio_frame.beat_detected { "🔴 DETECTED" } else { "⚪ silent" },
                 audio_frame.beat_strength,
                 audio_frame.estimated_bpm);

        println!("║                                                               ║");
        println!("║ 🎚️ VISUAL CONTROLS                                            ║");
        let palette_names = ["Rainbow", "Neon Cyber", "Warm Sunset", "Deep Ocean", "Purple Haze", "Electric Green"];
        let current_palette = palette_names.get(graphics_engine.palette_index as usize).unwrap_or(&"Unknown");
        println!("║   Volume:    {:>6.1}% | Palette: {:<15} | Smooth: {:>4.1} ║",
                 self.volume_control * 100.0,
                 current_palette,
                 graphics_engine.smoothing_factor);

        let projection_modes = ["Auto", "Spheres", "Cylinder", "Torus", "Flat"];
        let proj_mode = if graphics_engine.projection_mode < 0.0 {
            "Auto"
        } else {
            projection_modes.get(graphics_engine.projection_mode as usize).map_or("Unknown", |v| v)
        };
        println!("║   Projection: {:<10} | Dynamic Range: {:>6.3}             ║",
                 proj_mode, audio_frame.dynamic_range);

        println!("║                                                               ║");
        println!("║ 🌈 ACTIVE EFFECTS                                             ║");
        let effect_weights = graphics_engine.psychedelic_manager().get_effect_weights();
        for (effect, weight) in effect_weights {
            if *weight > 0.01 {
                let effect_name = match effect.as_str() {
                    "llama_plasma" => "Llama Plasma",
                    "geometric_kaleidoscope" => "Kaleidoscope",
                    "psychedelic_tunnel" => "Psyche Tunnel",
                    "particle_swarm" => "Particle Swarm",
                    "fractal_madness" => "Fractal Madness",
                    "spectralizer_bars" => "Spectralizer",
                    _ => effect
                };
                println!("║   {:<15}: {:>6.3} ■{:<15}                    ║",
                         effect_name, weight,
                         "█".repeat((*weight * 15.0) as usize));
            }
        }

        println!("║                                                               ║");
        println!("║ 📊 SPECTRAL FEATURES                                          ║");
        println!("║   Centroid: {:>6.3} | Rolloff: {:>6.3} | Flux: {:>6.3}      ║",
                 audio_frame.spectral_centroid, audio_frame.spectral_rolloff, audio_frame.spectral_flux);
        println!("║   Pitch Conf: {:>5.3} | Zero Cross: {:>5.3} | Onset: {:>5.3}  ║",
                 audio_frame.pitch_confidence, audio_frame.zero_crossing_rate, audio_frame.onset_strength);

        println!("║                                                               ║");
        println!("║ 🎮 CONTROLS                                                   ║");
        println!("║   P: Palette | [/]: Smoothing | Q/W/E/R/T: Projection       ║");
        println!("║   1-6: Effects | 0: Auto | D: Toggle Debug | Space: Pause   ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
    }

    fn toggle_overlay(&mut self) {
        self.show_overlay = !self.show_overlay;
    }

    fn adjust_volume(&mut self, delta: f32) -> f32 {
        self.volume_control = (self.volume_control + delta).clamp(0.0, 2.0);
        self.volume_control
    }
}

#[derive(Parser)]
#[command(name = "arrvee-audio-test")]
#[command(about = "Arrvee Music Visualizer - Audio File Test")]
struct Args {
    /// Audio file to visualize (WAV, MP3, OGG)
    #[arg(default_value = "sample.wav")]
    audio_file: String,

    /// Show developer overlay with analysis stats
    #[arg(long, short)]
    debug: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Starting Audio File Test with Real-time Visualization");
    info!("Audio file: {}", args.audio_file);
    info!("Debug overlay: {}", args.debug);

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee Audio File Test")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?);

    let mut graphics_engine = pollster::block_on(GraphicsEngine::new(&window))?;
    let mut shutdown_requested = false;
    let mut audio_playback = AudioPlayback::new()?;
    let mut debug_overlay = if args.debug {
        Some(DebugOverlay::new())
    } else {
        None
    };

    // Load and start playing the specified audio file
    info!("Loading {}...", args.audio_file);
    audio_playback.load_file(&args.audio_file)?;
    audio_playback.play();
    info!("Audio playback started - you should hear the music!");

    info!("Audio file test initialized successfully");

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
                            PhysicalKey::Code(KeyCode::KeyD) => {
                                if let Some(debug) = &mut debug_overlay {
                                    debug.toggle_overlay();
                                    info!("Debug overlay toggled");
                                }
                            }
                            PhysicalKey::Code(KeyCode::Equal) | PhysicalKey::Code(KeyCode::NumpadAdd) => {
                                if let Some(debug) = &mut debug_overlay {
                                    let new_volume = debug.adjust_volume(0.1);
                                    audio_playback.set_volume(new_volume);
                                    info!("Volume increased to {:.1}%", new_volume * 100.0);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Minus) | PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                                if let Some(debug) = &mut debug_overlay {
                                    let new_volume = debug.adjust_volume(-0.1);
                                    audio_playback.set_volume(new_volume);
                                    info!("Volume decreased to {:.1}%", new_volume * 100.0);
                                }
                            }
                            // Effect switching controls
                            PhysicalKey::Code(KeyCode::Digit1) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("llama_plasma".to_string()));
                                info!("🌈 Effect switched to: Llama Plasma Fields");
                            }
                            PhysicalKey::Code(KeyCode::Digit2) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("geometric_kaleidoscope".to_string()));
                                info!("🌈 Effect switched to: Geometric Kaleidoscope");
                            }
                            PhysicalKey::Code(KeyCode::Digit3) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("psychedelic_tunnel".to_string()));
                                info!("🌈 Effect switched to: Psychedelic Tunnel");
                            }
                            PhysicalKey::Code(KeyCode::Digit4) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("particle_swarm".to_string()));
                                info!("🌈 Effect switched to: Particle Swarm");
                            }
                            PhysicalKey::Code(KeyCode::Digit5) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("fractal_madness".to_string()));
                                info!("🌈 Effect switched to: Fractal Madness");
                            }
                            PhysicalKey::Code(KeyCode::Digit6) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("spectralizer_bars".to_string()));
                                info!("🌈 Effect switched to: Spectralizer Bars");
                            }
                            PhysicalKey::Code(KeyCode::Digit0) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(None);
                                info!("🌈 Effect switched to: Auto-Blend Mode (intelligent music analysis)");
                            }
                            // 3D Projection controls
                            PhysicalKey::Code(KeyCode::KeyQ) => {
                                graphics_engine.projection_mode = -1.0; // Auto projection
                                info!("📐 Projection: Auto (intelligent selection based on music)");
                            }
                            PhysicalKey::Code(KeyCode::KeyW) => {
                                graphics_engine.projection_mode = 0.0; // Sphere projection
                                info!("📐 Projection: Multiple Spheres");
                            }
                            PhysicalKey::Code(KeyCode::KeyE) => {
                                graphics_engine.projection_mode = 1.0; // Cylinder projection
                                info!("📐 Projection: Cylinder");
                            }
                            PhysicalKey::Code(KeyCode::KeyR) => {
                                graphics_engine.projection_mode = 2.0; // Torus projection
                                info!("📐 Projection: Torus (Donut)");
                            }
                            PhysicalKey::Code(KeyCode::KeyT) => {
                                graphics_engine.projection_mode = 3.0; // Flat projection
                                info!("📐 Projection: Flat (Traditional)");
                            }
                            // Palette switching
                            PhysicalKey::Code(KeyCode::KeyP) => {
                                graphics_engine.palette_index = (graphics_engine.palette_index + 1.0) % 6.0;
                                let palette_names = ["Rainbow", "Neon Cyber", "Warm Sunset", "Deep Ocean", "Purple Haze", "Electric Green"];
                                let palette_name = palette_names[graphics_engine.palette_index as usize];
                                info!("🎨 Palette: {} ({})", palette_name, graphics_engine.palette_index as i32);
                            }
                            // Smoothing controls
                            PhysicalKey::Code(KeyCode::BracketLeft) => {
                                graphics_engine.smoothing_factor = (graphics_engine.smoothing_factor - 0.1).max(0.1);
                                info!("🎛️ Smoothing: {:.1}", graphics_engine.smoothing_factor);
                            }
                            PhysicalKey::Code(KeyCode::BracketRight) => {
                                graphics_engine.smoothing_factor = (graphics_engine.smoothing_factor + 0.1).min(2.0);
                                info!("🎛️ Smoothing: {:.1}", graphics_engine.smoothing_factor);
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

                    // Get real-time audio analysis from the loaded file
                    let audio_data = audio_playback.get_current_audio_frame();

                    // Render debug overlay if enabled (limit to ~2Hz to avoid spam)
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 30 == 0 { // Show debug every 30 frames (~2Hz at 60fps)
                            if let Some(debug) = &mut debug_overlay {
                                debug.render_debug_info(&audio_data, &graphics_engine);
                            }
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