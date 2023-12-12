use std::num::NonZeroU64;

use encase::{ShaderType, ShaderSize, internal::{WriteInto, Writer, SizeValue}};
use crate::uniformscontroller::{Increment, Opposite};
use winit::event::{WindowEvent, VirtualKeyCode, ElementState, KeyboardInput};
use encase::CalculateSizeFor;
use crate::settings::number_from_virtual_key_code;
use encase::{DynamicUniformBuffer, UniformBuffer};
use encase::*;

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
    // could change this into a more generic UniformBuffer thing like encase does
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize);
    fn size(&self) -> NonZeroU64;
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
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize) {
        let mut writer = Writer::new(&self.value, buffer, offset).unwrap();
        self.value.write_into(&mut writer);
    }
    fn size(&self) -> NonZeroU64 {
        self.value.size()
    }
}

// mental gymnastics ends

pub struct OtherUniforms<const N: usize> {
    pub positive_modifier_key_code: VirtualKeyCode,
    pub negative_modifier_key_code: VirtualKeyCode,
    pub other_uniforms: [OtherUniform; N],
    pub modifier_number_pressed: Option<usize>,
}

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