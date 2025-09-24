use anyhow::Result;
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureView};
use winit::{event::WindowEvent, window::Window};

use crate::graphics::GraphicsEngine;

pub struct UserInterface {
    context: egui::Context,
    state: State,
    renderer: Renderer,
    show_controls: bool,
    volume: f32,
    selected_preset: usize,
}

impl UserInterface {
    pub fn new(window: &Window, graphics_engine: &GraphicsEngine) -> Self {
        let context = egui::Context::default();

        let egui_state = State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
        );

        let renderer = Renderer::new(
            &graphics_engine.device,
            graphics_engine.config.format,
            None,
            1,
        );

        Self {
            context,
            state: egui_state,
            renderer,
            show_controls: true,
            volume: 0.1,
            selected_preset: 0,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent, window: &Window) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        device: &Device,
        queue: &Queue,
        window: &Window,
    ) -> Result<()> {
        let raw_input = self.state.take_egui_input(window);

        let show_controls = &mut self.show_controls;
        let volume = &mut self.volume;
        let selected_preset = &mut self.selected_preset;

        let full_output = self.context.run(raw_input, |ctx| {
            Self::ui_content(ctx, show_controls, volume, selected_preset);
        });

        self.state.handle_platform_output(window, full_output.platform_output);

        let tris = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [device.limits().max_texture_dimension_2d; 2],
            pixels_per_point: full_output.pixels_per_point,
        };

        self.renderer.update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        Ok(())
    }

    fn ui_content(ctx: &egui::Context, show_controls: &mut bool, volume: &mut f32, selected_preset: &mut usize) {
        if *show_controls {
            egui::Window::new("Arrvee Controls")
                .default_pos([10.0, 10.0])
                .default_size([300.0, 200.0])
                .show(ctx, |ui| {
                    ui.heading("Music Visualizer");

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Volume:");
                        ui.add(egui::Slider::new(volume, 0.0..=1.0));
                    });

                    ui.separator();

                    ui.label("Presets:");
                    ui.radio_value(selected_preset, 0, "Plasma Dreams");
                    ui.radio_value(selected_preset, 1, "Spectrum Bars");
                    ui.radio_value(selected_preset, 2, "Radial Waves");
                    ui.radio_value(selected_preset, 3, "Beat Sync");

                    ui.separator();

                    if ui.button("Load Audio File").clicked() {
                        // TODO: Implement file picker
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() {
                            // TODO: Implement play functionality
                        }
                        if ui.button("Pause").clicked() {
                            // TODO: Implement pause functionality
                        }
                        if ui.button("Stop").clicked() {
                            // TODO: Implement stop functionality
                        }
                    });

                    ui.separator();

                    ui.checkbox(show_controls, "Show Controls");

                    ui.separator();

                    ui.label("Press ESC to exit");
                });
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            *show_controls = !*show_controls;
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn selected_preset(&self) -> usize {
        self.selected_preset
    }
}