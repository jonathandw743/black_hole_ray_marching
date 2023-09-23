use std::time::Duration;

use winit::event::*;

use cgmath::prelude::*;

use cgmath::{Quaternion, Rad};

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub dir: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn right(&self) -> cgmath::Vector3<f32> {
        self.up.cross(self.dir)
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // camera's position vector
        let p = self.pos;
        // up vector (normalized)
        let u = self.up.normalize();
        // forward vector (normalized)
        let f = self.dir.normalize();
        // right vector (normalized)
        let r = self.right();

        #[rustfmt::skip]
        let view = cgmath::Matrix4::new(
            r.x.clone(), u.x.clone(), -f.x.clone(), 0.0,
            r.y.clone(), u.y.clone(), -f.y.clone(), 0.0,
            r.z.clone(), u.z.clone(), -f.z.clone(), 0.0,
            -p.dot(r), -p.dot(u), p.dot(f), 1.0,
        );
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub struct CameraController {
    speed: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,

    pan_speed: f32,

    is_pan_up_pressed: bool,
    is_pan_down_pressed: bool,
    is_pan_left_pressed: bool,
    is_pan_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, pan_speed: f32) -> Self {
        Self {
            speed,

            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            pan_speed,

            is_pan_up_pressed: false,
            is_pan_down_pressed: false,
            is_pan_left_pressed: false,
            is_pan_right_pressed: false,
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                match virtual_keycode {
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Up => {
                        self.is_pan_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Left => {
                        self.is_pan_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_pan_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Right => {
                        self.is_pan_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::R => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::F => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: Duration) -> bool {
        let dt = delta_time.as_secs_f32();

        // println!("{:?}", dt * 0.000000001);

        let x_movement_norm = match (self.is_left_pressed, self.is_right_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let x_movement = dt * self.speed * x_movement_norm;
        
        let z_movement_norm = match (self.is_backward_pressed, self.is_forward_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let z_movement = dt * self.speed * z_movement_norm;
        
        let y_movement_norm = match (self.is_down_pressed, self.is_up_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let y_movement = dt * self.speed * y_movement_norm;
        
        let x_pan_norm = match (self.is_pan_left_pressed, self.is_pan_right_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let x_pan = dt * self.pan_speed * x_pan_norm;
        
        let y_pan_norm = match (self.is_pan_up_pressed, self.is_pan_down_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let y_pan = dt * self.pan_speed * y_pan_norm;

        camera.pos += x_movement * camera.right();
        camera.pos += z_movement * camera.dir;
        camera.pos += y_movement * camera.up;

        let rotation = Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), Rad(x_pan));
        camera.dir = rotation.rotate_vector(camera.dir);
        camera.up = rotation.rotate_vector(camera.up);

        // println!("{:?}", camera.dir);

        let rotation = Quaternion::from_axis_angle(camera.right(), Rad(y_pan));
        camera.dir = rotation.rotate_vector(camera.dir);
        camera.up = rotation.rotate_vector(camera.up);

        [
            x_movement_norm,
            y_movement_norm,
            z_movement_norm,
            x_pan_norm,
            y_pan_norm,
        ].iter().any(|norm| *norm != 0.0)
    }
}
