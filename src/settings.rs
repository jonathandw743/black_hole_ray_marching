use winit::{
    event::*,
};

pub fn number_from_virtual_key_code(virtual_key_code: &VirtualKeyCode) -> Option<usize> {
    match virtual_key_code {
        VirtualKeyCode::Key0 => Some(0),
        VirtualKeyCode::Key1 => Some(1),
        VirtualKeyCode::Key2 => Some(2),
        VirtualKeyCode::Key3 => Some(3),
        VirtualKeyCode::Key4 => Some(4),
        VirtualKeyCode::Key5 => Some(5),
        VirtualKeyCode::Key6 => Some(6),
        VirtualKeyCode::Key7 => Some(7),
        VirtualKeyCode::Key8 => Some(8),
        VirtualKeyCode::Key9 => Some(9),
        _ => None,
    }
}

pub struct Settings {
    // pub anti_aliasing_number: f32,
    pub max_frame_rate: Option<f32>,
    // pub optical_density: f32,
}

impl Settings {
    pub fn new() -> Self {
        let settings = Self {
            // anti_aliasing_number: 0.0,
            max_frame_rate: Some(80.0),
            // optical_density: 1.2,
        };
        // settings.print_anti_aliasing_number();
        settings.print_max_frame_rate();
        // settings.print_optical_density();
        settings
    }
    // pub fn print_anti_aliasing_number(&self) {
    //     println!("Anti Aliasing Number:\n{}", self.anti_aliasing_number);
    // }
    pub fn print_max_frame_rate(&self) {
        match self.max_frame_rate {
            Some(max_frame_rate) => {
                println!("Max Frame Rate:\n{}", max_frame_rate);
            }
            None => {
                println!("Frame Rate Unlimited")
            }
        }
    }
    // pub fn print_optical_density(&self) {
    //     println!("Optical Density:\n{}", self.optical_density);
    // }
    // pub fn set_anti_aliasing_number(&mut self, new_anti_aliasing_number: f32) {
    //     self.anti_aliasing_number = new_anti_aliasing_number;
    //     self.print_anti_aliasing_number();
    // }
    pub fn set_max_frame_rate(&mut self, new_max_frame_rate: Option<f32>) {
        self.max_frame_rate = new_max_frame_rate;
        self.print_max_frame_rate();
    }
    // pub fn set_optical_density(&mut self, new_optical_density: f32) {
    //     self.optical_density = new_optical_density;
    //     self.print_optical_density();
    // }
}

pub struct SettingsController {
    // pub anti_aliasing_modifier_pressed: bool,
    pub max_frame_rate_modifier_pressed: bool,
    // pub optical_density_modifier_pressed: bool,

    pub number_just_pressed: Option<u8>,
}

impl SettingsController {
    pub fn new() -> Self {
        SettingsController {
            // anti_aliasing_modifier_pressed: false,
            max_frame_rate_modifier_pressed: false,
            // optical_density_modifier_pressed: false,

            number_just_pressed: None,
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
                // if the event is a number key press, set the number just pressed to that number
                if !is_pressed {
                    if let Some(number) = number_from_virtual_key_code(virtual_keycode)
                    {
                        self.number_just_pressed = Some(number as u8);
                        return true;
                    }
                }
                match virtual_keycode {
                    // VirtualKeyCode::A => {
                    //     self.anti_aliasing_modifier_pressed = is_pressed;
                    //     true
                    // }
                    VirtualKeyCode::F => {
                        self.max_frame_rate_modifier_pressed = is_pressed;
                        true
                    }
                    // VirtualKeyCode::O => {
                    //     self.optical_density_modifier_pressed = is_pressed;
                    //     true
                    // }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_settings(&mut self, settings: &mut Settings) {
        if let Some(number) = self.number_just_pressed {
            // if self.anti_aliasing_modifier_pressed {
            //     settings.set_anti_aliasing_number(number as f32 * 0.5);
            // }
            if self.max_frame_rate_modifier_pressed {
                settings.set_max_frame_rate(match number {
                    0 => None,
                    _ => Some(number as f32 * 20.0),
                });
            }
            // if self.optical_density_modifier_pressed {
            //     settings.set_optical_density(number as f32 * 0.1 + 1.0);
            // }
            self.number_just_pressed = None;
        }
    }
}
