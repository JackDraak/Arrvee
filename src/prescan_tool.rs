use anyhow::Result;
use clap::Parser;
use log::info;

mod audio;
use audio::{PrescanProcessor, ArvFormat};

#[derive(Parser)]
#[command(name = "arrvee-prescan")]
#[command(about = "Pre-scan audio files for real-time synchronized visualization")]
struct Args {
    /// Audio file to pre-scan (MP3, WAV, M4A, OGG, etc.)
    #[arg()]
    input_file: String,

    /// Output file for prescan data (JSON or ARV format)
    #[arg(short, long, default_value = "prescan_data.arv")]
    output: String,

    /// Output format: 'arv' (binary) or 'json' (text)
    #[arg(long, default_value = "arv")]
    format: String,

    /// Sample rate for analysis
    #[arg(long, default_value = "44100")]
    sample_rate: u32,

    /// Analysis chunk size (smaller = more precise, larger = faster)
    #[arg(long, default_value = "512")]
    chunk_size: usize,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Arrvee Pre-scan Tool");
    info!("Input file: {}", args.input_file);
    info!("Output file: {}", args.output);
    info!("Sample rate: {}Hz, Chunk size: {}", args.sample_rate, args.chunk_size);

    // Create prescan processor
    let mut processor = PrescanProcessor::new(args.sample_rate as f32, args.chunk_size);

    // Pre-scan the audio file
    info!("Starting pre-scan analysis...");
    let prescan_data = processor.prescan_file(&args.input_file)?;

    // Display statistics
    info!("\n=== PRE-SCAN RESULTS ===");
    info!("Duration: {:.2} seconds", prescan_data.file_info.duration_seconds);
    info!("Total frames: {}", prescan_data.frames.len());
    info!("Frame rate: {:.2} Hz", prescan_data.file_info.frame_rate);
    info!("Total beats detected: {}", prescan_data.statistics.total_beats);
    info!("Average BPM: {:.1}", prescan_data.statistics.average_bpm);
    info!("BPM range: {:.1} - {:.1}",
          prescan_data.statistics.bpm_range.0,
          prescan_data.statistics.bpm_range.1);
    info!("Dominant frequency range: {}", prescan_data.statistics.dominant_frequency_range);
    info!("Energy profile: {}", prescan_data.statistics.energy_profile);
    info!("Complexity score: {:.3}", prescan_data.statistics.complexity_score);

    // Peak values for calibration
    info!("\n=== PEAK VALUES (for calibration) ===");
    info!("Peak bass: {:.6}", prescan_data.statistics.peak_bass);
    info!("Peak mid: {:.6}", prescan_data.statistics.peak_mid);
    info!("Peak treble: {:.6}", prescan_data.statistics.peak_treble);
    info!("Peak presence: {:.6}", prescan_data.statistics.peak_presence);
    info!("Peak volume: {:.6}", prescan_data.statistics.peak_volume);
    info!("Peak spectral flux: {:.6}", prescan_data.statistics.peak_spectral_flux);
    info!("Peak onset: {:.6}", prescan_data.statistics.peak_onset);

    // Save results in requested format
    info!("Saving prescan data to: {} (format: {})", args.output, args.format);

    let file_size = if args.format.to_lowercase() == "arv" {
        ArvFormat::save_arv(&prescan_data, &args.output)?;
        std::fs::metadata(&args.output)?.len()
    } else {
        PrescanProcessor::save_prescan_data(&prescan_data, &args.output)?;
        std::fs::metadata(&args.output)?.len()
    };

    info!("Prescan data saved successfully ({:.1} KB)", file_size as f64 / 1024.0);

    // Show compression ratio if ARV format
    if args.format.to_lowercase() == "arv" && std::path::Path::new("sample_prescan.json").exists() {
        let json_size = std::fs::metadata("sample_prescan.json")?.len();
        let compression = ArvFormat::compression_ratio(file_size, json_size);
        info!("Compression ratio: {:.1}% smaller than JSON", compression * 100.0);
    }

    info!("\n✅ Pre-scan complete! You can now use this data for perfectly synchronized real-time visualization.");
    info!("💡 Tip: Use the synchronized playback mode in the visualizer for authentic real-time responsiveness.");

    Ok(())
}