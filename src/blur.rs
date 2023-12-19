use crate::{
    camera::{Camera, CameraController},
    indices::INDICES,
    otheruniforms::{BufferContent, IncValue, OtherUniform, OtherUniforms},
    podbool::PodBool,
    texture::Texture,
    uniforms::CameraUniform,
    vertex::PostProcessingVertex,
    vertices::{POSTPROCESSING_VERTICES},
};

use glam::{vec2, vec3, Vec2, Vec3};

use std::f32::consts::PI;

use wgpu::util::DeviceExt;

use winit::{event::{VirtualKeyCode, WindowEvent}, dpi::PhysicalPosition};

use cfg_if::cfg_if;

use std::time::Duration;


pub struct Blur {
    pub uniform_bind_group: wgpu::BindGroup,

    pub space_texture_bind_group: wgpu::BindGroup,

    pub render_pipeline: wgpu::RenderPipeline,
}

impl Blur {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Self {

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("camera uniforms"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: other_uniforms_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera uniforms"),
        });

        let space_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("space_bind_group_layout"),
        });

        let space_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &space_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&space_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&space_texture.sampler),
                },
            ],
            label: Some("space_bind_group"),
        });

        let black_hole_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("black_hole_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./black_hole_maybe.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &space_texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &black_hole_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &black_hole_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {

            camera,
            camera_controller,

            camera_uniform,
            camera_uniform_buffer,

            other_uniforms,
            other_uniforms_buffer,

            uniform_bind_group,

            space_texture_bind_group,

            render_pipeline,

            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, config: &wgpu::SurfaceConfiguration) {
        self.camera.aspect = config.width as f32 / config.height as f32;
    }

    pub fn process_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) -> bool {
        let other_uniforms_event_result = self.other_uniforms.process_event(event);
        if other_uniforms_event_result {
            queue.write_buffer(
                &self.other_uniforms_buffer,
                0,
                &self.other_uniforms.uniform_buffer_content(),
            );
        }
        [
            other_uniforms_event_result,
            self.camera_controller.process_event(event),
        ]
        .iter()
        .any(|&result| result)
    }

    pub fn update(&mut self, delta_time: Duration, prev_cursor_position: Option<PhysicalPosition<f64>>, cursor_position: Option<PhysicalPosition<f64>>, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(
            &mut self.camera,
            delta_time,
            match (prev_cursor_position, cursor_position) {
                (Some(prev), Some(curr)) => vec2(curr.x as f32, curr.y as f32) != vec2(prev.x as f32, prev.y as f32),
                _ => false,
            },
        );

        self.camera_uniform.update(&self.camera);

        let data = self.camera_uniform.uniform_buffer_content();
        queue.write_buffer(&self.camera_uniform_buffer, 0, &data);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
        let mut scene_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Scene Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.2,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        scene_pass.set_pipeline(&self.render_pipeline);

        scene_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        scene_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        scene_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        // render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        scene_pass.set_bind_group(1, &self.space_texture_bind_group, &[]);

        scene_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
