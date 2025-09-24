use anyhow::Result;
use clap::Parser;
use log::info;

mod audio;
mod graphics;
mod effects;
use audio::{
    PrescanProcessor, ArvFormat,
    AudioAnalyzer, CpuAudioAnalyzer, NewGpuAudioAnalyzer,
    FeatureNormalizer, RawAudioFeatures, NormalizedAudioFeatures
};

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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Arrvee Pre-scan Tool");
    info!("Input file: {}", args.input_file);
    info!("Output file: {}", args.output);
    info!("Sample rate: {}Hz, Chunk size: {}", args.sample_rate, args.chunk_size);

    // Pre-scan the audio file using unified architecture
    info!("Starting pre-scan analysis...");
    let prescan_data = prescan_with_unified_architecture(&args).await?;

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

/// Unified prescan function using transparent GPU-first with CPU fallback architecture
/// Automatically tries GPU acceleration, falls back to CPU if unavailable
async fn prescan_with_unified_architecture(args: &Args) -> Result<audio::PrescanData> {
    use audio::prescan::{PrescanFrame, FileInfo, AnalysisStatistics};
    use audio::{FrequencyBands, FeatureNormalizer};
    use rodio::{Decoder, Source};
    use std::fs::File;
    use std::io::BufReader;

    info!("Loading audio file...");

    // Load audio file
    let file = BufReader::new(File::open(&args.input_file)?);
    let source = Decoder::new(file)?;
    let channels = source.channels();
    let samples: Vec<i16> = source.convert_samples().collect();

    // Convert to f32 and mix to mono
    let audio_buffer: Vec<f32> = samples
        .chunks_exact(channels as usize)
        .map(|chunk| {
            let sum: f32 = chunk.iter().map(|&s| s as f32 / 32768.0).sum();
            sum / channels as f32
        })
        .collect();

    let total_samples = audio_buffer.len();
    let duration_seconds = total_samples as f32 / args.sample_rate as f32;
    let frame_rate = args.sample_rate as f32 / args.chunk_size as f32;

    info!("Loaded {} samples ({:.2}s) for analysis", total_samples, duration_seconds);

    // Try GPU first, fall back to CPU automatically
    let mut analyzer: Box<dyn AudioAnalyzer + Send> = {
        info!("Attempting GPU initialization...");
        match NewGpuAudioAnalyzer::new_standalone(args.sample_rate as f32, args.chunk_size).await {
            Ok(gpu_analyzer) => {
                info!("✅ GPU analyzer initialized successfully");
                Box::new(gpu_analyzer)
            }
            Err(e) => {
                info!("⚠️  GPU initialization failed: {}. Falling back to CPU.", e);
                Box::new(CpuAudioAnalyzer::new(args.sample_rate as f32, args.chunk_size)?)
            }
        }
    };

    info!("Using {} analyzer", analyzer.analyzer_type());

    // Initialize feature normalizer
    let mut normalizer = FeatureNormalizer::new();

    // Process entire file chunk by chunk
    let mut frames = Vec::new();
    let mut statistics = AnalysisStatistics::default();
    let mut sample_pos = 0;
    let mut beat_count = 0u32;
    let mut bpm_values = Vec::new();

    while sample_pos + args.chunk_size <= total_samples {
        let chunk = &audio_buffer[sample_pos..sample_pos + args.chunk_size];
        let raw_features = analyzer.analyze_chunk(chunk).await?;
        let normalized_features = normalizer.normalize(&raw_features);
        let timestamp = sample_pos as f32 / args.sample_rate as f32;

        // Convert to PrescanFrame using normalized features
        let prescan_frame = PrescanFrame {
            timestamp,
            frequency_bands: FrequencyBands {
                sub_bass: normalized_features.sub_bass,
                bass: normalized_features.bass,
                mid: normalized_features.mid,
                treble: normalized_features.treble,
                presence: normalized_features.presence,
            },
            beat_detected: normalized_features.beat_strength > 0.3,
            beat_strength: normalized_features.beat_strength,
            estimated_bpm: normalized_features.estimated_bpm,
            spectral_centroid: normalized_features.spectral_centroid,
            spectral_rolloff: normalized_features.spectral_rolloff,
            pitch_confidence: normalized_features.pitch_confidence,
            zero_crossing_rate: normalized_features.zero_crossing_rate,
            spectral_flux: normalized_features.spectral_flux,
            onset_strength: normalized_features.onset_strength,
            dynamic_range: normalized_features.dynamic_range,
            volume: normalized_features.volume,
        };

        // Update statistics using normalized features
        update_unified_statistics(&mut statistics, &normalized_features, &mut beat_count, &mut bpm_values);

        frames.push(prescan_frame);
        sample_pos += args.chunk_size;

        if frames.len() % 1000 == 0 {
            info!("Pre-scanned {} frames ({:.1}s of {:.1}s)",
                  frames.len(), timestamp, duration_seconds);
        }
    }

    // Finalize statistics
    statistics.total_beats = beat_count;
    if !bpm_values.is_empty() {
        statistics.average_bpm = bpm_values.iter().sum::<f32>() / bpm_values.len() as f32;
        statistics.bpm_range = (
            bpm_values.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
            bpm_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        );
    }

    // Classify content
    classify_unified_content(&mut statistics, &frames);

    info!("{} analysis complete: {} frames, {} beats, {:.1} BPM average",
          analyzer.analyzer_type(), frames.len(), beat_count, statistics.average_bpm);

    Ok(audio::PrescanData {
        file_info: FileInfo {
            filename: args.input_file.clone(),
            duration_seconds,
            sample_rate: args.sample_rate as f32,
            total_samples,
            frame_rate,
            chunk_size: args.chunk_size,
        },
        frames,
        statistics,
    })
}

