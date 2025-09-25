use anyhow::Result;
use clap::Parser;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

mod audio;
mod effects;

use audio::{AudioPlayback, AudioFrame, CpuAudioAnalyzer, NewGpuAudioAnalyzer, FeatureNormalizer, NormalizedAudioFeatures};
use audio::analysis_interface::AudioAnalyzer;
use effects::PsychedelicManager;

#[derive(Parser)]
#[command(name = "arrvee-audio-analyzer")]
#[command(about = "Comprehensive Audio Analysis Tool - Generates detailed statistics and frame-by-frame logs")]
struct Args {
    /// Audio file to analyze (WAV, MP3, OGG, M4A)
    #[arg(default_value = "sample.m4a")]
    audio_file: String,

    /// Output JSON file path
    #[arg(long, short, default_value = "analysis_results.json")]
    output: String,

    /// Include frame-by-frame data (creates large files but useful for fine-tuning)
    #[arg(long)]
    frame_by_frame: bool,

    /// Analysis chunk size in samples
    #[arg(long, default_value = "512")]
    chunk_size: usize,

    /// Sample rate override (0 = use file's native rate)
    #[arg(long, default_value = "0")]
    sample_rate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AudioFeatureStats {
    min: f32,
    max: f32,
    mean: f32,
    median: f32,
    std_dev: f32,
    samples: usize,
    histogram: Vec<(f32, usize)>, // (value_range, count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameData {
    timestamp: f32,
    audio_frame: SerializableAudioFrame,
    effect_weights: HashMap<String, f32>,
    dominant_effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableAudioFrame {
    // Frequency bands
    sub_bass: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    presence: f32,

    // Beat and rhythm
    beat_detected: bool,
    beat_strength: f32,
    estimated_bpm: f32,
    volume: f32,

    // Spectral characteristics
    spectral_centroid: f32,
    spectral_rolloff: f32,
    pitch_confidence: f32,

    // Temporal dynamics
    zero_crossing_rate: f32,
    spectral_flux: f32,
    onset_strength: f32,
    dynamic_range: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BeatEvent {
    timestamp: f32,
    strength: f32,
    estimated_bpm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EffectActivation {
    effect_name: String,
    start_time: f32,
    end_time: f32,
    peak_weight: f32,
    average_weight: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResults {
    // Metadata
    file_info: FileInfo,
    analysis_config: AnalysisConfig,

    // Overall statistics
    frequency_band_stats: HashMap<String, AudioFeatureStats>,
    spectral_feature_stats: HashMap<String, AudioFeatureStats>,
    temporal_feature_stats: HashMap<String, AudioFeatureStats>,
    beat_stats: BeatStats,

    // Effect analysis
    effect_activation_summary: HashMap<String, EffectActivationSummary>,
    effect_transitions: Vec<EffectTransition>,

    // Event logs
    beat_events: Vec<BeatEvent>,
    effect_activations: Vec<EffectActivation>,

    // Frame-by-frame data (optional)
    frame_data: Option<Vec<FrameData>>,

    // Analysis insights
    insights: AnalysisInsights,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    filename: String,
    duration_seconds: f32,
    sample_rate: f32,
    total_samples: usize,
    total_frames: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisConfig {
    chunk_size: usize,
    frame_rate: f32,
    include_frame_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BeatStats {
    total_beats: usize,
    average_bpm: f32,
    bpm_variance: f32,
    beat_consistency: f32, // 0-1, how consistent beat timing is
    strongest_beat: f32,
    weakest_beat: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct EffectActivationSummary {
    total_activation_time: f32,
    activation_percentage: f32,
    peak_weight: f32,
    average_weight: f32,
    activation_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct EffectTransition {
    timestamp: f32,
    from_effect: Option<String>,
    to_effect: String,
    transition_speed: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisInsights {
    dominant_frequency_range: String,
    music_complexity: f32, // 0-1 scale
    rhythmic_consistency: f32, // 0-1 scale
    harmonic_content: f32, // 0-1 scale
    recommended_effects: Vec<String>,
    optimal_smoothing_factor: f32,
    suggested_thresholds: HashMap<String, f32>,
}

impl From<&AudioFrame> for SerializableAudioFrame {
    fn from(frame: &AudioFrame) -> Self {
        Self {
            sub_bass: frame.frequency_bands.sub_bass,
            bass: frame.frequency_bands.bass,
            mid: frame.frequency_bands.mid,
            treble: frame.frequency_bands.treble,
            presence: frame.frequency_bands.presence,
            beat_detected: frame.beat_detected,
            beat_strength: frame.beat_strength,
            estimated_bpm: frame.estimated_bpm,
            volume: frame.volume,
            spectral_centroid: frame.spectral_centroid,
            spectral_rolloff: frame.spectral_rolloff,
            pitch_confidence: frame.pitch_confidence,
            zero_crossing_rate: frame.zero_crossing_rate,
            spectral_flux: frame.spectral_flux,
            onset_strength: frame.onset_strength,
            dynamic_range: frame.dynamic_range,
        }
    }
}

struct AudioAnalysisEngine {
    playback: AudioPlayback,
    analyzer: Box<dyn AudioAnalyzer + Send>,
    normalizer: FeatureNormalizer,
    psychedelic_manager: PsychedelicManager,

    // Statistics collectors
    feature_collectors: HashMap<String, Vec<f32>>,
    frame_data: Vec<FrameData>,
    beat_events: Vec<BeatEvent>,
    effect_activations: Vec<EffectActivation>,

    // Configuration
    chunk_size: usize,
    sample_rate: f32,
    frame_rate: f32,
}

impl AudioAnalysisEngine {
    async fn new(chunk_size: usize, sample_rate: f32) -> Result<Self> {
        let playback = AudioPlayback::new()?;

        // Try GPU first, fallback to CPU
        let analyzer: Box<dyn AudioAnalyzer + Send> = match NewGpuAudioAnalyzer::new_standalone(sample_rate, chunk_size).await {
            Ok(gpu_analyzer) => {
                info!("Using GPU analyzer");
                Box::new(gpu_analyzer)
            }
            Err(e) => {
                info!("GPU analyzer failed ({}), using CPU analyzer", e);
                Box::new(CpuAudioAnalyzer::new(sample_rate, chunk_size)?)
            }
        };

        let normalizer = FeatureNormalizer::new();
        let psychedelic_manager = PsychedelicManager::new();

        let frame_rate = sample_rate / chunk_size as f32; // Approximate frame rate

        Ok(Self {
            playback,
            analyzer,
            normalizer,
            psychedelic_manager,
            feature_collectors: HashMap::new(),
            frame_data: Vec::new(),
            beat_events: Vec::new(),
            effect_activations: Vec::new(),
            chunk_size,
            sample_rate,
            frame_rate,
        })
    }

    async fn analyze_file(&mut self, file_path: &str, include_frames: bool) -> Result<AnalysisResults> {
        info!("Loading audio file: {}", file_path);
        self.playback.load_file(file_path)?;

        let mut frame_count = 0;
        let mut active_effects: HashMap<String, f32> = HashMap::new(); // track when effects start

        info!("Starting comprehensive audio analysis...");

        // Get the entire audio buffer for sequential processing
        let audio_buffer = self.playback.get_full_audio_buffer().clone();
        let total_samples = audio_buffer.len();
        let total_duration = total_samples as f32 / self.sample_rate;
        info!("Processing {} samples ({:.2}s duration)", total_samples, total_duration);

        // Process the entire file chunk by chunk
        let mut sample_pos = 0;
        while sample_pos + self.chunk_size <= total_samples {
            let chunk = &audio_buffer[sample_pos..sample_pos + self.chunk_size];

            // Get raw features from analyzer
            let raw_features = self.analyzer.analyze_chunk(chunk).await?;
            let normalized_features = self.normalizer.normalize(&raw_features);
            let audio_frame = self.convert_to_audio_frame(&normalized_features);

            let timestamp = sample_pos as f32 / self.sample_rate;

            // Update psychedelic manager
            self.psychedelic_manager.update(1.0 / self.frame_rate, &audio_frame);
            let effect_weights = self.psychedelic_manager.get_effect_weights().clone();

            // Collect statistics
            self.collect_frame_statistics(&audio_frame, timestamp, &effect_weights);

            // Track effect activations
            self.track_effect_activations(timestamp, &effect_weights, &mut active_effects);

            // Collect frame data if requested
            if include_frames {
                let dominant_effect = effect_weights.iter()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .filter(|(_, weight)| **weight > 0.1)
                    .map(|(name, _)| name.clone());

                self.frame_data.push(FrameData {
                    timestamp,
                    audio_frame: SerializableAudioFrame::from(&audio_frame),
                    effect_weights: effect_weights.clone(),
                    dominant_effect,
                });
            }

            frame_count += 1;
            sample_pos += self.chunk_size;

            if frame_count % 1000 == 0 {
                info!("Processed {} frames ({:.1}s of {:.1}s)", frame_count, timestamp, total_duration);
            }
        }

        info!("Analysis complete. Processed {} frames ({:.2}s)", frame_count, total_duration);

        // Generate comprehensive results
        self.generate_results(file_path, frame_count, include_frames)
    }

    fn collect_frame_statistics(&mut self, frame: &AudioFrame, timestamp: f32, _effect_weights: &HashMap<String, f32>) {
        // Collect frequency band data
        self.add_sample("sub_bass", frame.frequency_bands.sub_bass);
        self.add_sample("bass", frame.frequency_bands.bass);
        self.add_sample("mid", frame.frequency_bands.mid);
        self.add_sample("treble", frame.frequency_bands.treble);
        self.add_sample("presence", frame.frequency_bands.presence);

        // Collect spectral features
        self.add_sample("spectral_centroid", frame.spectral_centroid);
        self.add_sample("spectral_rolloff", frame.spectral_rolloff);
        self.add_sample("pitch_confidence", frame.pitch_confidence);

        // Collect temporal features
        self.add_sample("zero_crossing_rate", frame.zero_crossing_rate);
        self.add_sample("spectral_flux", frame.spectral_flux);
        self.add_sample("onset_strength", frame.onset_strength);
        self.add_sample("dynamic_range", frame.dynamic_range);

        // Collect beat/rhythm data
        self.add_sample("beat_strength", frame.beat_strength);
        self.add_sample("estimated_bpm", frame.estimated_bpm);
        self.add_sample("volume", frame.volume);

        // Track beat events
        if frame.beat_detected || frame.beat_strength > 0.5 {
            self.beat_events.push(BeatEvent {
                timestamp,
                strength: frame.beat_strength,
                estimated_bpm: frame.estimated_bpm,
            });
        }
    }

    fn add_sample(&mut self, feature_name: &str, value: f32) {
        self.feature_collectors
            .entry(feature_name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    fn track_effect_activations(&mut self, timestamp: f32, effect_weights: &HashMap<String, f32>, active_effects: &mut HashMap<String, f32>) {
        for (effect_name, &weight) in effect_weights {
            if weight > 0.1 {
                // Effect is active
                if !active_effects.contains_key(effect_name) {
                    // Effect just started
                    active_effects.insert(effect_name.clone(), timestamp);
                }
            } else if let Some(start_time) = active_effects.remove(effect_name) {
                // Effect just ended
                let duration = timestamp - start_time;
                if duration > 0.1 { // Only record activations longer than 100ms
                    self.effect_activations.push(EffectActivation {
                        effect_name: effect_name.clone(),
                        start_time,
                        end_time: timestamp,
                        peak_weight: weight,
                        average_weight: weight * 0.7, // Approximate
                    });
                }
            }
        }
    }

    fn convert_to_audio_frame(&self, normalized: &NormalizedAudioFeatures) -> AudioFrame {
        // Convert normalized features back to AudioFrame format for compatibility
        use audio::FrequencyBands;

        AudioFrame {
            sample_rate: self.sample_rate,
            spectrum: Vec::new(), // Not used in current analysis
            time_domain: Vec::new(), // Not used in current analysis
            frequency_bands: FrequencyBands {
                sub_bass: normalized.sub_bass,
                bass: normalized.bass,
                mid: normalized.mid,
                treble: normalized.treble,
                presence: normalized.presence,
            },
            beat_detected: normalized.beat_detected,
            beat_strength: normalized.beat_strength,
            estimated_bpm: normalized.estimated_bpm,
            volume: normalized.volume,
            spectral_centroid: normalized.spectral_centroid,
            spectral_rolloff: normalized.spectral_rolloff,
            pitch_confidence: normalized.pitch_confidence,
            zero_crossing_rate: normalized.zero_crossing_rate,
            spectral_flux: normalized.spectral_flux,
            onset_strength: normalized.onset_strength,
            dynamic_range: normalized.dynamic_range,
        }
    }

    fn generate_results(&self, file_path: &str, frame_count: usize, include_frames: bool) -> Result<AnalysisResults> {
        let duration = frame_count as f32 / self.frame_rate;

        // Generate file info
        let file_info = FileInfo {
            filename: file_path.to_string(),
            duration_seconds: duration,
            sample_rate: self.sample_rate,
            total_samples: frame_count * self.chunk_size,
            total_frames: frame_count,
        };

        // Generate analysis config
        let analysis_config = AnalysisConfig {
            chunk_size: self.chunk_size,
            frame_rate: self.frame_rate,
            include_frame_data: include_frames,
        };

        // Calculate statistics for all features
        let mut frequency_band_stats = HashMap::new();
        let mut spectral_feature_stats = HashMap::new();
        let mut temporal_feature_stats = HashMap::new();

        // Frequency bands
        for band in ["sub_bass", "bass", "mid", "treble", "presence"] {
            if let Some(data) = self.feature_collectors.get(band) {
                frequency_band_stats.insert(band.to_string(), self.calculate_stats(data));
            }
        }

        // Spectral features
        for feature in ["spectral_centroid", "spectral_rolloff", "pitch_confidence"] {
            if let Some(data) = self.feature_collectors.get(feature) {
                spectral_feature_stats.insert(feature.to_string(), self.calculate_stats(data));
            }
        }

        // Temporal features
        for feature in ["zero_crossing_rate", "spectral_flux", "onset_strength", "dynamic_range"] {
            if let Some(data) = self.feature_collectors.get(feature) {
                temporal_feature_stats.insert(feature.to_string(), self.calculate_stats(data));
            }
        }

        // Beat statistics
        let beat_stats = self.calculate_beat_stats();

        // Effect activation analysis
        let effect_activation_summary = self.analyze_effect_activations(duration);

        // Generate insights
        let insights = self.generate_insights(&frequency_band_stats, &spectral_feature_stats, &temporal_feature_stats);

        Ok(AnalysisResults {
            file_info,
            analysis_config,
            frequency_band_stats,
            spectral_feature_stats,
            temporal_feature_stats,
            beat_stats,
            effect_activation_summary,
            effect_transitions: Vec::new(), // TODO: Implement transition analysis
            beat_events: self.beat_events.clone(),
            effect_activations: self.effect_activations.clone(),
            frame_data: if include_frames { Some(self.frame_data.clone()) } else { None },
            insights,
        })
    }

    fn calculate_stats(&self, data: &[f32]) -> AudioFeatureStats {
        if data.is_empty() {
            return AudioFeatureStats {
                min: 0.0, max: 0.0, mean: 0.0, median: 0.0, std_dev: 0.0,
                samples: 0, histogram: Vec::new(),
            };
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted_data[0];
        let max = sorted_data[sorted_data.len() - 1];
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        let median = sorted_data[sorted_data.len() / 2];

        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / data.len() as f32;
        let std_dev = variance.sqrt();

        // Create histogram (20 bins)
        let mut histogram = Vec::new();
        if max > min {
            let bin_size = (max - min) / 20.0;
            for i in 0..20 {
                let bin_start = min + i as f32 * bin_size;
                let bin_end = bin_start + bin_size;
                let count = data.iter()
                    .filter(|&&x| x >= bin_start && x < bin_end)
                    .count();
                histogram.push((bin_start, count));
            }
        }

        AudioFeatureStats {
            min, max, mean, median, std_dev,
            samples: data.len(),
            histogram,
        }
    }

    fn calculate_beat_stats(&self) -> BeatStats {
        if self.beat_events.is_empty() {
            return BeatStats {
                total_beats: 0, average_bpm: 0.0, bpm_variance: 0.0,
                beat_consistency: 0.0, strongest_beat: 0.0, weakest_beat: 0.0,
            };
        }

        let bpms: Vec<f32> = self.beat_events.iter().map(|b| b.estimated_bpm).collect();
        let strengths: Vec<f32> = self.beat_events.iter().map(|b| b.strength).collect();

        let average_bpm = bpms.iter().sum::<f32>() / bpms.len() as f32;
        let bpm_variance = bpms.iter()
            .map(|&bpm| (bpm - average_bpm).powi(2))
            .sum::<f32>() / bpms.len() as f32;

        // Calculate beat consistency (how regular the timing is)
        let mut intervals = Vec::new();
        for i in 1..self.beat_events.len() {
            intervals.push(self.beat_events[i].timestamp - self.beat_events[i-1].timestamp);
        }

        let beat_consistency = if intervals.len() > 1 {
            let mean_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
            let interval_variance = intervals.iter()
                .map(|&interval| (interval - mean_interval).powi(2))
                .sum::<f32>() / intervals.len() as f32;
            // Convert variance to consistency score (0-1, higher is more consistent)
            1.0 / (1.0 + interval_variance)
        } else {
            0.0
        };

        BeatStats {
            total_beats: self.beat_events.len(),
            average_bpm,
            bpm_variance,
            beat_consistency,
            strongest_beat: strengths.iter().fold(0.0f32, |a, &b| a.max(b)),
            weakest_beat: strengths.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
        }
    }

    fn analyze_effect_activations(&self, total_duration: f32) -> HashMap<String, EffectActivationSummary> {
        let mut summary = HashMap::new();

        // Group activations by effect
        let mut effect_groups: HashMap<String, Vec<&EffectActivation>> = HashMap::new();
        for activation in &self.effect_activations {
            effect_groups.entry(activation.effect_name.clone())
                .or_insert_with(Vec::new)
                .push(activation);
        }

        for (effect_name, activations) in effect_groups {
            let total_time: f32 = activations.iter()
                .map(|a| a.end_time - a.start_time)
                .sum();

            let activation_percentage = (total_time / total_duration) * 100.0;
            let peak_weight = activations.iter()
                .map(|a| a.peak_weight)
                .fold(0.0f32, |a, b| a.max(b));

            let average_weight = activations.iter()
                .map(|a| a.average_weight)
                .sum::<f32>() / activations.len() as f32;

            summary.insert(effect_name, EffectActivationSummary {
                total_activation_time: total_time,
                activation_percentage,
                peak_weight,
                average_weight,
                activation_count: activations.len(),
            });
        }

        summary
    }

    fn generate_insights(&self, frequency_stats: &HashMap<String, AudioFeatureStats>, spectral_stats: &HashMap<String, AudioFeatureStats>, temporal_stats: &HashMap<String, AudioFeatureStats>) -> AnalysisInsights {
        // Determine dominant frequency range
        let bass_energy = frequency_stats.get("bass").map(|s| s.mean).unwrap_or(0.0) +
                         frequency_stats.get("sub_bass").map(|s| s.mean).unwrap_or(0.0);
        let mid_energy = frequency_stats.get("mid").map(|s| s.mean).unwrap_or(0.0);
        let treble_energy = frequency_stats.get("treble").map(|s| s.mean).unwrap_or(0.0) +
                           frequency_stats.get("presence").map(|s| s.mean).unwrap_or(0.0);

        let dominant_frequency_range = if bass_energy > mid_energy && bass_energy > treble_energy {
            "Bass-Heavy".to_string()
        } else if treble_energy > bass_energy && treble_energy > mid_energy {
            "Treble-Heavy".to_string()
        } else {
            "Mid-Focused".to_string()
        };

        // Calculate music complexity
        let spectral_flux_var = temporal_stats.get("spectral_flux").map(|s| s.std_dev).unwrap_or(0.0);
        let pitch_confidence_mean = spectral_stats.get("pitch_confidence").map(|s| s.mean).unwrap_or(0.0);
        let music_complexity = (spectral_flux_var * 2.0 + (1.0 - pitch_confidence_mean)).clamp(0.0, 1.0);

        // Calculate rhythmic consistency from beat stats
        let beat_stats = self.calculate_beat_stats();
        let rhythmic_consistency = beat_stats.beat_consistency;

        // Harmonic content
        let harmonic_content = pitch_confidence_mean;

        // Recommend effects based on analysis
        let mut recommended_effects = Vec::new();
        if bass_energy > 0.3 {
            recommended_effects.push("llama_plasma".to_string());
        }
        if pitch_confidence_mean > 0.6 {
            recommended_effects.push("geometric_kaleidoscope".to_string());
            recommended_effects.push("parametric_waves".to_string());
        }
        if music_complexity > 0.5 {
            recommended_effects.push("fractal_madness".to_string());
        }
        if rhythmic_consistency > 0.7 {
            recommended_effects.push("particle_swarm".to_string());
        }

        // Suggest optimal smoothing factor based on dynamics
        let dynamic_range_mean = temporal_stats.get("dynamic_range").map(|s| s.mean).unwrap_or(0.5);
        let optimal_smoothing_factor = (1.0 - dynamic_range_mean * 0.8).clamp(0.1, 1.0);

        // Suggest thresholds
        let mut suggested_thresholds = HashMap::new();
        suggested_thresholds.insert("bass_threshold".to_string(), bass_energy * 0.7);
        suggested_thresholds.insert("beat_threshold".to_string(), beat_stats.strongest_beat * 0.6);
        suggested_thresholds.insert("onset_threshold".to_string(),
            temporal_stats.get("onset_strength").map(|s| s.mean * 1.2).unwrap_or(0.1));

        AnalysisInsights {
            dominant_frequency_range,
            music_complexity,
            rhythmic_consistency,
            harmonic_content,
            recommended_effects,
            optimal_smoothing_factor,
            suggested_thresholds,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("🎵 Starting Comprehensive Audio Analysis");
    info!("File: {}", args.audio_file);
    info!("Output: {}", args.output);
    info!("Frame-by-frame logging: {}", args.frame_by_frame);
    info!("Chunk size: {} samples", args.chunk_size);

    // Determine sample rate
    let sample_rate = if args.sample_rate > 0 {
        args.sample_rate as f32
    } else {
        44100.0 // Default
    };

    let mut engine = AudioAnalysisEngine::new(args.chunk_size, sample_rate).await?;

    info!("🔍 Analyzing audio file...");
    let results = engine.analyze_file(&args.audio_file, args.frame_by_frame).await?;

    info!("📊 Generating analysis report...");

    // Write results to JSON file
    let json_output = serde_json::to_string_pretty(&results)?;
    let mut file = File::create(&args.output)?;
    file.write_all(json_output.as_bytes())?;

    // Print summary to console
    info!("✅ Analysis Complete!");
    info!("📈 Summary:");
    info!("  Duration: {:.2}s", results.file_info.duration_seconds);
    info!("  Total frames: {}", results.file_info.total_frames);
    info!("  Total beats detected: {}", results.beat_stats.total_beats);
    info!("  Average BPM: {:.1}", results.beat_stats.average_bpm);
    info!("  Dominant frequency: {}", results.insights.dominant_frequency_range);
    info!("  Music complexity: {:.2}", results.insights.music_complexity);
    info!("  Rhythmic consistency: {:.2}", results.insights.rhythmic_consistency);
    info!("  Harmonic content: {:.2}", results.insights.harmonic_content);
    info!("  Recommended effects: {:?}", results.insights.recommended_effects);
    info!("  Optimal smoothing: {:.2}", results.insights.optimal_smoothing_factor);

    info!("📄 Detailed results written to: {}", args.output);

    Ok(())
}