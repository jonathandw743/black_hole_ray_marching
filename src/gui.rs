use std::sync::Arc;

use egui::{Response, Ui};
use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, SurfaceConfiguration};
use winit::{event::WindowEvent, window::Window};

use crate::otheruniforms::{OtherUniform};

// pub trait CreateUi {
//     fn create_ui(&mut self, ui: Ui) -> Response;
// }

// impl CreateUi for OtherUniform {
//     fn create_ui(&mut self, ui: Ui) -> Response {
        
//     }
// }


pub struct Gui {
    pub window: Arc<Window>,
    pub egui_context: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub fn new(
        window: Arc<Window>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let egui_context = egui::Context::default();
        egui_context.set_fonts(egui::FontDefinitions::default());

        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            Default::default(),
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1, false);

        Self {
            window,
            egui_context,
            egui_state,
            egui_renderer,
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        let _ = self.egui_state.on_window_event(&self.window, &event);
    }

    fn screen_descriptor(config: &SurfaceConfiguration) -> ScreenDescriptor {
        ScreenDescriptor {
            pixels_per_point: 1.0,
            size_in_pixels: [config.width, config.height],
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        // &mut other_uniforms: OtherUniformsK,
    ) {
        let screen_descriptor = Self::screen_descriptor(config);
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_context.run(raw_input, |ctx| {
            // Create the side panel UI
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                ui.heading("Controls");
                if ui.button("Click me").clicked() {
                    println!("Button was clicked!");
                }
                // ui.checkbox(&mut x, "hello");
                ui.label("Hello, egui!")
            });
        });
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output.clone());

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self.egui_context.tessellate(full_output.shapes, 1.0);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }
        self.egui_renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        // this forget_lifetime and drop business is not ok the egui people must be stopped
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();
        self.egui_renderer
            .render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
    }
}
