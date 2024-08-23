use pollster::block_on;
use std::{borrow::Cow, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{
    CommandEncoder, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration,
    TextureFormat, TextureView,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

#[macro_use]
mod smart_include;

mod bloom;
mod camera;
mod downsampling;
mod indices;
mod otheruniforms;
mod podbool;
mod scene;
mod settings;
mod texture;
mod time_replacement;
mod uniforms;
mod uniformscontroller;
mod upsampling;
mod vertex;
mod vertices;

mod state;
use state::State;

#[derive(Default)]
struct App<'a> {
    // we wrap this because the window and surface should be created after the first resume
    // (as in the docs for ApplicationHander::resumed)
    // so we start off with this as none
    // and we can't impl ApplicationHandler for Option<AppState> because of the orphan rules
    app_state: Option<State<'a>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes();
        let window = event_loop
            .create_window(window_attrs)
            .expect("Couldn't create window.");

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::Element;
            use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");

            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));
        }

        self.app_state = Some(block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(app_state) = self.app_state.as_mut() {
            let _ = app_state.process_event(&event);
        }
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit()
            }
            _ => {}
        };
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }
    let _ = event_loop.run_app(&mut app);
}
