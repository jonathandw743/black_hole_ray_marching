use std::{default, iter};

use wgpu::InstanceFlags;
use winit::dpi::PhysicalPosition;
use winit::{event::*, window::Window};

use crate::bloom::Bloom;
use std::thread::sleep;
//// use std::time::{Duration, Instant};
use crate::time_replacement::{Duration, Instant};

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
    pub bloom: Bloom<5>,

    // timing
    pub start_of_last_frame_instant: Instant,
    pub delta_time: Duration,

    pub prev_cursor_position: Option<PhysicalPosition<f64>>,
    pub cursor_position: Option<PhysicalPosition<f64>>,

    pub frame_number: u32,
}

impl State {
    pub async fn new(window: Window) -> Self {
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

        let settings = Settings::new();

        let settings_controller = SettingsController::new();

        surface.configure(&device, &config);

        let scene = Scene::new(&device, &queue, &config);

        // let blur = Blur::new(&device, &queue, &config, &scene.output_texture_view);

        let bloom = Bloom::new(&device, &config);

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
            start_of_last_frame_instant: last_frame_time,
            delta_time,

            prev_cursor_position: None,
            cursor_position: None,

            frame_number: 0,
        }
    }

    // pub fn create_scene_texture(
    //     device: &wgpu::Device,
    //     config: &SurfaceConfiguration,
    // ) -> (wgpu::Texture, wgpu::BindGroupLayout, wgpu::BindGroup) {
    //     let scene_texture = device.create_texture(&wgpu::TextureDescriptor {
    //         label: Some("Scene Texture"),
    //         size: wgpu::Extent3d {
    //             width: config.width,
    //             height: config.height,
    //             depth_or_array_layers: 1,
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: wgpu::TextureFormat::Bgra8UnormSrgb,
    //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT
    //             | wgpu::TextureUsages::TEXTURE_BINDING
    //             | wgpu::TextureUsages::COPY_SRC,
    //         view_formats: &[],
    //     });
    //
    //     let postprocessing_input_bind_group_layout =
    //         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //             entries: &[
    //                 wgpu::BindGroupLayoutEntry {
    //                     binding: 0,
    //                     visibility: wgpu::ShaderStages::FRAGMENT,
    //                     ty: wgpu::BindingType::Texture {
    //                         multisampled: false,
    //                         view_dimension: wgpu::TextureViewDimension::D2,
    //                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                     },
    //                     count: None,
    //                 },
    //                 wgpu::BindGroupLayoutEntry {
    //                     binding: 1,
    //                     visibility: wgpu::ShaderStages::FRAGMENT,
    //                     // This should match the filterable field of the
    //                     // corresponding Texture entry above.
    //                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
    //                     count: None,
    //                 },
    //             ],
    //             label: Some("postprocessing_input_bind_group_layout"),
    //         });
    //
    //     let postprocessing_input_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //         layout: &postprocessing_input_bind_group_layout,
    //         entries: &[
    //             wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: wgpu::BindingResource::TextureView(
    //                     &scene_texture.create_view(&wgpu::TextureViewDescriptor::default()),
    //                 ), // CHANGED!
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 1,
    //                 resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
    //                     address_mode_u: wgpu::AddressMode::ClampToEdge,
    //                     address_mode_v: wgpu::AddressMode::ClampToEdge,
    //                     address_mode_w: wgpu::AddressMode::ClampToEdge,
    //                     mag_filter: wgpu::FilterMode::Nearest,
    //                     min_filter: wgpu::FilterMode::Nearest,
    //                     mipmap_filter: wgpu::FilterMode::Nearest,
    //                     ..Default::default()
    //                 })), // CHANGED!
    //             },
    //         ],
    //         label: Some("postprocessing_input_bind_group"),
    //     });
    //
    //     return (
    //         scene_texture,
    //         postprocessing_input_bind_group_layout,
    //         postprocessing_input_bind_group,
    //     );
    // }

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

            // self.blur
            //     .resize(&self.device, &self.queue, &self.config, &self.scene.output_texture_view);
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
            Some(self.bloom.input_texture_view()),
            // Some(&output_view),
            Some(self.bloom.blackout_input_texture_view()),
            // None,
        );

        self.bloom.render(&mut encoder, Some(&output_view));

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

// fn save_texture_as_image(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, path: &str) {
//     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//         label: Some("Read Texture Encoder"),
//     });
//     encoder.copy_texture_to_buffer(
//         wgpu::ImageCopyTexture {
//             texture,
//             mip_level: 0,
//             origin: wgpu::Origin3d::ZERO,
//             aspect: wgpu::TextureAspect::All,
//         },
//         wgpu::ImageCopyBuffer {
//             buffer: &device.create_buffer(&wgpu::BufferDescriptor {
//                 label: Some("Texture Buffer"),
//                 size: (4 * texture.size().width * texture.size().height) as wgpu::BufferAddress,
//                 usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
//                 mapped_at_creation: false,
//             }),
//             layout: wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(4 * texture.size().width),
//                 rows_per_image: Some(texture.size().height),
//             },
//         },
//         texture.size(),
//     );
//     // Ensure the texture is ready for reading
//     queue.submit(iter::once(encoder.finish()));
//
//     let buffer = device.create_buffer(&wgpu::BufferDescriptor {
//         label: Some("Read Texture Buffer"),
//         size: (4 * texture.size().width * texture.size().height) as wgpu::BufferAddress,
//         usage: wgpu::BufferUsages::COPY_SRC,
//         mapped_at_creation: false,
//     });
//
//     // Map the buffer for reading
//     let buffer_slice = buffer.slice(..);
//
//     let buffer_view = buffer_slice.get_mapped_range();
//     let image = image::DynamicImage::ImageRgba8(
//         image::RgbaImage::from_raw(texture.size().width, texture.size().height, buffer_view.to_vec()).unwrap(),
//     );
//
//     // Save the image to a file
//     image.save(path).expect("Failed to save image file");
//
//     device.poll(wgpu::Maintain::Wait);
// }
