use anyhow::Result;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// GPU-accelerated audio analysis using compute shaders
pub struct GpuAudioAnalyzer {
    // Compute pipelines
    fft_pipeline: wgpu::ComputePipeline,
    feature_extraction_pipeline: wgpu::ComputePipeline,
    beat_detection_pipeline: wgpu::ComputePipeline,

    // Buffers
    audio_buffer: wgpu::Buffer,
    fft_buffer: wgpu::Buffer,
    features_buffer: wgpu::Buffer,
    time_data_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,

    // Bind groups
    fft_bind_group: wgpu::BindGroup,
    features_bind_group: wgpu::BindGroup,
    beat_bind_group: wgpu::BindGroup,

    // Configuration
    sample_rate: f32,
    buffer_size: u32,
    num_frequency_bands: u32,

    // Time tracking
    start_time: std::time::Instant,
    frame_count: u32,
    last_beat_time: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GpuAudioConfig {
    sample_rate: f32,
    buffer_size: u32,
    num_bands: u32,
    window_type: u32, // 0=Hann, 1=Hamming, 2=Blackman
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GpuAudioFeatures {
    // Frequency bands (5-band analysis)
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub presence: f32,

    // Spectral features
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub spectral_flux: f32,

    // Temporal features
    pub zero_crossing_rate: f32,
    pub onset_strength: f32,

    // Beat analysis
    pub beat_strength: f32,
    pub estimated_bpm: f32,

    // Dynamic features
    pub volume: f32,
    pub dynamic_range: f32,
    pub pitch_confidence: f32,

    _padding: f32, // Align to 16 bytes
}

impl GpuAudioAnalyzer {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sample_rate: f32,
        buffer_size: u32,
    ) -> Result<Self> {
        let num_frequency_bands = 5;

        // Create compute shaders
        let fft_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FFT Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/compute/fft.wgsl").into()),
        });

        let feature_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Feature Extraction Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/compute/features.wgsl").into()),
        });

        let beat_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Beat Detection Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/compute/beat_detection.wgsl").into()),
        });

        // Create buffers
        let audio_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Audio Input Buffer"),
            size: (buffer_size * std::mem::size_of::<f32>() as u32) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fft_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("FFT Output Buffer"),
            size: (buffer_size * 2 * std::mem::size_of::<f32>() as u32) as u64, // Complex numbers
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let features_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Features Buffer"),
            size: std::mem::size_of::<GpuAudioFeatures>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let time_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Time Data Buffer"),
            size: (4 * std::mem::size_of::<f32>()) as u64, // [current_time, delta_time, frame_count, last_beat_time]
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: std::mem::size_of::<GpuAudioFeatures>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layouts
        let fft_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("FFT Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let features_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Features Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let beat_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Beat Detection Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipelines
        let fft_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("FFT Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("FFT Pipeline Layout"),
                bind_group_layouts: &[&fft_bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &fft_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let feature_extraction_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Feature Extraction Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Feature Pipeline Layout"),
                bind_group_layouts: &[&features_bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &feature_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let beat_detection_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Beat Detection Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Beat Pipeline Layout"),
                bind_group_layouts: &[&beat_bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &beat_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        // Create bind groups
        let fft_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("FFT Bind Group"),
            layout: &fft_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: audio_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fft_buffer.as_entire_binding(),
                },
            ],
        });

        let features_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Features Bind Group"),
            layout: &features_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fft_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: features_buffer.as_entire_binding(),
                },
            ],
        });

        let beat_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Beat Detection Bind Group"),
            layout: &beat_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fft_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: features_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_data_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            fft_pipeline,
            feature_extraction_pipeline,
            beat_detection_pipeline,
            audio_buffer,
            fft_buffer,
            features_buffer,
            time_data_buffer,
            output_buffer,
            fft_bind_group,
            features_bind_group,
            beat_bind_group,
            sample_rate,
            buffer_size,
            num_frequency_bands,
            start_time: std::time::Instant::now(),
            frame_count: 0,
            last_beat_time: 0.0,
        })
    }

    /// Analyze audio data using GPU compute shaders
    pub async fn analyze(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, audio_data: &[f32]) -> Result<GpuAudioFeatures> {
        // Update time tracking
        let current_time = self.start_time.elapsed().as_secs_f32();
        let delta_time = if self.frame_count > 0 {
            current_time - (self.frame_count as f32 * (self.buffer_size as f32 / self.sample_rate))
        } else {
            0.0
        };

        // Prepare time data for GPU
        let time_data: [f32; 4] = [
            current_time,
            delta_time,
            self.frame_count as f32,
            self.last_beat_time,
        ];

        // Upload time data to GPU
        queue.write_buffer(
            &self.time_data_buffer,
            0,
            bytemuck::cast_slice(&time_data),
        );

        // Ensure data size matches buffer
        let data_size = audio_data.len().min(self.buffer_size as usize);

        // Upload audio data to GPU
        queue.write_buffer(
            &self.audio_buffer,
            0,
            bytemuck::cast_slice(&audio_data[..data_size]),
        );

        self.frame_count += 1;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Audio Analysis Encoder"),
        });

        // 1. FFT Computation
        {
            let mut fft_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("FFT Pass"),
                timestamp_writes: None,
            });
            fft_pass.set_pipeline(&self.fft_pipeline);
            fft_pass.set_bind_group(0, &self.fft_bind_group, &[]);
            fft_pass.dispatch_workgroups(self.buffer_size / 64, 1, 1); // 64 threads per workgroup
        }

        // 2. Feature Extraction
        {
            let mut features_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Feature Extraction Pass"),
                timestamp_writes: None,
            });
            features_pass.set_pipeline(&self.feature_extraction_pipeline);
            features_pass.set_bind_group(0, &self.features_bind_group, &[]);
            features_pass.dispatch_workgroups(1, 1, 1);
        }

        // 3. Beat Detection
        {
            let mut beat_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Beat Detection Pass"),
                timestamp_writes: None,
            });
            beat_pass.set_pipeline(&self.beat_detection_pipeline);
            beat_pass.set_bind_group(0, &self.beat_bind_group, &[]);
            beat_pass.dispatch_workgroups(1, 1, 1);
        }

        // Copy results to CPU-readable buffer
        encoder.copy_buffer_to_buffer(
            &self.features_buffer,
            0,
            &self.output_buffer,
            0,
            std::mem::size_of::<GpuAudioFeatures>() as u64,
        );

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));

        // Read results
        let buffer_slice = self.output_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::wait());
        receiver.receive().await.unwrap()?;

        let data = buffer_slice.get_mapped_range();
        let features: GpuAudioFeatures = *bytemuck::from_bytes(&data[..std::mem::size_of::<GpuAudioFeatures>()]);

        drop(data);
        self.output_buffer.unmap();

        Ok(features)
    }
}