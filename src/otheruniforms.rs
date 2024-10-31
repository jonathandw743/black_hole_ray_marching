use std::{fmt::Debug, num::NonZeroU64};

use egui::{Response, Ui};
use encase::{
    internal::{WriteInto, Writer},
    ShaderType,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::settings::number_from_virtual_key_code;


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

pub struct OtherUniform<T> {
    pub label: String,
    pub value: T,
}

impl<T> WriteInto for OtherUniform<T>
where
    T: WriteInto,
{
    fn write_into<B>(&self, writer: &mut Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        self.value.write_into(writer);
    }
}

// #[derive(Debug)]
pub struct IncValue<T, I> {
    pub value: T,
    pub inc: I,
}

impl<T, I> WriteInto for IncValue<T, I>
where
    T: WriteInto,
{
    fn write_into<B>(&self, writer: &mut Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        self.value.write_into(writer);
    }
}

impl<T, I> IncValue<T, I>
where
    T: Increment<I>,
    I: Opposite<I>,
{
    fn increment(&mut self) {
        self.value = self.value.increment(&self.inc);
        // println!("new value: {:?}", self.value);
    }
    fn decrement(&mut self) {
        self.value = self.value.increment(&self.inc.opposite());
        // println!("new value: {:?}", self.value);
    }
}

pub struct IncrementableOtherUniform<T, I> {
    pub other_uniform: OtherUniform<IncValue<T, I>>,
}

impl<T, I> WriteInto for IncrementableOtherUniform<T, I>
where
    T: WriteInto,
{
    fn write_into<B>(&self, writer: &mut Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        self.other_uniform.write_into(writer);
    }
}

pub trait Foo {
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize);
    fn size(&self) -> NonZeroU64;
    fn increment(&mut self);
    fn decrement(&mut self);
}

impl<T, I> Foo for IncrementableOtherUniform<T, I>
where
    T: ShaderType + WriteInto,
    T: Increment<I>,
    I: Opposite<I>,
    
{
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize) {
        let mut writer = Writer::new(&self.other_uniform.value.value, buffer, offset).unwrap();
        self.other_uniform.write_into(&mut writer);
    }
    fn size(&self) -> NonZeroU64 {
        self.other_uniform.value.value.size()
    }
    fn increment(&mut self) {
        self.other_uniform.value.increment();
        // println!("new value: {:?}", self.value);
    }
    fn decrement(&mut self) {
        self.other_uniform.value.decrement();
        // println!("new value: {:?}", self.value);
    }
}

pub struct IncrementableOtherUniforms<const N: usize> {
    pub other_uniforms: [Box<dyn Foo>; N],
}

impl<const N: usize> IncrementableOtherUniforms<N> {
    pub fn new(other_uniforms: [Box<dyn Foo>; N]) -> Self {
        Self { other_uniforms }
    }
    pub fn uniform_buffer_content(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut pos = 0;
        for other_uniform in &self.other_uniforms {
            {
                other_uniform.write_into_buffer(&mut buffer, pos);
                pos += other_uniform.size().get() as usize;
            }
        }
        for _i in buffer.len()..((buffer.len() as f32 / 16.0).ceil() * 16.0) as usize {
            buffer.push(0u8);
        }
        buffer
    }
}

pub struct IncrementableOtherUniformsControllerKeyboard {
    pub positive_modifier_key_code: KeyCode,
    pub negative_modifier_key_code: KeyCode,
    pub modifier_number_pressed: Option<usize>,
}

impl IncrementableOtherUniformsControllerKeyboard {
    pub fn new(
        positive_modifier_key_code: KeyCode,
        negative_modifier_key_code: KeyCode,
    ) -> Self {
        Self {
            positive_modifier_key_code,
            negative_modifier_key_code,
            modifier_number_pressed: None,
        }
    }
    pub fn process_event<const N: usize>(&mut self, event: &WindowEvent, other_uniforms: &mut IncrementableOtherUniforms<N>) -> bool {
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
                    // println!(
                    //     "{}",
                    //     match other_uniforms.get(number) {
                    //         Some(other_uniform) => format!("{} selected", other_uniform.label),
                    //         None => "nothing selected".into(),
                    //     }
                    // );
                    return true;
                }
                if let Some(modifier_number) = self.modifier_number_pressed {
                    if modifier_number < N {
                        if *code == self.positive_modifier_key_code {
                            other_uniforms.other_uniforms[modifier_number].increment();
                            return true;
                        }
                        if *code == self.negative_modifier_key_code {
                            other_uniforms.other_uniforms[modifier_number].decrement();
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


// mental gymnastics ends

// pub struct OtherUniformsK<const N: usize> {
//     pub positive_modifier_key_code: KeyCode,
//     pub negative_modifier_key_code: KeyCode,
//     pub other_uniforms: [OtherUniform; N],
//     pub modifier_number_pressed: Option<usize>,
// }

// impl<const N: usize> OtherUniformsK<N> {
//     pub fn new(
//         positive_modifier_key_code: KeyCode,
//         negative_modifier_key_code: KeyCode,
//         other_uniforms: [OtherUniform; N],
//     ) -> Self {
//         Self {
//             positive_modifier_key_code,
//             negative_modifier_key_code,
//             other_uniforms,
//             modifier_number_pressed: None,
//         }
//     }
//     pub fn uniform_buffer_content(&self) -> Vec<u8> {
//         let mut buffer: Vec<u8> = Vec::new();
//         let mut pos = 0;
//         for other_uniform in &self.other_uniforms {
//             {
//                 other_uniform.inc_value.write_into_buffer(&mut buffer, pos);
//                 pos += other_uniform.inc_value.size().get() as usize;
//             }
//         }
//         for _i in buffer.len()..((buffer.len() as f32 / 16.0).ceil() * 16.0) as usize {
//             buffer.push(0u8);
//         }
//         buffer
//     }
//     pub fn process_event(&mut self, event: &WindowEvent) -> bool {
//         match event {
//             WindowEvent::KeyboardInput {
//                 event:
//                     KeyEvent {
//                         physical_key: PhysicalKey::Code(code),
//                         state,
//                         ..
//                     },
//                 ..
//             } => {
//                 let is_pressed = match state {
//                     ElementState::Pressed => true,
//                     ElementState::Released => false,
//                 };
//                 if !is_pressed {
//                     return false;
//                 }
//                 if let Some(number) = number_from_virtual_key_code(code) {
//                     self.modifier_number_pressed = Some(number);
//                     println!(
//                         "{}",
//                         match self.other_uniforms.get(number) {
//                             Some(other_uniform) => format!("{} selected", other_uniform.label),
//                             None => "nothing selected".into(),
//                         }
//                     );
//                     return true;
//                 }
//                 if let Some(modifier_number) = self.modifier_number_pressed {
//                     if modifier_number < N {
//                         if *code == self.positive_modifier_key_code {
//                             self.other_uniforms[modifier_number].inc_value.increment();
//                             return true;
//                         }
//                         if *code == self.negative_modifier_key_code {
//                             self.other_uniforms[modifier_number].inc_value.decrement();
//                             return true;
//                         }
//                     }
//                 }
//                 false
//             }
//             _ => false,
//         }
//     }
// }
