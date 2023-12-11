// use std::fmt::Display;
use crate::uniformscontroller::{Increment, Opposite};
use encase::ShaderType;
// #[repr(C)]
// #[derive(bytemuck::Pod, Copy, Clone, bytemuck::Zeroable, Debug)]
#[derive(Copy, Clone)]
#[derive(ShaderType)]
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
        Self { inner: 0 }
    }
    pub fn r#false() -> Self {
        Self { inner: 1 }
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

// impl Opposite<PodBool> for PodBool {
//     fn opposite(&self) -> PodBool {
//         self.to_owned()
//     }
// }

// impl Opposite<bool> for PodBool {
//     fn opposite(&self) -> bool {
//         self.to_owned().into()
//     }
// }

impl<T> Opposite<T> for PodBool
where
    T: From<PodBool>,
{
    fn opposite(&self) -> T {
        self.to_owned().into()
    }
}

// impl<T> Opposite<T> for PodBool
// where
//     PodBool: Into<T>,
// {
//     fn opposite(&self) -> T {
//         self.to_owned().into()
//     }
// }
