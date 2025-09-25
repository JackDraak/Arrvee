use super::{RawAudioFeatures, NormalizedAudioFeatures};
use serde::{Serialize, Deserialize};

/// Normalization parameters defining the maximum expected ranges for each audio feature.
///
/// These parameters define the upper bounds for converting raw audio features to the
/// normalized 0.0-1.0 range. Values can be set based on typical music characteristics
/// or learned adaptively from data.
///
/// # Design Principle
/// All normalized features are clamped to 0.0-1.0 range:
/// `normalized_value = (raw_value / max_value).clamp(0.0, 1.0)`
///
/// # Parameter Sources
/// - **Default values**: Based on analysis of diverse music samples
/// - **Adaptive learning**: Can be updated based on observed data ranges
/// - **Manual tuning**: Adjusted for specific music genres or content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationParameters {
    // Frequency band normalization ranges
    pub sub_bass_max: f32,
    pub bass_max: f32,
    pub mid_max: f32,
    pub treble_max: f32,
    pub presence_max: f32,

    // Spectral feature ranges
    pub spectral_centroid_max: f32,  // Hz
    pub spectral_rolloff_max: f32,   // Hz
    pub spectral_flux_max: f32,      // Variance/change measure

    // Temporal feature ranges
    pub zero_crossing_rate_max: f32,
    pub onset_strength_max: f32,

    // Beat analysis ranges
    pub beat_strength_max: f32,
    pub bpm_min: f32,
    pub bpm_max: f32,

    // Dynamic feature ranges
    pub volume_max: f32,
    pub dynamic_range_max: f32,
    pub pitch_confidence_max: f32,
}

impl Default for NormalizationParameters {
    fn default() -> Self {
        Self {
            // Frequency bands - CORRECTED based on actual FFT magnitudes (order of magnitude smaller!)
            sub_bass_max: 0.0001,      // Actual raw FFT values are ~0.00001-0.0001
            bass_max: 0.0005,          // Real bass peaks at ~0.0001-0.0005
            mid_max: 0.0002,           // Mid frequencies typically ~0.00001-0.0002
            treble_max: 0.0001,        // Treble is even smaller ~0.00001-0.0001
            presence_max: 0.00005,     // High frequencies are smallest ~0.000005-0.00005

            // Spectral features - based on audio characteristics
            spectral_centroid_max: 8000.0,  // Most music centers below 8kHz
            spectral_rolloff_max: 12000.0,   // 85% energy usually below 12kHz
            spectral_flux_max: 0.0001,       // Spectral change measure is also tiny

            // Temporal features - CORRECTED based on observed ranges
            zero_crossing_rate_max: 0.5,     // ZCR is often already 0-1
            onset_strength_max: 0.0002,      // Onset detection strength is also tiny

            // Beat analysis - CORRECTED for actual beat energy values
            beat_strength_max: 0.0005,       // Beat energy measure is also small
            bpm_min: 60.0,                   // Reasonable BPM range
            bpm_max: 200.0,

            // Dynamic features - CORRECTED for actual RMS values
            volume_max: 0.0001,              // RMS magnitude is also tiny ~0.000001-0.0001
            dynamic_range_max: 0.0002,       // Dynamic range measure
            pitch_confidence_max: 1.0,       // Confidence is often already 0-1
        }
    }
}

/// The single source of truth for audio feature normalization.
///
/// `FeatureNormalizer` ensures that all audio features, regardless of their source
/// (CPU analyzer, GPU analyzer, etc.), are normalized to a consistent 0.0-1.0 range
/// for consumption by the visual effects system.
///
/// # Architecture Role
/// This component is critical to the unified analysis architecture:
/// - **Consistency Guarantee**: CPU and GPU analyzers produce identical normalized output
/// - **Single Source of Truth**: All normalization logic centralized in one place
/// - **Extensibility**: New analyzer types automatically benefit from normalization
///
/// # Usage Pattern
/// ```rust,no_run
/// use crate::audio::{FeatureNormalizer, RawAudioFeatures};
///
/// let mut normalizer = FeatureNormalizer::new();
/// let raw_features = RawAudioFeatures {
///     bass: 45.2,           // Raw FFT magnitude
///     volume: 0.8,          // RMS amplitude
///     // ... other fields
///     # sub_bass: 0.0, mid: 0.0, treble: 0.0, presence: 0.0,
///     # spectral_centroid: 0.0, spectral_rolloff: 0.0, spectral_flux: 0.0,
///     # zero_crossing_rate: 0.0, onset_strength: 0.0, beat_strength: 0.0,
///     # estimated_bpm: 120.0, dynamic_range: 0.0, pitch_confidence: 0.0,
/// };
///
/// let normalized = normalizer.normalize(&raw_features);
/// assert!(normalized.bass >= 0.0 && normalized.bass <= 1.0);
/// assert!(normalized.volume >= 0.0 && normalized.volume <= 1.0);
/// ```
///
/// # Adaptive Learning
/// When enabled, the normalizer can learn appropriate ranges from data:
/// ```rust,no_run
/// let mut adaptive_normalizer = FeatureNormalizer::new_adaptive();
/// // Normalization parameters automatically adjust based on observed data
/// ```
#[allow(dead_code)]
pub struct FeatureNormalizer {
    parameters: NormalizationParameters,
    adaptive: bool,

