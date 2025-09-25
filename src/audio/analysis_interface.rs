use anyhow::Result;
use async_trait::async_trait;

/// Raw audio features extracted from audio analysis before normalization.
///
/// Each analyzer implementation outputs features in their natural ranges, which may vary
/// significantly between CPU and GPU implementations. These raw features are then normalized
/// by `FeatureNormalizer` to ensure consistent 0.0-1.0 ranges for visual consumption.
///
/// # Example Ranges
/// - `bass`: Typically 0.0 to 100.0+ (raw FFT magnitude)
/// - `spectral_centroid`: 0.0 to 22050.0 Hz (Nyquist frequency)
/// - `volume`: 0.0 to 1.0 (RMS amplitude)
///
/// # Usage
/// ```rust,no_run
/// # use anyhow::Result;
/// # async fn example(analyzer: &mut dyn crate::audio::AudioAnalyzer, audio: &[f32]) -> Result<()> {
/// let raw_features = analyzer.analyze_chunk(audio).await?;
/// println!("Bass energy: {}", raw_features.bass);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RawAudioFeatures {
    // Frequency bands (raw magnitudes/energies)
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub presence: f32,

    // Spectral features (raw values)
    pub spectral_centroid: f32,      // Hz
    pub spectral_rolloff: f32,       // Hz
    pub spectral_flux: f32,          // Raw variance/change measure

    // Temporal features (raw values)
    pub zero_crossing_rate: f32,     // Raw ratio or count
    pub onset_strength: f32,         // Raw energy measure

    // Beat analysis (raw values)
    pub beat_strength: f32,          // Raw energy measure
    pub estimated_bpm: f32,          // BPM (already meaningful unit)

    // Dynamic features (raw values)
    pub volume: f32,                 // RMS magnitude
    pub dynamic_range: f32,          // Raw range measure
    pub pitch_confidence: f32,       // Raw confidence score
}

/// Common interface for all audio analysis implementations.
///
/// This trait enables transparent switching between CPU and GPU analyzers while guaranteeing
/// identical output ranges after normalization. Implementations must output raw features in
/// their natural ranges, which are then normalized by `FeatureNormalizer`.
///
/// # Implementation Requirements
/// - Must be thread-safe (`Send` bound required)
/// - Should output consistent raw features for identical input
/// - Raw features can be in any range (normalization happens later)
///
/// # Available Implementations
/// - `CpuAudioAnalyzer`: CPU-based FFT analysis
/// - `NewGpuAudioAnalyzer`: GPU-accelerated WGSL compute shaders
///
/// # Example Usage
/// ```rust,no_run
/// use crate::audio::{AudioAnalyzer, CpuAudioAnalyzer, FeatureNormalizer};
/// use anyhow::Result;
///
/// async fn process_audio(audio: &[f32]) -> Result<()> {
///     // Analyzer selection (automatic GPU/CPU fallback in practice)
///     let mut analyzer: Box<dyn AudioAnalyzer + Send> =
///         Box::new(CpuAudioAnalyzer::new(44100.0, 512)?);
///
///     let raw_features = analyzer.analyze_chunk(audio).await?;
///     let mut normalizer = FeatureNormalizer::new();
///     let normalized = normalizer.normalize(&raw_features);
///
///     println!("Analyzer: {}", analyzer.analyzer_type());
///     println!("Normalized bass: {:.3}", normalized.bass); // Always 0.0-1.0
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
#[async_trait]
pub trait AudioAnalyzer {
    /// Analyze a chunk of audio data and return raw features.
    ///
    /// # Arguments
    /// * `audio_data` - Mono audio samples (typically 512 samples at 44.1kHz)
    ///
    /// # Returns
    /// Raw audio features in their natural ranges. These will be normalized later.
    ///
    /// # Errors
    /// Returns an error if analysis fails (e.g., GPU compute error, invalid input size)
    async fn analyze_chunk(&mut self, audio_data: &[f32]) -> Result<RawAudioFeatures>;

    /// Get the sample rate this analyzer is configured for.
    fn sample_rate(&self) -> f32;

    /// Get the chunk size this analyzer expects for `analyze_chunk`.
    fn chunk_size(&self) -> usize;

    /// Get analyzer-specific identification string ("CPU", "GPU", etc.).
    ///
    /// Used for logging and debugging to identify which analyzer is active.
    fn analyzer_type(&self) -> &'static str;
}

/// Normalized audio features (guaranteed 0.0-1.0 range)
/// This is what the visual system consumes
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NormalizedAudioFeatures {
    // Frequency bands (0.0-1.0)
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub presence: f32,

    // Spectral features (0.0-1.0)
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub spectral_flux: f32,

    // Temporal features (0.0-1.0)
    pub zero_crossing_rate: f32,
    pub onset_strength: f32,

    // Beat analysis
    pub beat_detected: bool,
    pub beat_strength: f32,          // 0.0-1.0
    pub estimated_bpm: f32,          // Raw BPM (meaningful unit)

    // Dynamic features (0.0-1.0)
    pub volume: f32,
    pub dynamic_range: f32,
    pub pitch_confidence: f32,
}