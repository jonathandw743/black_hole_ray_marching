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

#[derive(ShaderType)]
pub struct CameraUniform {
    pos: Vec3,
    screen_space_screen_triangle: [Vec4; 3],
    pos_to_world_space_screen_triangle: [Vec4; 3],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            screen_space_screen_triangle: [
                vec4(3.0, 1.0, 0.0, 0.0),
                vec4(-1.0, 1.0, 0.0, 0.0),
                vec4(-1.0, -3.0, 0.0, 0.0),
            ],
            pos_to_world_space_screen_triangle: [Vec4::ZERO; 3],
        }
    }
    pub fn update(&mut self, camera: &Camera) {
        self.pos = camera.pos;
        self.pos_to_world_space_screen_triangle = camera
            .pos_to_world_space_screen_triangle(
                self.screen_space_screen_triangle.map(|v| vec2(v.x, v.y)),
            )
            .map(|v| vec4(v.x, v.y, v.z, 0.0));
    }
}

#[derive(ShaderType, Debug, Copy, Clone)]
pub struct BlackHole {
    pub pos: Vec3,
    pub rs: f32,
}

#[derive(ShaderType)]
pub struct BlackHolesUniform<const N: usize> {
    pub black_holes: [BlackHole; N],
}

impl<const N: usize> BlackHolesUniform<N> {
    pub fn new<const M: usize>(black_holes_in: [BlackHole; M]) -> Self {
        let mut black_holes = [BlackHole {
            pos: vec3(0.0, 0.0, 0.0),
            rs: 0.0,
        }; N];
        for i in 0..M.min(N) {
            black_holes[i] = black_holes_in[i];
        }
        Self {
            black_holes
        }
    }
}