    // For adaptive normalization - track observed ranges
    observed_ranges: Option<ObservedRanges>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ObservedRanges {
    // Running max values observed
    sub_bass_max: f32,
    bass_max: f32,
    mid_max: f32,
    treble_max: f32,
    presence_max: f32,
    spectral_centroid_max: f32,
    spectral_rolloff_max: f32,
    spectral_flux_max: f32,
    zero_crossing_rate_max: f32,
    onset_strength_max: f32,
    beat_strength_max: f32,
    volume_max: f32,
    dynamic_range_max: f32,
    pitch_confidence_max: f32,

    // Sample count for adaptive learning
    sample_count: usize,
}

impl Default for ObservedRanges {
    fn default() -> Self {
        Self {
            sub_bass_max: 0.001,
            bass_max: 0.001,
            mid_max: 0.001,
            treble_max: 0.001,
            presence_max: 0.001,
            spectral_centroid_max: 1.0,
            spectral_rolloff_max: 1.0,
            spectral_flux_max: 0.001,
            zero_crossing_rate_max: 0.001,
            onset_strength_max: 0.001,
            beat_strength_max: 0.001,
            volume_max: 0.001,
            dynamic_range_max: 0.001,
            pitch_confidence_max: 0.001,
            sample_count: 0,
        }
    }
}

impl FeatureNormalizer {
    /// Create a new normalizer with default parameters
    pub fn new() -> Self {
        Self {
            parameters: NormalizationParameters::default(),
            adaptive: false,
            observed_ranges: None,
        }
    }

    /// Create a new adaptive normalizer that learns from data
    pub fn new_adaptive() -> Self {
        Self {
            parameters: NormalizationParameters::default(),
            adaptive: true,
            observed_ranges: Some(ObservedRanges::default()),
        }
    }

    /// Create normalizer with custom parameters
    pub fn with_parameters(parameters: NormalizationParameters) -> Self {
        Self {
            parameters,
            adaptive: false,
            observed_ranges: None,
        }
    }

    /// Normalize raw features to 0.0-1.0 range
    pub fn normalize(&mut self, raw: &RawAudioFeatures) -> NormalizedAudioFeatures {
        // Debug logging to see raw input values (log occasionally to avoid spam)
        static mut RAW_DEBUG_COUNTER: u32 = 0;
        unsafe {
            RAW_DEBUG_COUNTER += 1;
            if RAW_DEBUG_COUNTER % 120 == 0 { // Log every ~2 seconds at 60fps
                log::debug!("🔬 RAW AUDIO VALUES from analyzer:");
                log::debug!("  🎵 Raw Frequency Bands: bass={:.6}, mid={:.6}, volume={:.6}",
                    raw.bass, raw.mid, raw.volume);
                log::debug!("  📐 Normalization Params: bass_max={:.3}, mid_max={:.3}, volume_max={:.3}",
                    self.parameters.bass_max, self.parameters.mid_max, self.parameters.volume_max);
            }
        }

        // Update observed ranges if adaptive
        if self.adaptive {
            self.update_observed_ranges(raw);
        }

        let params = self.effective_parameters();

        NormalizedAudioFeatures {
            // Frequency bands
            sub_bass: self.normalize_value(raw.sub_bass, params.sub_bass_max),
            bass: self.normalize_value(raw.bass, params.bass_max),
            mid: self.normalize_value(raw.mid, params.mid_max),
            treble: self.normalize_value(raw.treble, params.treble_max),
            presence: self.normalize_value(raw.presence, params.presence_max),

            // Spectral features
            spectral_centroid: self.normalize_value(raw.spectral_centroid, params.spectral_centroid_max),
            spectral_rolloff: self.normalize_value(raw.spectral_rolloff, params.spectral_rolloff_max),
            spectral_flux: self.normalize_value(raw.spectral_flux, params.spectral_flux_max),

            // Temporal features
            zero_crossing_rate: self.normalize_value(raw.zero_crossing_rate, params.zero_crossing_rate_max),
            onset_strength: self.normalize_value(raw.onset_strength, params.onset_strength_max),

            // Beat analysis
            beat_detected: raw.beat_strength > (params.beat_strength_max * 0.3), // 30% threshold
            beat_strength: self.normalize_value(raw.beat_strength, params.beat_strength_max),
            estimated_bpm: raw.estimated_bpm.clamp(params.bpm_min, params.bpm_max), // Keep as raw BPM

            // Dynamic features
            volume: self.normalize_value(raw.volume, params.volume_max),
            dynamic_range: self.normalize_value(raw.dynamic_range, params.dynamic_range_max),
            pitch_confidence: self.normalize_value(raw.pitch_confidence, params.pitch_confidence_max),
        }
    }

