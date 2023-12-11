use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::ops::{Add, Sub};

use encase::internal::{WriteInto, Writer};
use encase::{DynamicUniformBuffer, ShaderType, UniformBuffer};
// use cgmath::num_traits::float;
use wgpu::{Queue, ShaderStages};
use winit::{event::*, window::Window};

use std::thread::sleep;
use std::time::{Duration, Instant};

use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// mod texture;
use crate::texture::Texture;

// mod settings;
use crate::settings::{number_from_virtual_key_code, Settings, SettingsController};

use crate::uniforms::{CameraPosUniform, CameraViewProjUniform, OpticalDensityUniform};

// mod camera;
use crate::camera::{Camera, CameraController};

// mod vertex;
use crate::vertex::Vertex;

// mod vertices;
use crate::vertices::VERTICES;

// // mod indices;
use crate::indices::INDICES;

pub struct UniformAndBuffer<UniformType>
where
    UniformType: bytemuck::Pod + std::fmt::Display,
{
    pub label: String,
    pub value: UniformType,
    pub buffer: wgpu::Buffer,
    pub shader_stage: wgpu::ShaderStages,
}

impl<UniformType: bytemuck::Pod + std::fmt::Display> UniformAndBuffer<UniformType> {
    pub fn new(label: String, value: UniformType, device: &wgpu::Device, shader_stage: wgpu::ShaderStages) -> Self {
        Self {
            label: label.to_owned(),
            value,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&label.to_owned()),
                contents: bytemuck::cast_slice(&[value]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            shader_stage,
        }
    }
    pub fn update(&mut self, new_value: UniformType, queue: &wgpu::Queue, logging: bool) {
        self.value = new_value;
        if logging {
            println!("{} = {}", self.label, new_value);
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.value]));
    }
}

pub trait Opposite<T> {
    fn opposite(&self) -> T;
}

pub trait Increment<T> {
    fn increment(&self, other: &T) -> Self;
    // fn decrement(&self, other: Self) -> Self;
}

impl Opposite<Self> for f32 {
    fn opposite(&self) -> Self {
        -self
    }
}

impl Increment<Self> for f32 {
    fn increment(&self, other: &Self) -> Self {
        self + other
    }
    // fn decrement(&self, other: Self) -> Self {
    //     self.sub(other)
    // }
}

impl Opposite<Self> for bool {
    fn opposite(&self) -> Self {
        self.to_owned()
    }
}

impl Increment<Self> for bool {
    fn increment(&self, other: &Self) -> Self {
        self ^ other
    }
}

impl Opposite<i32> for usize {
    fn opposite(&self) -> i32 {
        -(*self as i32)
    }
}

impl Increment<i32> for usize {
    fn increment(&self, other: &i32) -> Self {
        ((*self as i32) + other) as usize
    }
}

impl Opposite<Self> for i32 {
    fn opposite(&self) -> Self {
        -self
    }
}

impl Increment<Self> for i32 {
    fn increment(&self, other: &Self) -> Self {
        self + other
    }
}

impl<UniformType> UniformAndBuffer<UniformType>
where
    UniformType: bytemuck::Pod + std::fmt::Display,
{
    fn increment<IncrementType>(&mut self, other: &IncrementType, queue: &wgpu::Queue, logging: bool)
    where
        UniformType: Increment<IncrementType>,
    {
        let new_value = self.value.increment(other);
        self.update(new_value, queue, logging);
    }
}

pub struct UniformController<UniformType, IncrementType>
where
    UniformType: bytemuck::Pod + std::fmt::Display + Increment<IncrementType>,
    IncrementType: Opposite<IncrementType>,
{
    pub uniform: UniformAndBuffer<UniformType>,
    pub increment: IncrementType,
    // pub decrement: UniformType,
    pub positive_modifier_key_code: VirtualKeyCode,
    pub negative_modifier_key_code: VirtualKeyCode,
    // pub
}

pub trait UniformControllerTrait {
    fn process_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue, logging: bool) -> bool;
    fn get_shader_stage(&self) -> wgpu::ShaderStages;
    fn get_buffer(&self) -> &wgpu::Buffer;
    fn get_label(&self) -> &String;
}

impl<UniformType, IncrementType> UniformControllerTrait for UniformController<UniformType, IncrementType>
where
    UniformType: bytemuck::Pod + std::fmt::Display + Increment<IncrementType>,
    IncrementType: Opposite<IncrementType>,
{
    fn process_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue, logging: bool) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_key_code),
                        ..
                    },
                ..
            } => {
                let is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                if !is_pressed {
                    return false;
                }
                if *virtual_key_code == self.positive_modifier_key_code {
                    self.uniform.increment(&self.increment, queue, logging);
                    return true;
                }
                if *virtual_key_code == self.negative_modifier_key_code {
                    self.uniform.increment(&self.increment.opposite(), queue, logging);
                    return true;
                }
                false
            }
            _ => false,
        }
    }
    fn get_shader_stage(&self) -> wgpu::ShaderStages {
        self.uniform.shader_stage
    }
    fn get_buffer(&self) -> &wgpu::Buffer {
        &self.uniform.buffer
    }
    fn get_label(&self) -> &String {
        return &self.uniform.label;
    }
}

