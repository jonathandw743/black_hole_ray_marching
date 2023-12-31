use std::time::Duration;

use winit::{dpi::PhysicalPosition, event::*};

// use cgmath::prelude::*;

// use cgmath::{Quaternion, Rad};

use glam::{mat4, vec2, vec4, Mat4, Quat, Vec2, Vec3};

pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn right(&self) -> Vec3 {
        return self.up.cross(self.dir);
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // camera's position vector
        let p = self.pos;
        // up vector (normalized)
        let u = self.up.normalize();
        // forward vector (normalized)
        let f = self.dir.normalize();
        // right vector (normalized)
        let r = self.right();

        #[rustfmt::skip]
        let view = mat4(
            vec4(r.x.clone(), u.x.clone(), -f.x.clone(), 0.0),
            vec4(r.y.clone(), u.y.clone(), -f.y.clone(), 0.0),
            vec4(r.z.clone(), u.z.clone(), -f.z.clone(), 0.0),
            vec4(-p.dot(r), -p.dot(u), p.dot(f), 1.0),
        );
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

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

    prev_cursor_position: Option<PhysicalPosition<f64>>,
    curr_cursor_position: Option<PhysicalPosition<f64>>,

    mouse_is_pressed: bool,
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

            prev_cursor_position: None,
            curr_cursor_position: None,

            mouse_is_pressed: false,
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            &WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                true
            }
            &WindowEvent::CursorMoved { position, .. } => {
                self.prev_cursor_position = self.curr_cursor_position;
                self.curr_cursor_position = Some(position);
                true
            }
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

    pub fn cursor_movement(&self) -> Vec2 {
        match (self.prev_cursor_position, self.curr_cursor_position) {
            (Some(prev), Some(curr)) => {
                let curr_vec2: Vec2 = vec2(curr.x as f32, curr.y as f32);
                let prev_vec2: Vec2 = vec2(prev.x as f32, prev.y as f32);
                curr_vec2 - prev_vec2
            }
            _ => Vec2::ZERO,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: Duration, do_pan: bool) -> bool {
        let dt = delta_time.as_secs_f32();

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
        let _z_movement = dt * self.speed * z_movement_norm;

        let y_movement_norm = match (self.is_down_pressed, self.is_up_pressed) {
            (false, false) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (true, true) => 0.0,
        };
        let y_movement = dt * self.speed * y_movement_norm;

        camera.pos += x_movement * camera.right();
        // camera.pos += z_movement * camera.dir;
        camera.pos += y_movement * camera.up;

        camera.pos = camera.pos * (-dt * z_movement_norm).exp();

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

        let rotation = Quat::from_axis_angle(Vec3::Y, x_pan);
        camera.dir = rotation.mul_vec3(camera.dir);
        camera.up = rotation.mul_vec3(camera.up);

        let rotation = Quat::from_axis_angle(camera.right(), y_pan);
        camera.dir = rotation.mul_vec3(camera.dir);
        camera.up = rotation.mul_vec3(camera.up);

        let pan = dt * self.pan_speed * self.cursor_movement();

        if self.mouse_is_pressed && do_pan {
            let rotation = Quat::from_axis_angle(Vec3::Y, pan.x);
            camera.dir = rotation.mul_vec3(camera.dir);
            camera.up = rotation.mul_vec3(camera.up);

            let rotation = Quat::from_axis_angle(camera.right(), pan.y);
            camera.dir = rotation.mul_vec3(camera.dir);
            camera.up = rotation.mul_vec3(camera.up);
        }
        [
            x_movement_norm,
            y_movement_norm,
            z_movement_norm,
            x_pan_norm,
            y_pan_norm,
        ]
        .iter()
        .any(|norm| *norm != 0.0)
    }
}
