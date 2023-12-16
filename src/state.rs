use std::f32::consts::PI;

use std::iter;

use wgpu::{InstanceFlags};
use winit::dpi::{PhysicalPosition};
// use cgmath::num_traits::float;
use winit::{event::*, window::Window};

use glam::{vec2, Vec3};

use std::thread::sleep;
// use std::time::{Duration, Instant};
use crate::time_replacement::{Duration, Instant};

use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// mod texture;
use crate::texture::Texture;

// mod settings;
use crate::settings::{Settings, SettingsController};

use crate::uniforms::{CameraUniform};

// mod camera;
use crate::camera::{Camera, CameraController};

// mod vertex;
use crate::vertex::Vertex;

// mod vertices;
use crate::vertices::VERTICES;

// // mod indices;
use crate::indices::INDICES;

use crate::otheruniforms::*;
use crate::uniformscontroller::*;

use crate::podbool::*;

pub struct State {
    // wgpu and winit setup
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub render_pipeline: wgpu::RenderPipeline,

    // vertices and indices
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    pub settings: Settings,
    pub settings_controller: SettingsController,
    pub camera: Camera,
    pub camera_controller: CameraController,

    // uniforms
    // pub camera_view_proj_uniform: CameraViewProjUniform,
    // pub camera_view_proj_buffer: wgpu::Buffer,
    // pub camera_pos_uniform: CameraPosUniform,
    // pub camera_pos_buffer: wgpu::Buffer,
    // pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform: CameraUniform,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,

    // pub uniform_controller_group: UniformControllerGroup<6>,
    pub other_uniforms: OtherUniforms<6>,
    pub other_uniforms_buffer: wgpu::Buffer,

    pub diffuse_bind_group: wgpu::BindGroup,

    // timing
    pub start_of_last_frame_instant: Instant,
    pub delta_time: Duration,
    pub should_render: bool,

    pub prev_cursor_position: Option<PhysicalPosition<f64>>,
    pub cursor_position: Option<PhysicalPosition<f64>>,

    pub frame_number: u32,
    // pub buffer5: encase::UniformBuffer<Vec<u8>>,
}

impl State {
    pub async fn new(window: Window, background_image: &[u8]) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raymarching shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./black_hole_maybe.wgsl").into()),
            // source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/raymarching.wgsl").into()),
        });

        //or let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // setings

        let settings = Settings::new();

        let settings_controller = SettingsController::new();

        // camera

        let camera = Camera {
            pos: (0.0, 0.0, -20.0).into(),
            dir: (0.0, 0.0, 1.0).into(),
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: PI * 0.4,
            znear: 0.1,
            zfar: 100.0,
        };

        // camera controller

        let camera_controller = CameraController::new(5.0, 0.5);

        // camera uniform buffer

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camer uniforms"),
            contents: &camera_uniform.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // other uniforms
        let other_uniforms = OtherUniforms::new(
            VirtualKeyCode::PageUp,
            VirtualKeyCode::PageDown,
            [
                OtherUniform {
                    label: "swartschild radius".into(),
                    // shader_stage: ShaderStages::FRAGMENT,
                    inc_value: Box::new(IncValue { value: 1.0, inc: 0.2 }),
                },
                OtherUniform {
                    label: "max delta time".into(),
                    inc_value: Box::new(IncValue { value: 0.3, inc: 0.02 }),
                },
                OtherUniform {
                    label: "background brightness".into(),
                    inc_value: Box::new(IncValue { value: 0.5, inc: 0.1 }),
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
                    inc_value: Box::new(IncValue { value: 50.0, inc: 1.0 }),
                },
                OtherUniform {
                    label: "distortion power".into(),
                    inc_value: Box::new(IncValue { value: 1.0, inc: 0.2 }),
                },
            ],
        );

        let data = other_uniforms.uniform_buffer_content();
        let mut d: [u8; 1284] = unsafe { std::mem::zeroed() };
        for (i, x) in data.iter().enumerate() {
            d[i] = *x;
        }

        let other_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camer uniforms"),
            contents: &other_uniforms.uniform_buffer_content(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // uniform bind group

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

        // diffuse texture stuff

        surface.configure(&device, &config);
        surface.configure(&device, &config);
        let diffuse_bytes = background_image;
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "space").unwrap();

        let diffuse_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // render pipeline

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &diffuse_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        // vertex and index buffers

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

        // time stuff

        let last_frame_time = Instant::now();

        let delta_time = Duration::from_secs_f32(0.0);

        let should_render = true;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,

            settings,
            settings_controller,
            camera,
            camera_controller,
            camera_uniform,
            camera_uniform_buffer,

            uniform_bind_group,

            other_uniforms,
            other_uniforms_buffer,

            diffuse_bind_group,

            start_of_last_frame_instant: last_frame_time,
            delta_time,
            should_render,

            prev_cursor_position: None,
            cursor_position: None,

            frame_number: 0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        }
    }

    fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            &WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
                true
            }
            _ => false,
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // allow controllers to process the event
        let other_uniforms_event_result = self.other_uniforms.process_event(event);
        if other_uniforms_event_result {
            self.queue.write_buffer(
                &self.other_uniforms_buffer,
                0,
                &self.other_uniforms.uniform_buffer_content(),
            );
        }
        [
            self.settings_controller.process_event(event),
            other_uniforms_event_result,
            self.camera_controller.process_event(event),
            self.process_event(event),
        ]
        .iter()
        .any(|&result| result)
    }

    pub fn update(&mut self) {
        self.delta_time = self.start_of_last_frame_instant.elapsed();
        self.start_of_last_frame_instant += self.delta_time;
        // update controllers
        self.settings_controller.update_settings(&mut self.settings);
        self.should_render = self.camera_controller.update_camera(
            &mut self.camera,
            self.delta_time,
            match (self.prev_cursor_position, self.cursor_position) {
                (Some(prev), Some(curr)) => vec2(curr.x as f32, curr.y as f32) != vec2(prev.x as f32, prev.y as f32),
                _ => false,
            },
        );
        self.should_render = true;
        // camera update camera uniform and buffer
        self.camera_uniform.update(&self.camera);

        let data = self.camera_uniform.uniform_buffer_content();
        self.queue.write_buffer(&self.camera_uniform_buffer, 0, &data);
        self.prev_cursor_position = self.cursor_position;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.should_render {
            return Ok(());
        }

        let render_start = Instant::now();

        // the output texture for the render
        // let output = self.surface.get_current_texture()?;
        let output = self.surface.get_current_texture()?;
        let render_time = Instant::now() - render_start;
        if self.frame_number % 100 == 0 {
            dbg!(render_time);
        }

        // a way to access this output texture
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(iter::once(encoder.finish()));

        output.present();

        self.frame_number += 1;

        Ok(())
    }

    pub fn sleep(&mut self) {
        let current_frame_duration = self.start_of_last_frame_instant.elapsed();
        if let Some(max_frame_rate) = self.settings.max_frame_rate {
            #[cfg(not(target_arch = "wasm32"))] // can't sleep normally in wasm
            {
                let min_frame_duration = Duration::from_secs_f32(1.0 / max_frame_rate);
                if current_frame_duration < min_frame_duration {
                    let sleep_duration = min_frame_duration - current_frame_duration;
                    sleep(sleep_duration);
                }
            }
        }
    }
}