pub struct UniformControllerGroup<const N: usize> {
    uniform_controllers: [Box<dyn UniformControllerTrait>; N],
    modifier_number_pressed: Option<usize>,
    logging: bool,
}

impl<const N: usize> UniformControllerGroup<N> {
    pub fn new(uniform_controllers: [Box<dyn UniformControllerTrait>; N], logging: bool) -> Self {
        if logging {
            println!("new uniform controller group with:");
            for (i, uniform_controller) in uniform_controllers.iter().enumerate() {
                println!("{} for {}", i, uniform_controller.get_label());
            }
        }
        Self {
            uniform_controllers,
            modifier_number_pressed: None,
            logging,
        }
    }
    pub fn bind_group(&self, device: &wgpu::Device, label: Option<&str>) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        // ! idk stfu
        let mut layout_entries: [wgpu::BindGroupLayoutEntry; N] = unsafe { std::mem::zeroed() };
        for (i, uniform_controller) in self.uniform_controllers.iter().enumerate() {
            layout_entries[i] = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: uniform_controller.get_shader_stage(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        }
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &layout_entries,
        });

        // ! idk stfu
        let mut entries: [wgpu::BindGroupEntry; N] = unsafe { std::mem::zeroed() };
        for (i, uniform_controller) in self.uniform_controllers.iter().enumerate() {
            entries[i] = wgpu::BindGroupEntry {
                binding: i as u32,
                resource: uniform_controller.get_buffer().as_entire_binding(),
            }
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label,
        });
        (layout, bind_group)
    }

    pub fn process_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_key_code),
                        ..
                    },
                ..
            } => {
                let is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                if !is_pressed {
                    return false;
                }
                if let Some(number) = number_from_virtual_key_code(virtual_key_code) {
                    self.modifier_number_pressed = Some(number);
                    return true;
                }
                if let Some(modifier_number) = self.modifier_number_pressed {
                    if modifier_number < N {
                        self.uniform_controllers[modifier_number].process_event(event, queue, self.logging);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

// mental gymnastics begins

pub trait BufferContent {
    fn storage_buffer_content(&self) -> Vec<u8>;
    fn uniform_buffer_content(&self) -> Vec<u8>;
}

impl<T> BufferContent for T
where
    T: ShaderType + WriteInto,
{
    fn storage_buffer_content(&self) -> Vec<u8> {
        let mut buffer = encase::StorageBuffer::new(Vec::new());
        buffer.write(self).unwrap();
        buffer.into_inner()
    }

    fn uniform_buffer_content(&self) -> Vec<u8> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self).unwrap();
        buffer.into_inner()
    }
}

pub struct OtherUniform {
    pub label: String,
    // pub shader_stage: ShaderStages,
    pub inc_value: Box<dyn IncValueTrait>,
}

pub struct IncValue<T, I>
where
    T: Increment<I>,
    I: Opposite<I>,
    T: ShaderType + WriteInto,
{
    pub value: T,
    pub inc: I,
}

pub trait IncValueTrait {
    fn increment(&mut self);
    fn decrement(&mut self);
    // fn uniform_buffer_content(&self) -> Vec<u8>;
    fn raw_data(&self) -> Vec<u8>;
    // could change this into a more generic UniformBuffer thing like encase does
    // fn write_into_buffer(&self, buffer: &mut DynamicUniformBuffer<Vec<u8>>);
    fn write_into(&self, writer: &mut Writer<&mut Vec<u8>>);
    fn size(&self) -> NonZeroU64;
    // fn new_writer(&self, buffer: &mut Vec<u8>, offset: usize) -> Writer<&mut Vec<u8>>;
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize);
}

impl<T, I> IncValueTrait for IncValue<T, I>
where
    T: Increment<I>,
    I: Opposite<I>,
    T: ShaderType + WriteInto,
{
    fn increment(&mut self) {
        self.value = self.value.increment(&self.inc);
    }
    fn decrement(&mut self) {
        self.value = self.value.increment(&self.inc.opposite());
    }
    // fn uniform_buffer_content(&self) -> Vec<u8> {
    //     self.value.uniform_buffer_content()
    // }
    fn raw_data(&self) -> Vec<u8> {
        let size = std::mem::size_of::<T>();
        let mut result = Vec::with_capacity(size);

        unsafe {
            let value_ptr = &self.value as *const T as *const u8;
            std::ptr::copy(value_ptr, result.as_mut_ptr(), size);
            result.set_len(size);
        }

        result
    }
    // fn write_into_buffer(&self, buffer: &mut DynamicUniformBuffer<Vec<u8>>) {
    //     // let x = buffer.write(&self.value);
    //     buffer.write(&self.value).unwrap();
    //     // dbg!(x);
    //     // let mut v: Vec<f32> = Vec::new();
    //     // println!("hello");
    //     // let v: Result<[glam::Vec4; 1], _> = buffer.create();
    //     // dbg!(v);
    // }
    fn write_into(&self, writer: &mut Writer<&mut Vec<u8>>) {
        self.value.write_into(writer);
    }
    fn size(&self) -> NonZeroU64 {
        self.value.size()
    }
    // fn new_writer(&self, buffer: &mut Vec<u8>, offset: usize) -> Writer<&mut Vec<u8>> {
    //     Writer::new(&self.value, buffer, offset).unwrap()
    // }
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize) {
        let mut writer = Writer::new(&self.value, buffer, offset).unwrap();
        self.value.write_into(&mut writer);
    }
}

