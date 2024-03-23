use std::iter;
use wgpu::InstanceFlags;
use winit::dpi::PhysicalPosition;
use winit::{event::*, window::Window};

use crate::bloom::Bloom;
use crate::downsampling::{self, Downsampling};
use crate::time_replacement::{Duration, Instant};
use crate::upsampling::Upsampling;
use std::thread::sleep;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::settings::{Settings, SettingsController};

use crate::scene::Scene;

pub struct State {
    // wgpu and winit setup
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,

    pub settings: Settings,
    pub settings_controller: SettingsController,

    pub scene: Scene,
    // pub blur: Blur,
    pub bloom: Bloom<{ Self::LEVELS }>,
    pub downsampling: Downsampling<{ Self::LEVELS }>,
    pub upsampling: Upsampling<{ Self::LEVELS }>,

    // timing
    pub start_of_last_frame_instant: Instant,
    pub delta_time: Duration,

    pub prev_cursor_position: Option<PhysicalPosition<f64>>,
    pub cursor_position: Option<PhysicalPosition<f64>>,

    pub frame_number: u32,
}

impl State {
    const LEVELS: usize = 3;

    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let x = 3.0;

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

        let settings = Settings::new();

        let settings_controller = SettingsController::new();

        surface.configure(&device, &config);

        let scene = Scene::new(&device, &queue, &config, false);

        // let blur = Blur::new(&device, &queue, &config, &scene.output_texture_view);

        let bloom = Bloom::new(&device, &config);

        let downsampling = Downsampling::new(&device, &config);
        let upsampling = Upsampling::new(&device, &config, &downsampling.textures);
        // time stuff

        let last_frame_time = Instant::now();

        let delta_time = Duration::from_secs_f32(0.0);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,

            settings,
            settings_controller,

            scene,

            // blur,
            bloom,
            downsampling,
            upsampling,
            start_of_last_frame_instant: last_frame_time,
            delta_time,

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
            self.scene.resize(&self.device, &self.queue, &self.config);
            self.bloom.resize(&self.device, &self.config);
            // self.downsampling.resize(&self.device, &self.config);
            self.downsampling.resize(&self.device, &self.config);
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

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        [
            self.settings_controller.process_event(event),
            self.scene.process_event(event, &self.queue),
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
        self.scene.update(
            self.delta_time,
            self.prev_cursor_position,
            self.cursor_position,
            &self.queue,
        );
        self.prev_cursor_position = self.cursor_position;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_start = Instant::now();

        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scene Render Encoder"),
            });

        self.scene.render(
            &mut encoder,
            // Some(self.bloom.input_texture_view()),
            // Some(self.bloom.input_texture_view()),
            // None,
            Some(&output_view),
            // Some(self.downsampling.input_texture_view()),
            None,
        );

        // self.bloom.render(&mut encoder, Some(&output_view));
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
