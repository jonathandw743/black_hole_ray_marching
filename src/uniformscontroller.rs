







// use cgmath::num_traits::float;

use winit::{event::*, keyboard::{KeyCode, PhysicalKey}};




use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// mod texture;


// mod settings;
use crate::settings::{number_from_virtual_key_code};



// mod camera;


// mod vertex;


// mod vertices;


// // mod indices;


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
    pub positive_modifier_key_code: KeyCode,
    pub negative_modifier_key_code: KeyCode,
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
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
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
                if *code == self.positive_modifier_key_code {
                    self.uniform.increment(&self.increment, queue, logging);
                    return true;
                }
                if *code == self.negative_modifier_key_code {
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
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
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
                if let Some(number) = number_from_virtual_key_code(code) {
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