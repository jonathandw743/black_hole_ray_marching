use std::{fmt::Debug, num::NonZeroU64};

use crate::uniformscontroller::{Increment, Opposite};
use encase::{
    internal::{WriteInto, Writer},
    ShaderType,
};
use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::settings::number_from_virtual_key_code;

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

#[derive(Debug)]
pub struct IncValue<T, I>
where
    T: Increment<I>,
    I: Opposite<I>,
    T: ShaderType + WriteInto,
{
    pub value: T,
    pub inc: I,
}

pub trait IncValueTrait: Debug {
    fn increment(&mut self);
    fn decrement(&mut self);
    // could change this into a more generic UniformBuffer thing like encase does
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize);
    fn size(&self) -> NonZeroU64;
}

impl<T, I> IncValueTrait for IncValue<T, I>
where
    T: Increment<I> + Debug,
    I: Opposite<I> + Debug,
    T: ShaderType + WriteInto,
{
    fn increment(&mut self) {
        self.value = self.value.increment(&self.inc);
        println!("new value: {:?}", self.value);
    }
    fn decrement(&mut self) {
        self.value = self.value.increment(&self.inc.opposite());
        println!("new value: {:?}", self.value);
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
    pub positive_modifier_key_code: KeyCode,
    pub negative_modifier_key_code: KeyCode,
    pub other_uniforms: [OtherUniform; N],
    pub modifier_number_pressed: Option<usize>,
}

impl<const N: usize> OtherUniforms<N> {
    pub fn new(
        positive_modifier_key_code: KeyCode,
        negative_modifier_key_code: KeyCode,
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
        for _i in buffer.len()..((buffer.len() as f32 / 16.0).ceil() * 16.0) as usize {
            buffer.push(0u8);
        }
        buffer
    }
    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
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
                        if *code == self.positive_modifier_key_code {
                            self.other_uniforms[modifier_number].inc_value.increment();
                            return true;
                        }
                        if *code == self.negative_modifier_key_code {
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
