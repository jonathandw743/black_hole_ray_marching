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

const MLC: usize = 4;

pub struct Bloom {
    pub texture_sampler: wgpu::Sampler,

    pub downsampling_compute_pipeline: wgpu::ComputePipeline,
    pub downsampling_textures: [(wgpu::Texture, wgpu::TextureView); MLC],
    pub downsampling_bind_groups: [wgpu::BindGroup; MLC],
}

impl Bloom {
    pub fn new(
        device: &wgpu::Device,
        // queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        input_texture_view: &wgpu::TextureView,
    ) -> Self {
        // let downsample_compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("downsample compute pipeline layout"),
        //     bind_group_layouts: &[

        //     ],
        //     push_constant_ranges: &[],
        // });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let downsampling_compute_shader_module = device.create_shader_module(include_wgsl!("bloom.wgsl"));
        let downsampling_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            entry_point: "main",
            label: Some("compute_pipeline"),
            layout: None,
            module: &downsampling_compute_shader_module,
        });
        let downsampling_textures = Self::create_textures(device, config);
        let downsampling_bind_groups = Self::create_bind_groups(
            device,
            &downsampling_compute_pipeline.get_bind_group_layout(0),
            &downsampling_textures,
            input_texture_view,
            &texture_sampler,
        );

        Self {
            texture_sampler,

            downsampling_textures,
            downsampling_bind_groups,

            downsampling_compute_pipeline,
        }
    }

    // this creates all the levels of texture for downsampling
    fn create_textures(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> [(wgpu::Texture, wgpu::TextureView); MLC] {
        // initialise the textures as a null array
        let mut result: [(wgpu::Texture, wgpu::TextureView); MLC] = unsafe { mem::zeroed() };
        // track the dimensions of the current texture
        let mut dim = (config.width, config.height);
        // add all the texture to the array
        for level in 0..MLC {
            // ammend the dimension
            dim = (dim.0 / 2, dim.1 / 2);
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
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                sample_count: 1,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            result[level] = (texture, texture_view);
        }
        result
    }

    fn create_bind_group(
        device: &wgpu::Device,
        label: Option<&str>,
        layout: &wgpu::BindGroupLayout,
        texture_view_to_read_from: &wgpu::TextureView,
        texture_view_to_write_to: &wgpu::TextureView,
        texture_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        return device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view_to_read_from),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture_view_to_write_to),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
            ],
        });
    }

    // this creates all the bind groups for all the downsampling shader calls
    fn create_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &[(wgpu::Texture, wgpu::TextureView); MLC],
        input_texture_view: &wgpu::TextureView,
        texture_sampler: &wgpu::Sampler,
    ) -> [wgpu::BindGroup; MLC] {
        // initialise the bind groups as a null array
        let mut result: [wgpu::BindGroup; MLC] = unsafe { mem::zeroed() };
        // fist bind group processes the input texture into the first internal downsampling texture
        result[0] = Self::create_bind_group(
            device,
            Some(&format!("downsampling bind group {}", 0)),
            layout,
            input_texture_view,
            &textures[0].1,
            texture_sampler,
        );
        // add all the bind groups to the array
        for level in 1..MLC {
            // create the bind group
            // first binding is the texture to be read from in the shader
            // second binding is the texture to be written to in the shader
            let bind_group = Self::create_bind_group(
                device,
                Some(&format!("downsampling bind group {}", 0)),
                layout,
                &textures[level - 1].1,
                &textures[level].1,
                texture_sampler,
            );
            result[level] = bind_group;
        }
        result
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        // queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        input_texture_view: &wgpu::TextureView,
    ) {
        self.downsampling_textures = Self::create_textures(device, config);
        self.downsampling_bind_groups = Self::create_bind_groups(
            device,
            &self.downsampling_compute_pipeline.get_bind_group_layout(0),
            &self.downsampling_textures,
            input_texture_view,
            &self.texture_sampler,
        );
        // self.resolution_uniform = Self::create_resolution(queue, config, &self.resolution_uniform_buffer);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) {
        // let output_view = match optional_output_view {
        //     Some(output_view) => output_view,
        //     None => &self.output_texture_view,
        // };

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { ..Default::default() });

        compute_pass.set_pipeline(&self.downsampling_compute_pipeline);
        for level in 0..MLC {
            compute_pass.set_bind_group(0, &self.downsampling_bind_groups[level], &[]);
            let texture_size = self.downsampling_textures[level].0.size();
            compute_pass.dispatch_workgroups(texture_size.width, texture_size.height, 1);
        }
    }
}