fn update_unified_statistics(
    stats: &mut audio::prescan::AnalysisStatistics,
    features: &NormalizedAudioFeatures,
    beat_count: &mut u32,
    bpm_values: &mut Vec<f32>,
) {
    // Update peak values (normalized features are 0.0-1.0)
    stats.peak_bass = stats.peak_bass.max(features.bass);
    stats.peak_mid = stats.peak_mid.max(features.mid);
    stats.peak_treble = stats.peak_treble.max(features.treble);
    stats.peak_presence = stats.peak_presence.max(features.presence);
    stats.peak_volume = stats.peak_volume.max(features.volume);
    stats.peak_spectral_flux = stats.peak_spectral_flux.max(features.spectral_flux);
    stats.peak_onset = stats.peak_onset.max(features.onset_strength);

    // Track beats and BPM
    if features.beat_strength > 0.3 {
        *beat_count += 1;
        if features.estimated_bpm > 60.0 && features.estimated_bpm < 200.0 {
            bpm_values.push(features.estimated_bpm);
        }
    }
}

fn classify_unified_content(stats: &mut audio::prescan::AnalysisStatistics, frames: &[audio::prescan::PrescanFrame]) {
    // Determine dominant frequency range
    let avg_bass: f32 = frames.iter().map(|f| f.frequency_bands.bass).sum::<f32>() / frames.len() as f32;
    let avg_mid: f32 = frames.iter().map(|f| f.frequency_bands.mid).sum::<f32>() / frames.len() as f32;
    let avg_treble: f32 = frames.iter().map(|f| f.frequency_bands.treble).sum::<f32>() / frames.len() as f32;

    stats.dominant_frequency_range = if avg_bass > avg_mid && avg_bass > avg_treble {
        "Bass-Heavy".to_string()
    } else if avg_treble > avg_bass && avg_treble > avg_mid {
        "Treble-Focused".to_string()
    } else {
        "Balanced".to_string()
    };

    // Determine energy profile
    let avg_volume: f32 = frames.iter().map(|f| f.volume).sum::<f32>() / frames.len() as f32;
    let volume_variance: f32 = frames.iter()
        .map(|f| (f.volume - avg_volume).powi(2))
        .sum::<f32>() / frames.len() as f32;

    stats.energy_profile = if volume_variance > 0.1 {
        "Dynamic".to_string()
    } else if avg_volume > 0.3 {
        "High".to_string()
    } else if avg_volume > 0.1 {
        "Medium".to_string()
    } else {
        "Low".to_string()
    };

    // Calculate complexity score (0.0-1.0)
    let spectral_complexity = frames.iter().map(|f| f.spectral_flux).sum::<f32>() / frames.len() as f32;
    let harmonic_complexity = frames.iter().map(|f| f.pitch_confidence).sum::<f32>() / frames.len() as f32;
    stats.complexity_score = (spectral_complexity + harmonic_complexity + volume_variance).min(1.0);
}