    /// Get current normalization parameters (either fixed or adaptive)
    pub fn get_parameters(&self) -> &NormalizationParameters {
        &self.parameters
    }

    /// Save current parameters to file
    pub fn save_parameters(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let params = self.effective_parameters();
        let json = serde_json::to_string_pretty(&params)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load parameters from file
    pub fn load_parameters(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        self.parameters = serde_json::from_str(&json)?;
        Ok(())
    }

    // Private helper methods

    fn normalize_value(&self, value: f32, max_value: f32) -> f32 {
        (value / max_value).clamp(0.0, 1.0)
    }

    fn effective_parameters(&self) -> NormalizationParameters {
        if let Some(ref observed) = self.observed_ranges {
            if observed.sample_count > 100 { // Need enough samples for reliable ranges
                // Use observed ranges with some headroom
                NormalizationParameters {
                    sub_bass_max: observed.sub_bass_max * 1.2,
                    bass_max: observed.bass_max * 1.2,
                    mid_max: observed.mid_max * 1.2,
                    treble_max: observed.treble_max * 1.2,
                    presence_max: observed.presence_max * 1.2,
                    spectral_centroid_max: observed.spectral_centroid_max * 1.1,
                    spectral_rolloff_max: observed.spectral_rolloff_max * 1.1,
                    spectral_flux_max: observed.spectral_flux_max * 1.2,
                    zero_crossing_rate_max: observed.zero_crossing_rate_max * 1.2,
                    onset_strength_max: observed.onset_strength_max * 1.2,
                    beat_strength_max: observed.beat_strength_max * 1.2,
                    volume_max: observed.volume_max * 1.2,
                    dynamic_range_max: observed.dynamic_range_max * 1.2,
                    pitch_confidence_max: observed.pitch_confidence_max * 1.1,
                    ..self.parameters
                }
            } else {
                self.parameters.clone()
            }
        } else {
            self.parameters.clone()
        }
    }

    fn update_observed_ranges(&mut self, raw: &RawAudioFeatures) {
        if let Some(ref mut observed) = self.observed_ranges {
            observed.sub_bass_max = observed.sub_bass_max.max(raw.sub_bass);
            observed.bass_max = observed.bass_max.max(raw.bass);
            observed.mid_max = observed.mid_max.max(raw.mid);
            observed.treble_max = observed.treble_max.max(raw.treble);
            observed.presence_max = observed.presence_max.max(raw.presence);
            observed.spectral_centroid_max = observed.spectral_centroid_max.max(raw.spectral_centroid);
            observed.spectral_rolloff_max = observed.spectral_rolloff_max.max(raw.spectral_rolloff);
            observed.spectral_flux_max = observed.spectral_flux_max.max(raw.spectral_flux);
            observed.zero_crossing_rate_max = observed.zero_crossing_rate_max.max(raw.zero_crossing_rate);
            observed.onset_strength_max = observed.onset_strength_max.max(raw.onset_strength);
            observed.beat_strength_max = observed.beat_strength_max.max(raw.beat_strength);
            observed.volume_max = observed.volume_max.max(raw.volume);
            observed.dynamic_range_max = observed.dynamic_range_max.max(raw.dynamic_range);
            observed.pitch_confidence_max = observed.pitch_confidence_max.max(raw.pitch_confidence);
            observed.sample_count += 1;
        }
    }
}

impl Default for FeatureNormalizer {
    fn default() -> Self {
        Self::new()
    }
}