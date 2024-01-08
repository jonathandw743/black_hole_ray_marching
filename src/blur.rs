use crate::{
    camera::{Camera, CameraController},
    indices::INDICES,
    otheruniforms::{BufferContent, IncValue, OtherUniform, OtherUniforms},
    podbool::PodBool,
    texture::Texture,
    uniforms::CameraUniform,
    vertex::PostProcessingVertex,
    vertices::POSTPROCESSING_VERTICES,
};

use glam::{uvec2, vec2, vec3, UVec2, Vec2, Vec3};

use std::f32::consts::PI;

use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalPosition,
    event::{VirtualKeyCode, WindowEvent},
};

use cfg_if::cfg_if;

use std::time::Duration;

pub struct Blur {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub output_texture: wgpu::Texture,
    pub output_texture_view: wgpu::TextureView,
    pub input_texture_sampler: wgpu::Sampler,
    pub resolution_uniform: UVec2,
    pub resolution_uniform_buffer: wgpu::Buffer,
    pub blur_size_uniform: f32,
    pub blur_size_uniform_buffer: wgpu::Buffer,
}

impl Blur {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        input_texture_view: &wgpu::TextureView,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(POSTPROCESSING_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = POSTPROCESSING_VERTICES.len() as u32;

        let (output_texture, output_texture_view) = Self::create_output_texture(device, config);

        // or include_wgsl!
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blur_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./blur.wgsl").into()),
        });

        let input_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let resolution_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<UVec2>() as wgpu::BufferAddress,
            label: Some("blur resolution_uniform_buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let resolution_uniform = Self::create_resolution(queue, config, &resolution_uniform_buffer);

        let blur_size_uniform = 2.0;

        let blur_size_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("blur blur size uniform buffer"),
            contents: &bytemuck::cast_slice(&[blur_size_uniform]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("blur bind_group_layout"),
        });

        let bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            input_texture_view,
            &input_texture_sampler,
            &resolution_uniform_buffer,
            &blur_size_uniform_buffer,
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blur_shader,
                entry_point: "vs_main",
                buffers: &[PostProcessingVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &blur_shader,
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

        Self {
            bind_group_layout,
            bind_group,

            output_texture,
            output_texture_view,

            render_pipeline,

            vertex_buffer,
            num_vertices,

            input_texture_sampler,

            resolution_uniform,
            resolution_uniform_buffer,

            blur_size_uniform,
            blur_size_uniform_buffer,
        }
    }

    pub fn create_resolution(queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, buffer: &wgpu::Buffer) -> UVec2 {
        let resolution_uniform = uvec2(config.width, config.height);
        queue.write_buffer(buffer, 0, &resolution_uniform.uniform_buffer_content());
        return resolution_uniform;
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        input_texture_view: &wgpu::TextureView,
        input_texture_sampler: &wgpu::Sampler,
        resolution_uniform_buffer: &wgpu::Buffer,
        blur_size_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(input_texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: resolution_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: blur_size_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("blur bind_group"),
        });

        return bind_group;
    }

    pub fn create_output_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blur output_texture"),
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

        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        return (output_texture, output_texture_view);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // new_size: winit::dpi::PhysicalSize<u32>,
        config: &wgpu::SurfaceConfiguration,
        input_texture_view: &wgpu::TextureView,
    ) {
        (self.output_texture, self.output_texture_view) = Self::create_output_texture(device, config);

        self.resolution_uniform = Self::create_resolution(queue, config, &self.resolution_uniform_buffer);

        self.bind_group = Self::create_bind_group(
            device,
            &self.bind_group_layout,
            input_texture_view,
            &self.input_texture_sampler,
            &self.resolution_uniform_buffer,
            &self.blur_size_uniform_buffer,
        );
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, optional_output_view: Option<&wgpu::TextureView>) {
        let output_view = match optional_output_view {
            Some(output_view) => output_view,
            None => &self.output_texture_view,
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blur render pass"),
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

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.draw(0..self.num_vertices, 0..1);
    }
}

// pub fn blur(
//     device: &wgpu::Device,
//     encoder: &mut wgpu::CommandEncoder,
//     source_texture: &wgpu::Texture,
//     destination_texture: &wgpu::Texture,
// ) {
//     let source_texture_copy_view = wgpu::ImageCopyTexture {
//         aspect: wgpu::TextureAspect::All,
//         texture: &source_texture,
//         mip_level: 0,
//         origin: wgpu::Origin3d::ZERO,
//     };

//     let intermediate_texture_size = wgpu::Extent3d {
//         width: source_texture.width() / 8,
//         height: source_texture.height() / 8,
//         depth_or_array_layers: 1,
//     };

//     let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
//         label: Some("blur destination_texture"),
//         size: intermediate_texture_size,
//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Bgra8UnormSrgb,
//         usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
//         view_formats: &[],
//     });

//     let intermediate_texture_copy_view = wgpu::ImageCopyTexture {
//         aspect: wgpu::TextureAspect::All,
//         texture: &intermediate_texture,
//         mip_level: 0,
//         origin: wgpu::Origin3d::ZERO,
//     };

//     let destination_texture_copy_view = wgpu::ImageCopyTexture {
//         aspect: wgpu::TextureAspect::All,
//         texture: &destination_texture,
//         mip_level: 0,
//         origin: wgpu::Origin3d::ZERO,
//     };

//     encoder.copy_texture_to_texture(
//         source_texture_copy_view,
//         intermediate_texture_copy_view,
//         intermediate_texture_size,
//     );

//     encoder.copy_texture_to_texture(
//         intermediate_texture_copy_view,
//         destination_texture_copy_view,
//         intermediate_texture_size,
//     );
// }