// mental gymnastics ends

pub struct OtherUniforms<const N: usize> {
    pub positive_modifier_key_code: VirtualKeyCode,
    pub negative_modifier_key_code: VirtualKeyCode,
    pub other_uniforms: [OtherUniform; N],
    pub modifier_number_pressed: Option<usize>,
}

// impl<const N: usize> BufferContent for OtherUniforms<N> {
//     fn storage_buffer_content(&self) -> Vec<u8> {
//         let mut buffer = encase::StorageBuffer::new(self.raw_data());
//         buffer.write(&glam::Vec2::ZERO);
//         buffer.into_inner()
//     }
//     fn uniform_buffer_content(&self) -> Vec<u8> {
//         let mut buffer = encase::UniformBuffer::new(self.raw_data());
//         buffer.write(&glam::Vec2::ZERO);
//         buffer.into_inner()
//     }
// }

impl<const N: usize> OtherUniforms<N> {
    pub fn new(
        positive_modifier_key_code: VirtualKeyCode,
        negative_modifier_key_code: VirtualKeyCode,
        other_uniforms: [OtherUniform; N],
    ) -> Self {
        Self {
            positive_modifier_key_code,
            negative_modifier_key_code,
            other_uniforms,
            modifier_number_pressed: None,
        }
    }
    pub fn raw_data(&self) -> Vec<u8> {
        self.other_uniforms
            .iter()
            .flat_map(|other_uniform| other_uniform.inc_value.raw_data())
            .collect()
    }
    pub fn uniform_buffer_content(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut pos = 0;
        for other_uniform in &self.other_uniforms {
            {
                other_uniform.inc_value.write_into_buffer(&mut buffer, pos);
                pos += other_uniform.inc_value.size().get() as usize;
            }
        }
        buffer
    }
    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_key_code),
                        ..
                    },
                ..
            } => {
                let is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                if !is_pressed {
                    return false;
                }
                if let Some(number) = number_from_virtual_key_code(virtual_key_code) {
                    self.modifier_number_pressed = Some(number);
                    println!(
                        "{}",
                        match self.other_uniforms.get(number) {
                            Some(other_uniform) => format!("{} selected", other_uniform.label),
                            None => "nothing selected".into(),
                        }
                    );
                    return true;
                }
                if let Some(modifier_number) = self.modifier_number_pressed {
                    if modifier_number < N {
                        if *virtual_key_code == self.positive_modifier_key_code {
                            self.other_uniforms[modifier_number].inc_value.increment();
                            return true;
                        }
                        if *virtual_key_code == self.negative_modifier_key_code {
                            self.other_uniforms[modifier_number].inc_value.decrement();
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}

// pub struct UniformsController {
//     pub modifier_map: [bool; 10],
//     pub increment: f32,
// }

// impl UniformsController {
//     pub fn new(increment: f32) -> Self {
//         Self {
//             modifier_map: [false; 10],
//             increment,
//         }
//     }

//     pub fn process_event(
//         &mut self,
//         event: &WindowEvent,
//         uniforms: &mut Vec<UniformAndBuffer<f32>>,
//         queue: &wgpu::Queue,
//     ) -> bool {
//         match event {
//             WindowEvent::KeyboardInput {
//                 input:
//                     KeyboardInput {
//                         state,
//                         virtual_keycode: Some(virtual_key_code),
//                         ..
//                     },
//                 ..
//             } => {
//                 let is_pressed = match state {
//                     ElementState::Pressed => true,
//                     ElementState::Released => false,
//                 };
//                 if let Some(number) = number_from_virtual_key_code(virtual_key_code) {
//                     self.modifier_map[number] = is_pressed;
//                 }
//                 if !is_pressed {
//                     return false;
//                 }
//                 match virtual_key_code {
//                     VirtualKeyCode::PageUp => {
//                         for i in 0..std::cmp::min(self.modifier_map.len(), uniforms.len()) {
//                             if self.modifier_map[i] {
//                                 let value = uniforms[i].value + self.increment;
//                                 uniforms[i].update(value, queue, true);
//                             }
//                         }
//                         true
//                     }
//                     VirtualKeyCode::PageDown => {
//                         for i in 0..std::cmp::min(self.modifier_map.len(), uniforms.len()) {
//                             if self.modifier_map[i] {
//                                 let value = uniforms[i].value - self.increment;
//                                 uniforms[i].update(value, queue, true);
//                             }
//                         }
//                         true
//                     }
//                     _ => false,
//                 }
//             }
//             _ => false,
//         }
//     }
// }
