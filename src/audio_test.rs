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

use graphics::GraphicsEngine;
use audio::{AudioPlayback, AudioFrame};

struct DebugOverlay {
    show_overlay: bool,
    volume_control: f32,
}

impl DebugOverlay {
    fn new() -> Self {
        Self {
            show_overlay: true,
            volume_control: 1.0,
        }
    }

    fn render_debug_info(&mut self, audio_frame: &AudioFrame) {
        // For now, we'll just print to console (in a real implementation, this would render UI)
        if self.show_overlay {
            println!("\n=== ARRVEE AUDIO ANALYSIS DEBUG ===");
            println!("🎵 FREQUENCY BANDS:");
            println!("  Sub-Bass: {:.3}", audio_frame.frequency_bands.sub_bass);
            println!("  Bass:     {:.3}", audio_frame.frequency_bands.bass);
            println!("  Mid:      {:.3}", audio_frame.frequency_bands.mid);
            println!("  Treble:   {:.3}", audio_frame.frequency_bands.treble);
            println!("  Presence: {:.3}", audio_frame.frequency_bands.presence);

            println!("🥁 RHYTHM ANALYSIS:");
            println!("  Beat Detected: {}", if audio_frame.beat_detected { "YES" } else { "no" });
            println!("  Beat Strength: {:.3}", audio_frame.beat_strength);
            println!("  Estimated BPM: {:.1}", audio_frame.estimated_bpm);

            println!("🎶 SPECTRAL FEATURES:");
            println!("  Centroid (Brightness): {:.1} Hz", audio_frame.spectral_centroid);
            println!("  Rolloff (High Freq):   {:.1} Hz", audio_frame.spectral_rolloff);
            println!("  Pitch Confidence:      {:.3}", audio_frame.pitch_confidence);
            println!("  Zero Crossing Rate:    {:.3}", audio_frame.zero_crossing_rate);

            println!("🌊 DYNAMIC FEATURES:");
            println!("  Volume (RMS):      {:.3}", audio_frame.volume);
            println!("  Dynamic Range:     {:.3}", audio_frame.dynamic_range);
            println!("  Spectral Flux:     {:.3}", audio_frame.spectral_flux);
            println!("  Onset Strength:    {:.3}", audio_frame.onset_strength);

            println!("🎛️  CONTROLS:");
            println!("  Volume Control: {:.1}%", self.volume_control * 100.0);
            println!("=====================================");
        }
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
                    info!("Close requested");
                    audio_playback.stop();
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event,
                    ..
                } => {
                    if event.state == ElementState::Pressed {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Escape) => {
                                info!("Escape pressed");
                                audio_playback.stop();
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
                            _ => {}
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    graphics_engine.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Get real-time audio analysis from the loaded file
                    let audio_data = audio_playback.get_current_audio_frame();

                    // Render debug overlay if enabled (limit to ~2Hz to avoid spam)
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 30 == 0 { // Show debug every 30 frames (~2Hz at 60fps)
                            if let Some(debug) = &mut debug_overlay {
                                debug.render_debug_info(&audio_data);
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