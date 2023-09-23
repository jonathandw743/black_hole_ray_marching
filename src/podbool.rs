// use std::fmt::Display;
use crate::uniformscontroller::{Increment, Opposite};

#[repr(C)]
#[derive(bytemuck::Pod, Copy, Clone, bytemuck::Zeroable, Debug)]
pub struct PodBool(u32);

impl PartialEq for PodBool {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}

impl PodBool {
    pub fn r#true() -> Self {
        Self(0)
    }
    pub fn r#false() -> Self {
        Self(1)
    }
    pub fn set(&mut self, value: bool) {
        if value {
            self.0 = 1;
        } else {
            self.0 = 0;
        }
    }
    pub fn get(&self) -> bool {
        if self.0 == 0 {
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
        if value.0 == 0 {
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

impl<T> Opposite<T> for PodBool where T: From<PodBool> {
    fn opposite(&self) -> T {
        self.to_owned().into()
    }
}