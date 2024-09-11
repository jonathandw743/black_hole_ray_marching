use std::iter;
use std::sync::Arc;
use wgpu::{Device, Instance, InstanceFlags, Queue, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalPosition;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Fullscreen;
use winit::{event::*, window::Window};

use crate::bloom::Bloom;
use crate::downsampling::{self, Downsampling};
use crate::gaussian_blur::GaussianBlur;
use crate::time_replacement::{Duration, Instant};
use crate::upsampling::Upsampling;
use std::thread::sleep;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::settings::{Settings, SettingsController};

use crate::scene::Scene;

const LEVELS: usize = 2;

pub struct State<'a> {
    // wgpu and winit setup
    pub window: Arc<Window>,
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,

    // pub size: winit::dpi::PhysicalSize<u32>,
    pub settings: Settings,
    pub settings_controller: SettingsController,

    pub scene: Scene,
    // pub blur: Blur,
    pub bloom: Bloom<{ LEVELS }>,
    pub downsampling: Downsampling<{ LEVELS }>,
    pub upsampling: Upsampling<{ LEVELS }>,

    pub gaussian_blur: GaussianBlur,

    // timing
    pub start_of_last_frame_instant: Instant,
    pub delta_time: Duration,

    pub prev_cursor_position: Option<PhysicalPosition<f64>>,
    pub cursor_position: Option<PhysicalPosition<f64>>,

    pub frame_number: u32,
}

impl State<'_> {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let settings = Settings::new();

        let settings_controller = SettingsController::new();

        surface.configure(&device, &config);

        let scene = Scene::new(&device, &queue, &config, true);

        // let blur = Blur::new(&device, &queue, &config, &scene.output_texture_view);

        let bloom = Bloom::new(&device, &config);

        let downsampling = Downsampling::new(&device, &config);
        let upsampling = Upsampling::new(&device, &config, &downsampling.textures);
        // time stuff

        let gaussian_blur = GaussianBlur::new(&device, &config);

        let last_frame_time = Instant::now();

        let delta_time = Duration::from_secs_f32(0.0);

        Self {
            surface,
            device,
            queue,
            config,
            // size,
            window,

            settings,
            settings_controller,

            scene,

            // blur,
            bloom,
            downsampling,
            upsampling,
            
            gaussian_blur,

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

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.scene.resize(&self.device, &self.queue, &self.config);
            self.bloom.resize(&self.device, &self.config);
            // self.downsampling.resize(&self.device, &self.config);
            self.downsampling.resize(&self.device, &self.config);

            self.gaussian_blur.resize(&self.device, &self.config);
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        self.scene.process_event(event, &self.queue);
        self.settings_controller.process_event(event);
        match event {
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
                // On macos the window needs to be redrawn manually after resizing
                let _ = self.render();
                false
            }
            WindowEvent::RedrawRequested => {
                self.update();
                let _ = self.render();
                self.sleep();
                false
            }
            &WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F11),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(_) = self.window.fullscreen() {
                    self.window.set_fullscreen(None);
                } else {
                    self.window
                        .set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
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
            // None,
            Some(&self.bloom.input_texture_view()),
            Some(&self.gaussian_blur.input_texture_view()),
            // Some(self.bloom.input_texture_view()),
            // None,
            // Some(&output_view),
            // Some(self.downsampling.input_texture_view()),
        );

        self.gaussian_blur.render(&mut encoder, Some(&output_view));

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

        self.window.request_redraw();

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
