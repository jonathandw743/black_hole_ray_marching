use crate::kawase_blur::{KawaseDownsampling, KawaseUpsampling};

pub struct Bloom {
    pub full_image_input_texture: wgpu::Texture,
    pub full_image_input_texture_view: wgpu::TextureView,

    pub blurred_blackout_input_texture: wgpu::Texture,
    pub blurred_blackout_texture_view: wgpu::TextureView,

    pub downsampling: KawaseDownsampling,
    pub upsampling: KawaseUpsampling,

    pub final_remix_texture_sampler: wgpu::Sampler,

    pub final_remix_bind_group_layout: wgpu::BindGroupLayout,
    pub final_remix_bind_group: wgpu::BindGroup,
    pub final_remix_render_pipeline: wgpu::RenderPipeline,
}

impl Bloom {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let (full_image_input_texture, full_image_input_texture_view) = Self::create_input_texture(device, config);

        let downsampling = KawaseDownsampling::new(device, config);
        let upsampling = KawaseUpsampling::new(device, config);
        let final_remix_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let (blurred_blackout_input_texture, blurred_blackout_input_texture_view) =
            Self::create_input_texture(device, config);

        let screen_triangle_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("screen_triangle.wgsl"));
        let final_remix_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("final_remix.wgsl"));

        let final_remix_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("final remix bind group layout"),
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

        let final_remix_bind_group = Self::create_final_remix_bind_group(
            device,
            &final_remix_bind_group_layout,
            &full_image_input_texture_view,
            &upsampling.textures[0].1,
            &final_remix_texture_sampler,
        );

        let final_remix_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("final remix pipeline layout"),
                bind_group_layouts: &[&final_remix_bind_group_layout],
                push_constant_ranges: &[],
            });

        let final_remix_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("final remix Pipeline"),
                layout: Some(&final_remix_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &screen_triangle_shader_module,
                    entry_point: "main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &final_remix_shader_module,
                    entry_point: "main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
            full_image_input_texture,
            full_image_input_texture_view,

            blurred_blackout_input_texture,
            blurred_blackout_texture_view: blurred_blackout_input_texture_view,

            downsampling,
            upsampling,

            final_remix_texture_sampler,

            final_remix_bind_group_layout,
            final_remix_bind_group,
            final_remix_render_pipeline,
        }
    }

    pub fn full_image_input_texture_view(&self) -> &wgpu::TextureView {
        &self.full_image_input_texture_view
    }

    pub fn blackout_input_texture_view(&self) -> &wgpu::TextureView {
        &self.downsampling.input_texture_view()
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

    // this creates all the bind groups for the final re-mix
    fn create_final_remix_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        full_image_texture_view: &wgpu::TextureView,
        blurred_blackout_texture_view: &wgpu::TextureView,
        texture_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("final remix bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(full_image_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(blurred_blackout_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
            ],
        });
        bind_group
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, queue: &wgpu::Queue) {
        (self.full_image_input_texture, self.full_image_input_texture_view) = Self::create_input_texture(device, config);
        (
            self.blurred_blackout_input_texture,
            self.blurred_blackout_texture_view,
        ) = Self::create_input_texture(device, config);
        self.downsampling.resize(device, config, queue);
        self.upsampling
            .resize(device, config, queue);
        self.final_remix_bind_group = Self::create_final_remix_bind_group(
            device,
            &self.final_remix_bind_group_layout,
            &self.full_image_input_texture_view,
            &self.blurred_blackout_texture_view,
            &self.final_remix_texture_sampler,
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: Option<&wgpu::TextureView>,
    ) {
        self.downsampling
            .render(encoder, Some(self.upsampling.input_texture_view()));
        // self.upsampling.render(encoder, output_view);
        self.upsampling.render(encoder, Some(&self.blurred_blackout_texture_view));
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur render pass"),
                color_attachments: &[output_view.map(|output_view| {
                    wgpu::RenderPassColorAttachment {
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
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.final_remix_render_pipeline);
            render_pass.set_bind_group(0, &self.final_remix_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
