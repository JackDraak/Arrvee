use anyhow::Result;
use std::collections::HashMap;
use wgpu::{Device, RenderPipeline, ShaderModule};

pub struct ShaderManager {
    shaders: HashMap<String, ShaderModule>,
    pipelines: HashMap<String, RenderPipeline>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    pub fn load_shader(&mut self, device: &Device, name: &str, source: &str) -> Result<()> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.shaders.insert(name.to_string(), shader);
        Ok(())
    }

    pub fn create_pipeline(
        &mut self,
        device: &Device,
        name: &str,
        shader_name: &str,
        format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        let shader = self.shaders.get(shader_name)
            .ok_or_else(|| anyhow::anyhow!("Shader '{}' not found", shader_name))?;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", name)),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Pipeline", name)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[crate::graphics::Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.pipelines.insert(name.to_string(), pipeline);
        Ok(())
    }

    pub fn get_pipeline(&self, name: &str) -> Option<&RenderPipeline> {
        self.pipelines.get(name)
    }
}