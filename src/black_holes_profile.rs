use std::time::Duration;

use glam::vec3;

use crate::uniforms::{BlackHole, BlackHolesUniform};

#[derive(PartialEq)]
pub enum BlackHolesProfile {
    None,
    Single,
    Dual,
    Orbiting,
    Binary,
    BackAndForth,
}

impl BlackHolesProfile {
    pub fn create_black_holes_uniform<const N: usize>(&self) -> BlackHolesUniform<N> {
        match self {
            Self::None => BlackHolesUniform::new([]),
            Self::Single => BlackHolesUniform::new([BlackHole {
                pos: vec3(0.0, 0.0, 0.0),
                rs: 1.0,
                accretion_disk_size: 6.0,
            }]),
            Self::Dual => BlackHolesUniform::new([
                BlackHole {
                    pos: vec3(-5.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
                BlackHole {
                    pos: vec3(5.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
            ]),
            Self::Orbiting => BlackHolesUniform::new([
                BlackHole {
                    pos: vec3(0.0, 0.0, 0.0),
                    rs: 2.0,
                    accretion_disk_size: 6.0,
                },
                BlackHole {
                    pos: vec3(20.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
            ]),
            Self::Binary => BlackHolesUniform::new([
                BlackHole {
                    pos: vec3(-5.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
                BlackHole {
                    pos: vec3(5.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
            ]),
            Self::BackAndForth => BlackHolesUniform::new([
                BlackHole {
                    pos: vec3(0.0, 0.0, 0.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
                BlackHole {
                    pos: vec3(0.0, 0.0, -10.0),
                    rs: 1.0,
                    accretion_disk_size: 6.0,
                },
            ]),
            _ => BlackHolesUniform::new([]),
        }
    }
    pub fn update_black_holes_uniform<const N: usize>(
        &self,
        black_holes_uniform: &mut BlackHolesUniform<N>,
        t: Duration,
    ) {
        match self {
            Self::None => {},
            Self::Single => {
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(0)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(0.0, 0.0, 0.0);
                        Some(())
                    });
            }
            Self::Dual => {
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(0)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(-5.0, 0.0, 0.0);
                        Some(())
                    });
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(1)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(5.0, 0.0, 0.0);
                        Some(())
                    });
            },
            Self::Orbiting => {
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(0)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(0.0, 0.0, 0.0);
                        Some(())
                    });
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(1)
                    .and_then(|black_hole| {
                        black_hole.pos = 20.0 * vec3(t.as_secs_f32().cos(), 0.0, t.as_secs_f32().sin());
                        Some(())
                    });
            },
            Self::Binary => {
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(0)
                    .and_then(|black_hole| {
                        black_hole.pos = -5.0 * vec3(t.as_secs_f32().cos(), 0.0, t.as_secs_f32().sin());
                        Some(())
                    });
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(1)
                    .and_then(|black_hole| {
                        black_hole.pos = 5.0 * vec3(t.as_secs_f32().cos(), 0.0, t.as_secs_f32().sin());
                        Some(())
                    });
            },
            Self::BackAndForth => {
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(0)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(0.0, 0.0, 0.0);
                        Some(())
                    });
                let _ = black_holes_uniform
                    .black_holes
                    .get_mut(1)
                    .and_then(|black_hole| {
                        black_hole.pos = vec3(20.0 * t.as_secs_f32().sin(), 0.0, -10.0);
                        Some(())
                    });
            }
            _ => {}
        }
    }
}
