use anyhow::Result;
use wgpu::util::DeviceExt;
use winit::window::Window;
use glam::{Mat4, Vec3};

use crate::audio::AudioFrame;
use crate::effects::PsychedelicManager;
use super::{ShaderManager, TextureManager, Vertex, VertexBuffer};

pub struct GraphicsEngine<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    pub shader_manager: ShaderManager,
    texture_manager: TextureManager,

    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    pub vertex_buffer: VertexBuffer,

    pub time: f32,
    pub psychedelic_manager: PsychedelicManager,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub time: f32,

    // Frequency bands (5-band analysis)
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub presence: f32,

    // Beat and rhythm
    pub beat_strength: f32,
    pub estimated_bpm: f32,
    pub volume: f32,

    // Spectral characteristics
    pub spectral_centroid: f32,    // Brightness
    pub spectral_rolloff: f32,     // High frequency content
    pub pitch_confidence: f32,     // Harmonic vs percussive

    // Temporal dynamics
    pub zero_crossing_rate: f32,   // Texture/noisiness
    pub spectral_flux: f32,        // Rate of change
    pub onset_strength: f32,       // Note attacks
    pub dynamic_range: f32,        // Volume variation

    // Effect weights for dynamic blending
    pub plasma_weight: f32,
    pub kaleidoscope_weight: f32,
    pub tunnel_weight: f32,
    pub particle_weight: f32,
    pub fractal_weight: f32,

    pub _padding: [f32; 3],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            time: 0.0,
            sub_bass: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            presence: 0.0,
            beat_strength: 0.0,
            estimated_bpm: 120.0,
            volume: 0.0,
            spectral_centroid: 0.0,
            spectral_rolloff: 0.0,
            pitch_confidence: 0.0,
            zero_crossing_rate: 0.0,
            spectral_flux: 0.0,
            onset_strength: 0.0,
            dynamic_range: 0.0,
            plasma_weight: 0.3,
            kaleidoscope_weight: 0.0,
            tunnel_weight: 0.0,
            particle_weight: 0.0,
            fractal_weight: 0.0,
            _padding: [0.0; 3],
        }
    }

    fn update_view_proj(&mut self, width: f32, height: f32) {
        let proj = Mat4::orthographic_rh(-width/2.0, width/2.0, -height/2.0, height/2.0, -1.0, 1.0);
        self.view_proj = proj.to_cols_array_2d();
    }
}

impl<'a> GraphicsEngine<'a> {
    pub async fn new(window: &'a Window) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(size.width as f32, size.height as f32);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let mut shader_manager = ShaderManager::new();
        let texture_manager = TextureManager::new();

        // Load both simple and psychedelic shaders
        let simple_shader = include_str!("../../shaders/simple.wgsl");
        shader_manager.load_shader(&device, "simple", simple_shader)?;

        let psychedelic_shader = include_str!("../../shaders/psychedelic_effects.wgsl");
        shader_manager.load_shader(&device, "psychedelic", psychedelic_shader)?;

        // Create pipeline with psychedelic shader
        shader_manager.create_pipeline(
            &device,
            "visualizer",
            "psychedelic",
            surface_format,
            &uniform_bind_group_layout,
        )?;

        let vertices = Self::create_fullscreen_quad();
        let vertex_buffer = VertexBuffer::new(&device, &vertices);

        let psychedelic_manager = PsychedelicManager::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            shader_manager,
            texture_manager,
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            vertex_buffer,
            time: 0.0,
            psychedelic_manager,
        })
    }

    fn create_fullscreen_quad() -> Vec<Vertex> {
        vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                color: [1.0, 1.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
        ]
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, audio_frame: &AudioFrame, window: &Window) -> Result<()> {
        let delta_time = 1.0 / 60.0;
        self.time += delta_time;

        // Update psychedelic effect manager
        self.psychedelic_manager.update(delta_time, audio_frame);
        let effect_weights = self.psychedelic_manager.get_effect_weights();

        let uniforms = Uniforms {
            view_proj: Mat4::orthographic_rh(
                -(self.size.width as f32) / 2.0,
                (self.size.width as f32) / 2.0,
                -(self.size.height as f32) / 2.0,
                (self.size.height as f32) / 2.0,
                -1.0,
                1.0,
            ).to_cols_array_2d(),
            time: self.time,
            sub_bass: audio_frame.frequency_bands.sub_bass,
            bass: audio_frame.frequency_bands.bass,
            mid: audio_frame.frequency_bands.mid,
            treble: audio_frame.frequency_bands.treble,
            presence: audio_frame.frequency_bands.presence,
            beat_strength: audio_frame.beat_strength,
            estimated_bpm: audio_frame.estimated_bpm,
            volume: audio_frame.volume,
            spectral_centroid: audio_frame.spectral_centroid,
            spectral_rolloff: audio_frame.spectral_rolloff,
            pitch_confidence: audio_frame.pitch_confidence,
            zero_crossing_rate: audio_frame.zero_crossing_rate,
            spectral_flux: audio_frame.spectral_flux,
            onset_strength: audio_frame.onset_strength,
            dynamic_range: audio_frame.dynamic_range,
            plasma_weight: *effect_weights.get("llama_plasma").unwrap_or(&0.0),
            kaleidoscope_weight: *effect_weights.get("geometric_kaleidoscope").unwrap_or(&0.0),
            tunnel_weight: *effect_weights.get("psychedelic_tunnel").unwrap_or(&0.0),
            particle_weight: *effect_weights.get("particle_swarm").unwrap_or(&0.0),
            fractal_weight: *effect_weights.get("fractal_madness").unwrap_or(&0.0),
            _padding: [0.0; 3],
        };

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if let Some(pipeline) = self.shader_manager.get_pipeline("visualizer") {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
                render_pass.draw(0..self.vertex_buffer.vertex_count, 0..1);
            }
        }

        // UI rendering would go here

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get mutable access to the psychedelic effect manager for configuration
    pub fn psychedelic_manager_mut(&mut self) -> &mut PsychedelicManager {
        &mut self.psychedelic_manager
    }

    /// Get read-only access to the psychedelic effect manager
    pub fn psychedelic_manager(&self) -> &PsychedelicManager {
        &self.psychedelic_manager
    }
}