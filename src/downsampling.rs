pub struct Downsampling<const LEVELS: usize> {
    pub texture_sampler: wgpu::Sampler,

    pub textures: Vec<(wgpu::Texture, wgpu::TextureView)>,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl<const LEVELS: usize> Downsampling<LEVELS> {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let downsampling_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let textures = Self::create_textures(device, config);

        let screen_triangle_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("screen_triangle.wgsl"));
        let downsample_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("downsample.wgsl"));

        let downsampling_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let downsampling_bind_groups = Self::create_bind_groups(
            device,
            &downsampling_bind_group_layout,
            &textures,
            &downsampling_texture_sampler,
        );

        let downsampling_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("downsampling pipeline layout"),
                bind_group_layouts: &[&downsampling_bind_group_layout],
                push_constant_ranges: &[],
            });

        let downsampling_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scene Pipeline"),
                layout: Some(&downsampling_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &screen_triangle_shader_module,
                    entry_point: "main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &downsample_shader_module,
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
                cache: None
            });

        Self {
            texture_sampler: downsampling_texture_sampler,

            textures,

            bind_group_layout: downsampling_bind_group_layout,
            bind_groups: downsampling_bind_groups,
            render_pipeline: downsampling_render_pipeline,
        }
    }

    pub fn input_texture_view(&self) -> &wgpu::TextureView {
        &self.textures[0].1
    }

    //this creates all the levels of texture for downsampling
    fn create_textures(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Vec<(wgpu::Texture, wgpu::TextureView)> {
        // initialise the textures as a null array
        let mut result = Vec::new();
        // track the dimensions of the current texture
        let mut dim = (config.width, config.height);
        // add all the texture to the array
        for level in 0..LEVELS {
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
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
    fn create_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &Vec<(wgpu::Texture, wgpu::TextureView)>,
        texture_sampler: &wgpu::Sampler,
    ) -> Vec<wgpu::BindGroup> {
        // initialise the bind groups as a null array
        let mut result = Vec::new();
        // add all the bind groups to the array
        for level in 0..LEVELS {
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

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.textures = Self::create_textures(device, config);
        self.bind_groups = Self::create_bind_groups(
            device,
            &self.bind_group_layout,
            &self.textures,
            &self.texture_sampler,
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: Option<&wgpu::TextureView>,
    ) {
        for level in 1..LEVELS {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures[level].1,
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_groups[level - 1], &[]);
            render_pass.draw(0..3, 0..1);
        }

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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_groups[LEVELS - 1], &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
