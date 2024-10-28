use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::state::State;


#[derive(Default)]
pub struct App<'a> {
    // we wrap this because the window and surface should be created after the first resume
    // (as in the docs for ApplicationHander::resumed)
    // so we start off with this as none
    // and we can't impl ApplicationHandler for Option<AppState> because of the orphan rules
    app_state: Option<State<'a>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes().with_inner_size(PhysicalSize {
            width: 1280,
            height: 720,
        });
        let window = event_loop
            .create_window(window_attrs)
            .expect("Couldn't create window.");

        self.app_state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if !self
            .app_state
            .as_ref()
            .map_or(false, |app_state| app_state.window.id() == id)
        {
            return;
        }
        if let Some(app_state) = self.app_state.as_mut() {
            let _ = app_state.process_event(&event);
        }
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            _ => {}
        };
    }
}