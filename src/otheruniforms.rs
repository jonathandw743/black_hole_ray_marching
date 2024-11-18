use std::{borrow::BorrowMut, fmt::Debug, num::NonZeroU64};

use bytemuck::{checked, Pod};
use egui::{Context, LayerId, Response, Ui};
use encase::{
    internal::{WriteInto, Writer},
    ShaderType,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::settings::number_from_virtual_key_code;

// ----------------------------------------------------------------------------------------------
//                                         Buffer Content Trait
// ----------------------------------------------------------------------------------------------

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

// ----------------------------------------------------------------------------------------------
//                                         Other Uniform
// ----------------------------------------------------------------------------------------------

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

pub trait OtherUniformTrait {
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize);
    fn size(&self) -> NonZeroU64;
    fn label(&self) -> &str;
}

// ----------------------------------------------------------------------------------------------
//                                         Gui Other Uniforms
// ----------------------------------------------------------------------------------------------

pub struct GuiOtherUniform<T> {
    pub other_uniform: OtherUniform<T>,
    pub ui_fn: Box<dyn FnMut(&mut Self, &mut Ui) -> ()>,
}

impl<T> OtherUniformTrait for GuiOtherUniform<T>
where
    T: ShaderType + WriteInto,
{
    fn label(&self) -> &str {
        &self.other_uniform.label
    }
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize) {
        let mut writer = Writer::new(&self.other_uniform.value, buffer, offset).unwrap();
        self.other_uniform.value.write_into(&mut writer);
    }
    fn size(&self) -> NonZeroU64 {
        self.other_uniform.value.size()
    }
}

impl GuiOtherUniform<f32> {
    pub fn new(other_uniform: OtherUniform<f32>, min: f32, max: f32) -> Self {
        Self {
            other_uniform,
            ui_fn: Box::new(move |s: &mut Self, ui: &mut Ui| {
                let _ = ui.label(&s.other_uniform.label);
                let _ = ui.add(egui::Slider::new(&mut s.other_uniform.value, min..=max));
            })
        }
    }
}

impl GuiOtherUniform<PodBool> {
    pub fn new(other_uniform: OtherUniform<PodBool>) -> Self {
        Self {
            other_uniform,
            ui_fn: Box::new(move |s: &mut Self, ui: &mut Ui| {
                let mut checked = s.other_uniform.value.get();
                let _ = ui.checkbox(&mut checked, &s.other_uniform.label);
                s.other_uniform.value.set(checked);
            })
        }
    }
}

impl<T> WriteInto for GuiOtherUniform<T>
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

pub trait GuiTrait {
    fn ui(&mut self, ui: &mut Ui);
}

impl<T> GuiTrait for GuiOtherUniform<T> {
    fn ui(&mut self, ui: &mut Ui) {
        // Temporarily move `ui_fn` out of `self` to avoid borrowing conflicts
        let mut ui_fn = std::mem::replace(&mut self.ui_fn, Box::new(|_, _| ()));
        // Call `ui_fn` and replace it back into `self`
        ui_fn(self, ui);
        self.ui_fn = ui_fn;
    }
}

pub trait OtherUniformGuiTrait: OtherUniformTrait + GuiTrait {}
impl<T> OtherUniformGuiTrait for T where T: OtherUniformTrait + GuiTrait {}

pub struct GuiOtherUniforms<const N: usize> {
    pub other_uniforms: [Box<dyn OtherUniformGuiTrait>; N],
}

