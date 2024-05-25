// use cgmath::Zero;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};

use encase::ShaderType;

use crate::camera::Camera;

pub fn setup_uniform<T: bytemuck::Pod>(
    device: &wgpu::Device,
    uniform: T,
    visibility: wgpu::ShaderStages,
    name: Option<&str>,
) -> (T, wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: name,
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: name,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: name,
    });
    return (uniform, buffer, bind_group_layout, bind_group);
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ResolutionUniform {
    res: [f32; 2],
}

impl ResolutionUniform {
    pub fn new() -> Self {
        ResolutionUniform { res: [0.0; 2] }
    }
    pub fn update(&mut self, size: &PhysicalSize<u32>) {
        self.res[0] = size.width as f32;
        self.res[1] = size.height as f32;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OpticalDensityUniform {
    density: f32,
}

impl OpticalDensityUniform {
    pub fn new() -> Self {
        OpticalDensityUniform { density: 1.0 }
    }
    pub fn update(&mut self, new_density: f32) {
        self.density = new_density;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AntiAliasingUniform {
    number: f32,
}

impl AntiAliasingUniform {
    pub fn new() -> Self {
        AntiAliasingUniform { number: 0.0 }
    }
    pub fn update_number(&mut self, new_anti_ailiasing_number: f32) {
        self.number = new_anti_ailiasing_number;
    }
}

#[derive(ShaderType)]
pub struct CameraUniform {
    pos: Vec3,
    // view_proj: Mat4,
    // inverse_view_proj: Mat4,
    // has to be vec4 for correct array stride
    screen_space_screen_triangle: [Vec4; 3],
    pos_to_world_space_screen_triangle: [Vec4; 3],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            // view_proj: Mat4::IDENTITY,
            // inverse_view_proj: Mat4::IDENTITY,
            screen_space_screen_triangle: [
                vec4(3.0, 1.0, 0.0, 0.0),
                vec4(-1.0, 1.0, 0.0, 0.0),
                vec4(-1.0, -3.0, 0.0, 0.0),
            ],
            // screen_space_screen_triangle: vec2(0.0, 0.0),
            pos_to_world_space_screen_triangle: [Vec4::ZERO; 3],
        }
    }
    pub fn update(&mut self, camera: &Camera) {
        self.pos = camera.pos;
        // self.view_proj = camera.build_view_projection_matrix();
        // self.inverse_view_proj = self.view_proj.inverse();

        self.pos_to_world_space_screen_triangle = camera
            .pos_to_world_space_screen_triangle(
                self.screen_space_screen_triangle.map(|v| vec2(v.x, v.y)),
            )
            .map(|v| vec4(v.x, v.y, v.z, 0.0));
    }
}

// We need this for Rust to store our data correctly for the shaders
// #[repr(C)]
// This is so we can store this in a buffer
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[derive(ShaderType)]
pub struct CameraViewProjUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: Mat4,
}

impl CameraViewProjUniform {
    pub fn new() -> Self {
        // use cgmath::SquareMatrix;
        Self {
            view_proj: glam::Mat4::IDENTITY.into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraPosUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pos: [f32; 3],
    _padding: f32,
}

impl CameraPosUniform {
    pub fn new() -> Self {
        Self {
            pos: [0.0; 3],
            _padding: 0.0,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.pos = camera.pos.into();
    }
}

// // We need this for Rust to store our data correctly for the shaders
// #[repr(C)]
// // This is so we can store this in a buffer
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct SimpleFloatUniform {
//     value: f32,
// }

// impl SimpleFloatUniform {
//     pub fn new(value: f32) -> Self {
//         SimpleFloatUniform { value: 0.0 }
//     }
//     pub fn update(&mut self, value: f32) {
//         self.value = value;
//     }
// }
