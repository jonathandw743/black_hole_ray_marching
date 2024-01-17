use crate::{
    camera::{Camera, CameraController},
    indices::INDICES,
    otheruniforms::{BufferContent, IncValue, OtherUniform, OtherUniforms},
    podbool::PodBool,
    texture::{self, Texture},
    uniforms::CameraUniform,
    vertex::PostProcessingVertex,
    vertices::POSTPROCESSING_VERTICES,
};

use glam::{uvec2, vec2, vec3, UVec2, Vec2, Vec3};

use std::{f32::consts::PI, fmt::format, mem};

use wgpu::{include_wgsl, util::DeviceExt, TextureDescriptor};

use winit::{
    dpi::PhysicalPosition,
    event::{VirtualKeyCode, WindowEvent},
};

use cfg_if::cfg_if;

use std::time::Duration;

const MLC: usize = 5;

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct MV {
//     foo: [f32; 2],
// }

pub struct Bloom {
    pub downsampling_texture_sampler: wgpu::Sampler,
    pub upsampling_texture_sampler: wgpu::Sampler,

    pub input_texture: wgpu::Texture,
    pub input_texture_view: wgpu::TextureView,

    pub textures: Vec<(wgpu::Texture, wgpu::TextureView)>,

    pub downsampling_bind_group_layout: wgpu::BindGroupLayout,
    pub downsampling_bind_groups: Vec<wgpu::BindGroup>,
    pub downsampling_render_pipeline: wgpu::RenderPipeline,

    pub upsampling_bind_group_layout: wgpu::BindGroupLayout,
    pub upsampling_bind_groups: Vec<wgpu::BindGroup>,
    pub upsampling_render_pipeline: wgpu::RenderPipeline,

    pub final_remix_bind_group_layout: wgpu::BindGroupLayout,
    pub final_remix_bind_group: wgpu::BindGroup,
    pub final_remix_render_pipeline: wgpu::RenderPipeline,
}

impl Bloom {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let (input_texture, input_texture_view) = Self::create_input_texture(device, config);

        let downsampling_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let upsampling_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let final_remix_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let textures = Self::create_textures(device, config);

        let screen_triangle_shader_module = device.create_shader_module(include_wgsl!("screen_triangle.wgsl"));
        let downsample_shader_module = device.create_shader_module(include_wgsl!("downsample.wgsl"));
        let upsample_shader_module = device.create_shader_module(include_wgsl!("upsample.wgsl"));
        let final_remix_shader_module = device.create_shader_module(include_wgsl!("final_remix.wgsl"));

        let downsampling_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("downsampling bind group layout"),
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

        let downsampling_bind_groups = Self::create_downsampling_bind_groups(
            device,
            &downsampling_bind_group_layout,
            &textures,
            &downsampling_texture_sampler,
        );

        let downsampling_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("downsampling pipeline layout"),
            bind_group_layouts: &[&downsampling_bind_group_layout],
            push_constant_ranges: &[],
        });

        let downsampling_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&downsampling_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_triangle_shader_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &downsample_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
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

        let upsampling_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("upsampling bind group layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let upsampling_bind_groups = Self::create_upsampling_bind_groups(
            device,
            &upsampling_bind_group_layout,
            &textures,
            &upsampling_texture_sampler,
        );

        let upsampling_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("upsampling pipeline layout"),
            bind_group_layouts: &[&upsampling_bind_group_layout],
            push_constant_ranges: &[],
        });

        let upsampling_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&upsampling_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_triangle_shader_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &upsample_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
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

        let final_remix_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("final remix upsampling bind group layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let final_remix_bind_group = Self::create_final_remix_bind_group(
            device,
            &final_remix_bind_group_layout,
            &input_texture_view,
            &textures,
            &final_remix_texture_sampler,
        );

        let final_remix_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("upsampling pipeline layout"),
            bind_group_layouts: &[&final_remix_bind_group_layout],
            push_constant_ranges: &[],
        });

        let final_remix_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&final_remix_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_triangle_shader_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &final_remix_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
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

        Self {
            downsampling_texture_sampler,
            upsampling_texture_sampler,

            input_texture,
            input_texture_view,

            textures,

            downsampling_bind_group_layout,
            downsampling_bind_groups,
            downsampling_render_pipeline,

            upsampling_bind_group_layout,
            upsampling_bind_groups,
            upsampling_render_pipeline,

            final_remix_bind_group_layout,
            final_remix_bind_group,
            final_remix_render_pipeline,
        }
    }

    pub fn blackout_input_texture_view(&self) -> &wgpu::TextureView {
        &self.textures[0].1
    }

    pub fn input_texture_view(&self) -> &wgpu::TextureView {
        &self.input_texture_view
    }

    fn create_input_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let input_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom input texture"),
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
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (input_texture, input_texture_view)
    }

    // this creates all the levels of texture for downsampling
    fn create_textures(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Vec<(wgpu::Texture, wgpu::TextureView)> {
        // initialise the textures as a null array
        let mut result = Vec::new();
        // track the dimensions of the current texture
        let mut dim = (config.width, config.height);
        // add all the texture to the array
        for level in 0..MLC {
            // create the texture
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("downsample texture {}", level)),
                mip_level_count: 1,
                size: wgpu::Extent3d {
                    width: dim.0,
                    height: dim.1,
                    depth_or_array_layers: 1,
                },
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                sample_count: 1,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            result.push((texture, texture_view));
            // ammend the dimension
            dim = (dim.0 / 2, dim.1 / 2);
        }
        result
    }

    // this creates all the bind groups for all the downsampling shader calls
    fn create_downsampling_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &Vec<(wgpu::Texture, wgpu::TextureView)>,
        texture_sampler: &wgpu::Sampler,
    ) -> Vec<wgpu::BindGroup> {
        // initialise the bind groups as a null array
        let mut result = Vec::new();
        // add all the bind groups to the array
        for level in 0..MLC - 1 {
            // create the bind group
            // first binding is the texture to be read from in the shader
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("downsampling bind group {}", level)),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&textures[level].1),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(texture_sampler),
                    },
                ],
            });
            result.push(bind_group);
        }
        result
    }

    // this creates all the bind groups for all the upsampling shader calls
    fn create_upsampling_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &Vec<(wgpu::Texture, wgpu::TextureView)>,
        texture_sampler: &wgpu::Sampler,
    ) -> Vec<wgpu::BindGroup> {
        // initialise the bind groups as a null array
        let mut result = Vec::new();
        // add all the bind groups to the array
        for level in 1..MLC - 1 {
            // create the bind group
            // first binding is the texture to be read from in the shader
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("upsampling bind group {}", level)),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&textures[level + 1].1),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&textures[level - 1].1),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(texture_sampler),
                    },
                ],
            });
            result.push(bind_group);
        }
        result
    }

    // this creates all the bind groups for the final re-mix
    fn create_final_remix_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        original_texture: &wgpu::TextureView,
        textures: &Vec<(wgpu::Texture, wgpu::TextureView)>,
        texture_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("final remix bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(original_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&textures[1].1),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&textures[0].1),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
            ],
        });
        bind_group
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        (self.input_texture, self.input_texture_view) = Self::create_input_texture(device, config);
        self.textures = Self::create_textures(device, config);
        self.downsampling_bind_groups = Self::create_downsampling_bind_groups(
            device,
            &self.downsampling_bind_group_layout,
            &self.textures,
            &self.downsampling_texture_sampler,
        );
        self.upsampling_bind_groups = Self::create_upsampling_bind_groups(
            device,
            &self.upsampling_bind_group_layout,
            &self.textures,
            &self.upsampling_texture_sampler,
        );
        self.final_remix_bind_group = Self::create_final_remix_bind_group(
            device,
            &self.final_remix_bind_group_layout,
            &self.input_texture_view,
            &self.textures,
            &self.upsampling_texture_sampler,
        );
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, output_view: Option<&wgpu::TextureView>) {
        for level in 0..MLC - 1 {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures[level + 1].1,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.downsampling_render_pipeline);
            render_pass.set_bind_group(0, &self.downsampling_bind_groups[level], &[]);
            render_pass.draw(0..3, 0..1);
        }

        for level in (1..MLC - 1).rev() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures[level].1,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
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
            render_pass.set_pipeline(&self.upsampling_render_pipeline);
            render_pass.set_bind_group(0, &self.upsampling_bind_groups[level - 1], &[]);
            render_pass.draw(0..3, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur render pass"),
                color_attachments: &[output_view.map(|output_view| wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.final_remix_render_pipeline);
            render_pass.set_bind_group(0, &self.final_remix_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("blur render pass"),
        //         color_attachments: &[
        //             output_view.map(|output_view| wgpu::RenderPassColorAttachment {
        //                 view: output_view,
        //                 resolve_target: None,
        //                 ops: wgpu::Operations {
        //                     load: wgpu::LoadOp::Clear(wgpu::Color {
        //                         r: 0.0,
        //                         g: 0.0,
        //                         b: 1.0,
        //                         a: 1.0,
        //                     }),
        //                     store: wgpu::StoreOp::Store,
        //                 },
        //             })
        //         ],
        //         depth_stencil_attachment: None,
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //     });
        //     render_pass.set_pipeline(&self.upsampling_render_pipeline);
        //     render_pass.set_bind_group(0, &self.upsampling_bind_groups[0], &[]);
        //     render_pass.draw(0..3, 0..1);
        // }
    }
}