impl<const N: usize> GuiOtherUniforms<N> {
    pub fn new(other_uniforms: [Box<dyn OtherUniformGuiTrait>; N]) -> Self {
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
    pub fn ui(&mut self, ui: &mut Ui) {
        for other_uniform in &mut self.other_uniforms {
            other_uniform.ui(ui);
        }
    }
}

// ----------------------------------------------------------------------------------------------
//                                         Incrementable
// ----------------------------------------------------------------------------------------------

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
    T: Debug,
{
    fn increment(&mut self) {
        self.value = self.value.increment(&self.inc);
        #[cfg(feature = "logging")]
        println!("new value: {:?}", self.value);
    }
    fn decrement(&mut self) {
        self.value = self.value.increment(&self.inc.opposite());
        #[cfg(feature = "logging")]
        println!("new value: {:?}", self.value);
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

impl<T, I> OtherUniformTrait for IncrementableOtherUniform<T, I>
where
    T: ShaderType + WriteInto,
{
    fn label(&self) -> &str {
        &self.other_uniform.label
    }
    fn write_into_buffer(&self, buffer: &mut Vec<u8>, offset: usize) {
        let mut writer = Writer::new(&self.other_uniform.value.value, buffer, offset).unwrap();
        self.other_uniform.value.value.write_into(&mut writer);
    }
    fn size(&self) -> NonZeroU64 {
        self.other_uniform.value.value.size()
    }
}

pub trait IncrementableTrait {
    fn increment(&mut self);
    fn decrement(&mut self);
}

impl<T, I> IncrementableTrait for IncrementableOtherUniform<T, I>
where
    T: Increment<I>,
    I: Opposite<I>,
    T: Debug,
{
    fn increment(&mut self) {
        self.other_uniform.value.increment();
        #[cfg(feature = "logging")]
        println!("new value: {:?}", self.other_uniform.value.value);
    }
    fn decrement(&mut self) {
        self.other_uniform.value.decrement();
        #[cfg(feature = "logging")]
        println!("new value: {:?}", self.other_uniform.value.value);
    }
}

pub trait IncrementableOtherUniformTrait: OtherUniformTrait + IncrementableTrait {}
impl<T> IncrementableOtherUniformTrait for T where T: OtherUniformTrait + IncrementableTrait {}

pub struct IncrementableOtherUniforms<const N: usize> {
    pub other_uniforms: [Box<dyn IncrementableOtherUniformTrait>; N],
}

impl<const N: usize> IncrementableOtherUniforms<N> {
    pub fn new(other_uniforms: [Box<dyn IncrementableOtherUniformTrait>; N]) -> Self {
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

// ----------------------------------------------------------------------------------------------
//                                         Incrementable Controller
// ----------------------------------------------------------------------------------------------

pub struct IncrementableOtherUniformsControllerKeyboard {
    pub positive_modifier_key_code: KeyCode,
    pub negative_modifier_key_code: KeyCode,
    pub modifier_number_pressed: Option<usize>,
}

impl IncrementableOtherUniformsControllerKeyboard {
    pub fn new(positive_modifier_key_code: KeyCode, negative_modifier_key_code: KeyCode) -> Self {
        Self {
            positive_modifier_key_code,
            negative_modifier_key_code,
            modifier_number_pressed: None,
        }
    }
    pub fn process_event<const N: usize>(
        &mut self,
        event: &WindowEvent,
        other_uniforms: &mut IncrementableOtherUniforms<N>,
    ) -> bool {
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
                    #[cfg(feature = "logging")]
                    println!(
                        "{}",
                        match other_uniforms.other_uniforms.get(number) {
                            Some(other_uniform) => format!("{} selected", other_uniform.label()),
                            None => "nothing selected".into(),
                        }
                    );
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

// ----------------------------------------------------------------------------------------------
//                                         primitive implementations
// ----------------------------------------------------------------------------------------------

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

#[derive(Copy, Clone, ShaderType, Debug)]
pub struct PodBool {
    inner: u32,
}

impl PartialEq for PodBool {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
    fn ne(&self, other: &Self) -> bool {
        self.inner != other.inner
    }
}

impl PodBool {
    pub fn r#true() -> Self {
        Self { inner: 1 }
    }
    pub fn r#false() -> Self {
        Self { inner: 0 }
    }
    pub fn set(&mut self, value: bool) {
        if value {
            self.inner = 1;
        } else {
            self.inner = 0;
        }
    }
    pub fn get(&self) -> bool {
        if self.inner == 0 {
            false
        } else {
            true
        }
    }
}

impl From<bool> for PodBool {
    fn from(value: bool) -> Self {
        if value {
            PodBool::r#true()
        } else {
            PodBool::r#false()
        }
    }
}

impl From<PodBool> for bool {
    fn from(value: PodBool) -> Self {
        if value.inner == 0 {
            false
        } else {
            true
        }
    }
}

impl std::fmt::Display for PodBool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Increment<PodBool> for PodBool {
    fn increment(&self, other: &PodBool) -> Self {
        (self.get() ^ other.get()).into()
    }
}

impl Increment<bool> for PodBool {
    fn increment(&self, other: &bool) -> Self {
        (self.get() ^ other).into()
    }
}

impl<T> Opposite<T> for PodBool
where
    T: From<PodBool>,
{
    fn opposite(&self) -> T {
        self.to_owned().into()
    }
}
