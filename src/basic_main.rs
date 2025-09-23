use anyhow::Result;
use log::info;

mod audio;

use audio::AudioProcessor;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Basic Arrvee Music Visualizer");

    let mut audio_processor = AudioProcessor::new()?;

    info!("Basic visualizer initialized successfully");
    info!("Processing audio and displaying spectrum data...");

    loop {
        let audio_frame = audio_processor.get_latest_frame();

        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[1;1H");

        println!("🎵 Arrvee Music Visualizer - Live Audio Analysis 🎵");
        println!("================================================");
        println!();

        // Display frequency bands
        println!("📊 Frequency Bands:");
        println!("Sub-Bass (0-60Hz):   {:#>width$} {:.3}",
                 "█".repeat((audio_frame.frequency_bands.sub_bass * 20.0) as usize),
                 audio_frame.frequency_bands.sub_bass, width = 10);
        println!("Bass (60-250Hz):     {:#>width$} {:.3}",
                 "█".repeat((audio_frame.frequency_bands.bass * 20.0) as usize),
                 audio_frame.frequency_bands.bass, width = 10);
        println!("Mid (250-2kHz):      {:#>width$} {:.3}",
                 "█".repeat((audio_frame.frequency_bands.mid * 20.0) as usize),
                 audio_frame.frequency_bands.mid, width = 10);
        println!("Treble (2k-8kHz):    {:#>width$} {:.3}",
                 "█".repeat((audio_frame.frequency_bands.treble * 20.0) as usize),
                 audio_frame.frequency_bands.treble, width = 10);
        println!("Presence (8kHz+):    {:#>width$} {:.3}",
                 "█".repeat((audio_frame.frequency_bands.presence * 20.0) as usize),
                 audio_frame.frequency_bands.presence, width = 10);

        println!();

        // Beat detection
        let beat_indicator = if audio_frame.beat_detected { "🥁 BEAT!" } else { "     " };
        println!("🎯 Beat Detection: {} (Strength: {:.2})", beat_indicator, audio_frame.beat_strength);

        // Volume
        let volume_bars = "█".repeat((audio_frame.volume * 30.0) as usize);
        println!("🔊 Volume:         {:#>width$} {:.3}", volume_bars, audio_frame.volume, width = 15);

        println!();

        // Spectrum visualization (first 16 bins)
        println!("📈 Frequency Spectrum (Sample):");
        for (i, &magnitude) in audio_frame.spectrum.iter().enumerate().take(16) {
            let freq = (i as f32 * audio_frame.sample_rate) / audio_frame.spectrum.len() as f32;
            let bar_length = (magnitude * 30.0) as usize;
            let bar = "▓".repeat(bar_length);
            println!("{:6.0}Hz │{:<30} │ {:.3}", freq, bar, magnitude);
        }

        println!();
        println!("Press Ctrl+C to exit");

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}