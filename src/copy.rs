use wgpu::include_wgsl;

pub struct Copy {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::ComputePipeline,
}

impl Copy {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("copy bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm, //config.format,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm, //config.format,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                },
            ],
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, source, destination);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("copy compute pipeline layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });

        let module = device.create_shader_module(include_wgsl!("copy-compute.wgsl"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("copy compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "cs_main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("copy compute bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(destination),
                },
            ],
        })
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
    ) {
        self.bind_group =
            Self::create_bind_group(device, &self.bind_group_layout, source, destination);
    }

    pub fn pass(&self, encoder: &mut wgpu::CommandEncoder, x: u32, y: u32) {
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("copy compute pass"),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            //pass.dispatch_workgroups(x, y, 1);
        }
    }
}
