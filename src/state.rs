use anyhow::{anyhow, Result};
use glam::uvec2;
use std::iter;
use std::sync::Arc;
use wgpu::{
    Device, Instance, InstanceFlags, Queue, Surface, SurfaceConfiguration, Texture,
    TextureViewDescriptor,
};
use winit::dpi::PhysicalPosition;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Fullscreen;
use winit::{event::*, window::Window};

use crate::copy::Copy;
//use crate::bloom::Bloom;
// use crate::bloom::Bloom;
// use crate::downsampling::{self, Downsampling};
// use crate::gaussian_blur::GaussianBlur;
use crate::kawase_downsampling::KawaseDownsampling;
use crate::time_replacement::{Duration, Instant};
// use crate::upsampling::Upsampling;
use std::thread::sleep;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::settings::{Settings, SettingsController};

use crate::scene::Scene;

pub struct State<'a> {
    // wgpu and winit setup
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,

    // pub size: winit::dpi::PhysicalSize<u32>,
    pub settings: Settings,
    pub settings_controller: SettingsController,

    pub scene: Scene,

    pub source: Texture,
    pub destination: Texture,
    pub copy: Copy,
    // pub blur: Blur,
    //pub bloom: Bloom,
    // pub downsampling: Downsampling<{ LEVELS }>,
    // pub upsampling: Upsampling<{ LEVELS }>,

    // pub gaussian_blur: GaussianBlur,

    // pub kawase_upsampling: KawaseUpsampling,
    // pub kawase_downsampling: KawaseDownsampling,

    // timing
    pub start_of_last_frame_instant: Instant,
    pub delta_time: Duration,

    pub prev_cursor_position: Option<PhysicalPosition<f64>>,
    pub cursor_position: Option<PhysicalPosition<f64>>,

    pub frame_number: u32,
}

impl State<'_> {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = Instance::default();

        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(anyhow!("Failed to find an appropriate adapter"))?;

        let mut limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        limits.max_storage_textures_per_shader_stage = 2;
        // limits.max_texture_dimension_1d = 10000;
        // limits.max_texture_dimension_2d = 10000;
        // limits.max_texture_dimension_3d = 1;
        limits.max_compute_workgroup_size_x = 1;
        limits.max_compute_workgroup_size_y = 1;
        limits.max_compute_workgroup_size_z = 1;
        limits.max_compute_invocations_per_workgroup = 1;
        // limits.max_compute_workgroup_storage_size = 10000;
        limits.max_compute_workgroups_per_dimension = 1280;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await.map_err(|_| anyhow!("Failed to request device"))?;

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(anyhow!("Failed to get surface configuration"))?;
        //config.format = wgpu::TextureFormat::Bgra8UnormSrgb;
        surface.configure(&device, &config);

        let settings = Settings::new();

        let settings_controller = SettingsController::new();

        surface.configure(&device, &config);

        let scene = Scene::new(&device, &queue, &config, false);
        let source = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("source"),
            mip_level_count: 1,
            size: wgpu::Extent3d {
                width: 1280,
                height: 270,
                depth_or_array_layers: 1,
            },
            format: wgpu::TextureFormat::Rgba8Unorm, //config.format,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            view_formats: &[],
        });
        let destination = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("destination"),
            mip_level_count: 1,
            size: wgpu::Extent3d {
                width: 1280,
                height: 270,
                depth_or_array_layers: 1,
            },
            format: wgpu::TextureFormat::Rgba8Unorm, //config.format,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            view_formats: &[],
        });
        let copy = Copy::new(
            &device,
            &config,
            &source.create_view(&wgpu::TextureViewDescriptor::default()),
            &destination.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        // let blur = Blur::new(&device, &queue, &config, &scene.output_texture_view);

        // let bloom = Bloom::new(&device, &config);

        // let downsampling = Downsampling::new(&device, &config);
        // let upsampling = Upsampling::new(&device, &config, &downsampling.textures);
        // time stuff

        // let gaussian_blur = GaussianBlur::new(&device, &config);

        // let kawase_downsampling = KawaseDownsampling::new(&device, &config);
        // let kawase_upsampling = KawaseUpsampling::new(&device, &config);

        //let bloom = Bloom::new(&device, &config, 3);

        let last_frame_time = Instant::now();

        let delta_time = Duration::from_secs_f32(0.0);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            // size,
            settings,
            settings_controller,

            scene,

            source,
            destination,
            copy,

            // blur,
            // bloom,
            // downsampling,
            // upsampling,

            // gaussian_blur,

            // kawase_upsampling,
            // kawase_downsampling,
            //bloom,
            start_of_last_frame_instant: last_frame_time,
            delta_time,

            prev_cursor_position: None,
            cursor_position: None,

            frame_number: 0,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.scene.resize(&self.device, &self.queue, &self.config);
            // self.bloom.resize(&self.device, &self.config);
            // self.downsampling.resize(&self.device, &self.config);
            // self.downsampling.resize(&self.device, &self.config);

            // self.kawase_downsampling.resize(&self.device, &self.config, &self.queue);
            // self.kawase_upsampling.resize(&self.device, &self.config, &self.queue);
            //self.bloom.resize(&self.device, &self.config, &self.queue);

            // self.gaussian_blur.resize(&self.device, &self.config);
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        self.scene.process_event(event, &self.queue);
        self.settings_controller.process_event(event);
        match event {
            &WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
                true
            }
            _ => false,
        }
    }

    // pub fn input(&mut self, event: &WindowEvent) -> bool {
    //     [
    //         self.settings_controller.process_event(event),
    //         self.scene.process_event(event, &self.queue),
    //         self.process_event(event),
    //     ]
    //     .iter()
    //     .any(|&result| result)
    // }

    pub fn update(&mut self) {
        // dbg!(self.prev_cursor_position, self.cursor_position);
        self.delta_time = self.start_of_last_frame_instant.elapsed();
        self.start_of_last_frame_instant += self.delta_time;
        // update controllers
        self.settings_controller.update_settings(&mut self.settings);
        self.scene.update(
            self.delta_time,
            self.prev_cursor_position,
            self.cursor_position,
            &self.queue,
        );
        self.prev_cursor_position = self.cursor_position;
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let render_start = Instant::now();

        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // dbg!(output.texture.width(), output.texture.height());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scene Render Encoder"),
            });

        self.scene.render(
            &mut encoder,
            //Some(&self.bloom.full_image_input_texture_view()),
            Some(&output_view),
            //Some(&self.bloom.blackout_input_texture_view()),
            None,
        );

        self.copy.pass(&mut encoder, 1280, 720);

        // self.kawase_downsampling.render(&mut encoder, Some(self.kawase_upsampling.input_texture_view()));
        // self.kawase_upsampling.render(&mut encoder, Some(&output_view));

        // self.gaussian_blur.render(&mut encoder, Some(&output_view));

        //self.bloom.render(&mut encoder, Some(&output_view));

        // self.downsampling
        // .render(&mut encoder, Some(self.upsampling.input_texture_view()));
        // self.downsampling.render(&mut encoder, Some(&output_view));
        // self.upsampling.render(&mut encoder, Some(&output_view));
        // self.upsampling.render(&mut encoder, Some(&output_view));

        self.queue.submit(iter::once(encoder.finish()));

        output.present();

        let render_time = Instant::now() - render_start;
        if self.frame_number % 100 == 0 {
            dbg!(render_time);
        }

        self.frame_number += 1;

        // window.request_redraw();

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
