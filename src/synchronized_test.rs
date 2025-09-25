use anyhow::Result;
use clap::Parser;
use log::info;
use std::sync::Arc;
use std::time::Instant;
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
use audio::{AudioPlayback, AudioFrame, ArvFormat, SynchronizedPlayback};

struct DebugOverlay {
    show_overlay: bool,
    volume_control: f32,
    frame_count: u32,
    last_sync_info: String,
}

impl DebugOverlay {
    fn new() -> Self {
        Self {
            show_overlay: true,
            volume_control: 0.1, // 10% default volume
            frame_count: 0,
            last_sync_info: String::new(),
        }
    }

    fn render_debug_info(&mut self, audio_frame: &AudioFrame, graphics_engine: &graphics::GraphicsEngine, sync_info: &str) {
        if !self.show_overlay {
            return;
        }

        self.frame_count += 1;

        // Only update display every 30 frames (roughly twice per second) to reduce spam
        if self.frame_count % 30 != 0 {
            return;
        }

        self.last_sync_info = sync_info.to_string();

        // Clear screen and position cursor at top
        print!("\x1B[2J\x1B[1;1H");

        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║       🎵 ARRVEE SYNCHRONIZED VISUALIZATION 🎵                ║");
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
        println!("║ 🎚️ VISUAL CONTROLS & SYNC STATUS                             ║");
        let palette_names = ["Rainbow", "Neon Cyber", "Warm Sunset", "Deep Ocean", "Purple Haze", "Electric Green"];
        let current_palette = palette_names.get(graphics_engine.palette_index as usize).unwrap_or(&"Unknown");
        println!("║   Volume:    {:>6.1}% | Palette: {:<15} | Smooth: {:>4.1} ║",
                 self.volume_control * 100.0,
                 current_palette,
                 graphics_engine.smoothing_factor);

        println!("║   Sync: {:<48} ║", self.last_sync_info);

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
                    "parametric_waves" => "Parametric Waves",
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
        println!("║   1-7: Effects | 0: Auto | D: Toggle Debug | Space: Pause   ║");
        println!("║   +/-: Volume | S: Show Sync Info | ESC: Exit               ║");
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
#[command(name = "arrvee-sync-test")]
#[command(about = "Arrvee Music Visualizer - Synchronized Playback Test")]
struct Args {
    /// Audio file to visualize (WAV, MP3, OGG)
    #[arg(default_value = "sample.m4a")]
    audio_file: String,

    /// ARV prescan data file
    #[arg(short, long, default_value = "sample_prescan.arv")]
    arv_file: String,

    /// Show developer overlay with analysis stats
    #[arg(long, short)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Starting Synchronized Audio Visualization Test");
    info!("Audio file: {}", args.audio_file);
    info!("ARV data: {}", args.arv_file);
    info!("Debug overlay: {}", args.debug);

    // Load synchronized playback data
    info!("Loading ARV prescan data...");
    let prescan_data = ArvFormat::load_arv(&args.arv_file)?;
    let mut synchronized_playback = SynchronizedPlayback::new(prescan_data);

    info!("Loaded synchronized data:");
    info!("  Duration: {:.1}s", synchronized_playback.get_file_info().duration_seconds);
    info!("  Frames: {} analysis points", synchronized_playback.get_file_info().total_samples / synchronized_playback.get_file_info().chunk_size);
    info!("  BPM: {:.1}", synchronized_playback.get_statistics().average_bpm);
    info!("  Profile: {} energy, {} frequency balance",
          synchronized_playback.get_statistics().energy_profile,
          synchronized_playback.get_statistics().dominant_frequency_range);

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Arrvee Synchronized Playback Test")
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

    let mut paused = false;
    let mut playback_start_time = Instant::now();

    // Load and start playing the specified audio file
    info!("Loading {}...", args.audio_file);
    audio_playback.load_file(&args.audio_file).await?;

    // Set initial volume
    let initial_volume = if let Some(debug) = &debug_overlay {
        debug.volume_control
    } else {
        0.1
    };
    audio_playback.set_volume(initial_volume);

    audio_playback.play();
    info!("Audio playback started at {:.0}% volume with synchronized analysis", initial_volume * 100.0);

    info!("Synchronized visualization test initialized successfully");

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
                                if paused {
                                    audio_playback.play();
                                    playback_start_time = Instant::now() - playback_start_time.elapsed();
                                    paused = false;
                                    info!("Audio resumed");
                                } else {
                                    audio_playback.pause();
                                    paused = true;
                                    info!("Audio paused");
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
                            PhysicalKey::Code(KeyCode::Digit7) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(Some("parametric_waves".to_string()));
                                info!("🌈 Effect switched to: Parametric Waves");
                            }
                            PhysicalKey::Code(KeyCode::Digit0) => {
                                graphics_engine.psychedelic_manager_mut().set_manual_effect(None);
                                info!("🌈 Effect switched to: Auto-Blend Mode");
                            }
                            // Projection controls
                            PhysicalKey::Code(KeyCode::KeyQ) => {
                                graphics_engine.projection_mode = -1.0;
                                info!("📐 Projection: Auto");
                            }
                            PhysicalKey::Code(KeyCode::KeyW) => {
                                graphics_engine.projection_mode = 0.0;
                                info!("📐 Projection: Spheres");
                            }
                            PhysicalKey::Code(KeyCode::KeyE) => {
                                graphics_engine.projection_mode = 1.0;
                                info!("📐 Projection: Cylinder");
                            }
                            PhysicalKey::Code(KeyCode::KeyR) => {
                                graphics_engine.projection_mode = 2.0;
                                info!("📐 Projection: Torus");
                            }
                            PhysicalKey::Code(KeyCode::KeyT) => {
                                graphics_engine.projection_mode = 3.0;
                                info!("📐 Projection: Flat");
                            }
                            // Palette switching
                            PhysicalKey::Code(KeyCode::KeyP) => {
                                graphics_engine.palette_index = (graphics_engine.palette_index + 1.0) % 6.0;
                                let palette_names = ["Rainbow", "Neon Cyber", "Warm Sunset", "Deep Ocean", "Purple Haze", "Electric Green"];
                                let palette_name = palette_names[graphics_engine.palette_index as usize];
                                info!("🎨 Palette: {}", palette_name);
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

                    // Get current playback time and synchronized frame
                    let current_time = if paused {
                        playback_start_time.elapsed().as_secs_f32()
                    } else {
                        playback_start_time.elapsed().as_secs_f32()
                    };

                    let file_info_sample_rate = synchronized_playback.get_file_info().sample_rate;
                    let _sync_info = if let Some(sync_frame) = synchronized_playback.get_synchronized_frame(current_time) {
                        // Convert prescan frame to AudioFrame for rendering
                        let audio_data = AudioFrame {
                            sample_rate: file_info_sample_rate,
                            spectrum: vec![0.0; 512], // Not used in rendering
                            time_domain: vec![0.0; 1024], // Not used in rendering
                            frequency_bands: sync_frame.frequency_bands.clone(),
                            beat_detected: sync_frame.beat_detected,
                            beat_strength: sync_frame.beat_strength,
                            volume: sync_frame.volume,
                            spectral_centroid: sync_frame.spectral_centroid,
                            spectral_rolloff: sync_frame.spectral_rolloff,
                            zero_crossing_rate: sync_frame.zero_crossing_rate,
                            spectral_flux: sync_frame.spectral_flux,
                            onset_strength: sync_frame.onset_strength,
                            pitch_confidence: sync_frame.pitch_confidence,
                            estimated_bpm: sync_frame.estimated_bpm,
                            dynamic_range: sync_frame.dynamic_range,
                        };

                        let sync_status = format!("T={:.2}s Frame@{:.3}s Perfect", current_time, sync_frame.timestamp);

                        // Render debug overlay if enabled
                        static mut FRAME_COUNT: u32 = 0;
                        unsafe {
                            FRAME_COUNT += 1;
                            if FRAME_COUNT % 30 == 0 {
                                if let Some(debug) = &mut debug_overlay {
                                    debug.render_debug_info(&audio_data, &graphics_engine, &sync_status);
                                }
                            }
                        }

                        if let Err(e) = graphics_engine.render(&audio_data, &window_clone) {
                            log::error!("Render error: {}", e);
                        }

                        sync_status
                    } else {
                        let sync_status = format!("T={:.2}s No sync data", current_time);

                        // Use default frame when out of sync
                        let default_frame = AudioFrame::default();
                        if let Err(e) = graphics_engine.render(&default_frame, &window_clone) {
                            log::error!("Render error: {}", e);
                        }

                        sync_status
                    };

                    // Check if audio finished
                    if audio_playback.is_finished() || current_time > synchronized_playback.get_file_info().duration_seconds {
                        info!("Synchronized playback finished");
                        elwt.exit();
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}