use crate::{
    camera::{Camera, CameraController},
    indices::INDICES,
    otheruniforms::{
        BufferContent, IncValue, IncrementableOtherUniform, IncrementableOtherUniforms,
        IncrementableOtherUniformsControllerKeyboard, OtherUniform,
        PodBool
    },
    texture::Texture,
    uniforms::{BlackHole, BlackHolesUniform, CameraUniform},
    vertex::Vertex,
    vertices::VERTICES,
};

use glam::{uvec2, vec2, vec3, vec4, UVec2, Vec2, Vec3, Vec4Swizzles};

use std::{default, f32::consts::PI};

use wgpu::{include_wgsl, util::DeviceExt};

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, WindowEvent},
    keyboard::KeyCode,
};

use cfg_if::cfg_if;

use std::time::Duration;

pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,

    pub camera_uniform: CameraUniform,
    pub camera_uniform_buffer: wgpu::Buffer,

    pub other_uniforms: IncrementableOtherUniforms<9>,
    pub other_uniforms_controller_keyboard: IncrementableOtherUniformsControllerKeyboard,
    pub other_uniforms_buffer: wgpu::Buffer,

    pub black_holes_uniform: BlackHolesUniform<10>,
    pub black_holes_uniform_buffer: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,

    pub space_texture_bind_group: wgpu::BindGroup,

    pub render_pipeline: wgpu::RenderPipeline,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_blackout: bool,
    ) -> Self {
        let camera = Camera {
            pos: (0.0, 0.0, -20.0).into(),
            dir: (0.0, 0.0, 1.0).into(),
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: PI * 0.5,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(5.0, 0.5);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera uniforms"),
            contents: &camera_uniform.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let other_uniforms = IncrementableOtherUniforms::new([
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "dist_to_surfaces_mult".into(),
                    value: IncValue {
                        value: 0.6,
                        inc: 0.05,
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "dist_to_singularity_squared_mult".into(),
                    value: IncValue {
                        value: 0.04,
                        inc: 0.004,
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "blackout_event_horizon".into(),
                    value: IncValue {
                        value: PodBool::r#true(),
                        inc: PodBool::r#true(),
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "min_dist".into(),
                    value: IncValue {
                        value: 0.001,
                        inc: 0.0001,
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "max_dist".into(),
                    value: IncValue {
                        value: 250.0,
                        inc: 5.0,
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "distortion_power".into(),
                    value: IncValue {
                        value: 1.0,
                        inc: 0.1,
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "debug_colours".into(),
                    value: IncValue {
                        value: PodBool::r#false(),
                        inc: PodBool::r#true(),
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "blackout_requires_ray_towards_black_hole".into(),
                    value: IncValue {
                        value: PodBool::r#false(),
                        inc: PodBool::r#true(),
                    },
                },
            }),
            Box::new(IncrementableOtherUniform {
                other_uniform: OtherUniform {
                    label: "photon_sphere".into(),
                    value: IncValue {
                        value: PodBool::r#false(),
                        inc: PodBool::r#true(),
                    },
                },
            }),
        ]);

        let other_uniforms_controller_keyboard =
            IncrementableOtherUniformsControllerKeyboard::new(KeyCode::PageUp, KeyCode::PageDown);

        let other_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("other_uniforms_buffer"),
            contents: &other_uniforms.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let black_holes_uniform = BlackHolesUniform::new([
            BlackHole {
                pos: vec3(0.0, 0.0, 0.0),
                rs: 1.0,
                accretion_disk_size: 0.0,
            },
            BlackHole {
                pos: vec3(3.0, 0.0, 0.0),
                rs: 1.0,
                accretion_disk_size: 0.0,
            },
        ]);

        let black_holes_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("other_uniforms_buffer"),
                contents: &black_holes_uniform.uniform_buffer_content(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene bind_group_layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene bind_group"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: black_holes_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // let space_bytes = include_bytes!("space_2048x1024.jpg");
                let space_bytes = include_bytes!("space_4096x2048.jpg");
            } else {
                let space_bytes = include_bytes!("space_4096x2048.jpg");
            }
        }
        let space_texture =
            Texture::from_bytes(&device, &queue, space_bytes, "space_texture").unwrap();

        let space_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("space_bind_group_layout"),
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

        let black_hole_shader = device.create_shader_module(include_wgsl!("black_holes.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scene pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &space_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &black_hole_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
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
                compilation_options: Default::default(),
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
            cache: None,
        });

        Self {
            camera,
            camera_controller,

            camera_uniform,
            camera_uniform_buffer,

            other_uniforms,
            other_uniforms_controller_keyboard,
            other_uniforms_buffer,

            black_holes_uniform,
            black_holes_uniform_buffer,

            bind_group,

            space_texture_bind_group,

            render_pipeline,
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

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.camera.aspect = config.width as f32 / config.height as f32;
    }

    pub fn process_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) -> bool {
        let other_uniforms_event_result = self
            .other_uniforms_controller_keyboard
            .process_event(event, &mut self.other_uniforms);
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
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.space_texture_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
