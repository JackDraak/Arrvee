use super::{AudioAnalyzer, RawAudioFeatures};
use super::gpu_analyzer::{GpuAudioAnalyzer as InnerGpuAnalyzer, GpuAudioFeatures};
use anyhow::Result;
use async_trait::async_trait;

/// GPU-based audio analyzer that implements the common AudioAnalyzer trait
/// This wraps the existing GPU analyzer and outputs raw features
#[allow(dead_code)]
pub struct GpuAudioAnalyzer {
    inner: InnerGpuAnalyzer,
    device: Option<wgpu::Device>, // Stored for GPU operations
    queue: Option<wgpu::Queue>,   // Stored for GPU operations
    sample_rate: f32,
    chunk_size: usize,
}

impl GpuAudioAnalyzer {
    /// Create a new GPU-based audio analyzer
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sample_rate: f32,
        chunk_size: usize
    ) -> Result<Self> {
        let inner = InnerGpuAnalyzer::new(
            device,
            queue,
            sample_rate,
            chunk_size as u32,
        ).await?;

        Ok(Self {
            inner,
            device: None, // We'll store these when needed
            queue: None,
            sample_rate,
            chunk_size,
        })
    }

    /// Create with stored device and queue references for standalone usage
    pub async fn new_standalone(sample_rate: f32, chunk_size: usize) -> Result<Self> {
        // Create headless GPU context for compute operations
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Standalone GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let inner = InnerGpuAnalyzer::new(
            &device,
            &queue,
            sample_rate,
            chunk_size as u32,
        ).await?;

        Ok(Self {
            inner,
            device: Some(device),
            queue: Some(queue),
            sample_rate,
            chunk_size,
        })
    }
}

#[async_trait]
impl AudioAnalyzer for GpuAudioAnalyzer {
    async fn analyze_chunk(&mut self, audio_data: &[f32]) -> Result<RawAudioFeatures> {
        // Get device and queue references
        let (device_ref, queue_ref) = if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
            (device, queue)
        } else {
            // If no stored references, we need them passed from outside
            // For now, create temporary ones (this is not ideal for performance)
            return Err(anyhow::anyhow!("GPU device and queue not available. Use new_standalone() or provide external references."));
        };

        // Use the existing GPU analyzer
        let gpu_features = self.inner.analyze(device_ref, queue_ref, audio_data).await?;

        // Convert GpuAudioFeatures to RawAudioFeatures
        Ok(self.convert_gpu_features(gpu_features))
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn analyzer_type(&self) -> &'static str {
        "GPU"
    }
}

impl GpuAudioAnalyzer {
    /// Helper method to analyze with external GPU context
    pub async fn analyze_with_context(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        audio_data: &[f32]
    ) -> Result<RawAudioFeatures> {
        let gpu_features = self.inner.analyze(device, queue, audio_data).await?;
        Ok(self.convert_gpu_features(gpu_features))
    }

    /// Convert GPU features to raw features
    fn convert_gpu_features(&self, gpu_features: GpuAudioFeatures) -> RawAudioFeatures {
        // The GPU features are already raw values from the compute shaders
        // (before any normalization that was previously applied)
        RawAudioFeatures {
            sub_bass: gpu_features.sub_bass,
            bass: gpu_features.bass,
            mid: gpu_features.mid,
            treble: gpu_features.treble,
            presence: gpu_features.presence,
            spectral_centroid: gpu_features.spectral_centroid,
            spectral_rolloff: gpu_features.spectral_rolloff,
            spectral_flux: gpu_features.spectral_flux,
            zero_crossing_rate: gpu_features.zero_crossing_rate,
            onset_strength: gpu_features.onset_strength,
            beat_strength: gpu_features.beat_strength,
            estimated_bpm: gpu_features.estimated_bpm,
            volume: gpu_features.volume,
            dynamic_range: gpu_features.dynamic_range,
            pitch_confidence: gpu_features.pitch_confidence,
        }
    }
}