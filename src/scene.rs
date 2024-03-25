use crate::{
    camera::{Camera, CameraController},
    indices::INDICES,
    otheruniforms::{BufferContent, IncValue, OtherUniform, OtherUniforms},
    podbool::PodBool,
    texture::Texture,
    uniforms::CameraUniform,
    vertex::Vertex,
    vertices::VERTICES,
};

use glam::{uvec2, vec2, vec3, vec4, UVec2, Vec2, Vec3, Vec4Swizzles};

use std::f32::consts::PI;

use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalPosition,
    event::{VirtualKeyCode, WindowEvent},
};

use cfg_if::cfg_if;

use std::time::Duration;

pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,

    pub camera_uniform: CameraUniform,
    pub camera_uniform_buffer: wgpu::Buffer,

    pub other_uniforms: OtherUniforms<6>,
    pub other_uniforms_buffer: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,

    pub space_texture_bind_group: wgpu::BindGroup,

    pub render_pipeline: wgpu::RenderPipeline,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    pub resolution_uniform: UVec2,
    pub resolution_uniform_buffer: wgpu::Buffer,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_blackout: bool,
    ) -> Self {
        let resolution_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<UVec2>() as wgpu::BufferAddress,
            label: Some("scene resolution_uniform_buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let resolution_uniform = Self::create_resolution(queue, config, &resolution_uniform_buffer);

        let camera = Camera {
            pos: (0.0, 0.0, -20.0).into(),
            dir: (0.0, 0.0, 1.0).into(),
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: PI * 0.4,
            znear: 0.1,
            zfar: 100.0,
        };
        {
            let positions = vec![vec2(3.0, 1.0), vec2(-1.0, 1.0), vec2(-1.0, -3.0)];

            println!("{:?}", camera.build_view_projection_matrix().inverse());
            // println!(
            //     "{:?}",
            //     camera.build_view_projection_matrix() * vec4(0.0, 0.0, 100.0, 1.0)
            // );
            // println!(
            //     "{:?}",
            //     camera.build_view_projection_matrix() * vec4(1.0, 0.0, 0.0, 1.0)
            // );
            // println!(
            //     "{:?}",
            //     camera.build_view_projection_matrix() * vec4(1.0, 0.0, 30.0, 1.0)
            // );
            for pos in positions {
                let clip_pos_hom = vec4(pos.x, pos.y, 0.0, 1.0);
                let mut world_pos_hom =
                    camera.build_view_projection_matrix().inverse() * clip_pos_hom;
                world_pos_hom /= world_pos_hom.w;
                println!("{:?}", world_pos_hom.xyz() - camera.pos);
            }
        }

        let camera_controller = CameraController::new(5.0, 0.5);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera uniforms"),
            contents: &camera_uniform.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let other_uniforms = OtherUniforms::new(
            VirtualKeyCode::PageUp,
            VirtualKeyCode::PageDown,
            [
                OtherUniform {
                    label: "swartschild radius".into(),
                    // shader_stage: ShaderStages::FRAGMENT,
                    inc_value: Box::new(IncValue {
                        value: 1.0,
                        inc: 0.2,
                    }),
                },
                OtherUniform {
                    label: "max delta time".into(),
                    inc_value: Box::new(IncValue {
                        value: 0.5,
                        inc: 0.02,
                    }),
                },
                OtherUniform {
                    label: "background brightness".into(),
                    inc_value: Box::new(IncValue {
                        value: 0.5,
                        inc: 0.1,
                    }),
                },
                OtherUniform {
                    label: "blackout event horizon".into(),
                    inc_value: Box::new(IncValue {
                        value: PodBool::r#false(),
                        inc: PodBool::r#true(),
                    }),
                },
                OtherUniform {
                    label: "max view distance".into(),
                    inc_value: Box::new(IncValue {
                        value: 250.0,
                        inc: 1.0,
                    }),
                },
                OtherUniform {
                    label: "distortion power".into(),
                    inc_value: Box::new(IncValue {
                        value: 1.0,
                        inc: 0.2,
                    }),
                },
            ],
        );

        let other_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camer uniforms"),
            contents: &other_uniforms.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("scene bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
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
            label: Some("scene bind_group"),
        });

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let space_bytes = include_bytes!("space_2048x1024.jpg");
            } else {
                // let space_bytes = include_bytes!("space_4096x2048.jpg");
                let space_bytes = include_bytes!("dark_space.jpg");
                // let space_bytes = include_bytes!("space_4096x2048.jpg");
            }
        }
        let space_texture = Texture::from_bytes(&device, &queue, space_bytes, "space").unwrap();

        let space_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // or include_wgsl!
        let black_hole_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("black_hole_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./black_hole_maybe.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scene Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &space_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &black_hole_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &black_hole_shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    if render_blackout {
                        Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    } else {
                        None
                    },
                ],
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

            bind_group,

            space_texture_bind_group,

            render_pipeline,

            vertex_buffer,
            index_buffer,
            num_indices,

            resolution_uniform,
            resolution_uniform_buffer,
        }
    }

    pub fn create_resolution(
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        buffer: &wgpu::Buffer,
    ) -> UVec2 {
        let resolution_uniform = uvec2(config.width, config.height);
        queue.write_buffer(buffer, 0, &resolution_uniform.uniform_buffer_content());
        return resolution_uniform;
    }

    pub fn create_output_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene output_texture"),
            mip_level_count: 1,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            view_formats: &[],
        });

        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        return (output_texture, output_texture_view);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // new_size: winit::dpi::PhysicalSize<u32>,
        config: &wgpu::SurfaceConfiguration,
        // input_texture_view: &wgpu::TextureView,
    ) {
        self.resolution_uniform =
            Self::create_resolution(queue, config, &self.resolution_uniform_buffer);

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

    pub fn update(
        &mut self,
        delta_time: Duration,
        prev_cursor_position: Option<PhysicalPosition<f64>>,
        cursor_position: Option<PhysicalPosition<f64>>,
        queue: &wgpu::Queue,
    ) {
        self.camera_controller.update_camera(
            &mut self.camera,
            delta_time,
            match (prev_cursor_position, cursor_position) {
                (Some(prev), Some(curr)) => {
                    vec2(curr.x as f32, curr.y as f32) != vec2(prev.x as f32, prev.y as f32)
                }
                _ => false,
            },
        );

        self.camera_uniform.update(&self.camera);

        let data = self.camera_uniform.uniform_buffer_content();
        queue.write_buffer(&self.camera_uniform_buffer, 0, &data);
    }

    // renders the scene onto the given view(s)
    // if none are given, then the render will have no output
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: Option<&wgpu::TextureView>,
        blackout_output_view: Option<&wgpu::TextureView>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene render_pass"),
            color_attachments: &[
                output_view.map(|output_view| wgpu::RenderPassColorAttachment {
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
                }),
                blackout_output_view.map(|blackout_output_view| wgpu::RenderPassColorAttachment {
                    view: blackout_output_view,
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
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_bind_group(1, &self.space_texture_bind_group, &[]);

        // render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        render_pass.draw(0..3, 0..1);
    }
}
