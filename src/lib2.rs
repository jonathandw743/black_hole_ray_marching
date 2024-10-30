use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};
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
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

#[macro_use]
mod smart_include;

//mod bloom;
mod camera;
// mod downsampling;
mod indices;
mod otheruniforms;
mod podbool;
mod scene;
mod settings;
mod texture;
mod time_replacement;
mod uniforms;
mod uniformscontroller;
// mod upsampling;
mod vertex;
mod vertices;
// mod gaussian_blur;
mod kawase_downsampling;
mod kawase_upsampling;
// mod kawase_mixing_upsampling;

//mod blur;
mod copy;
mod remix;
mod state;

#[cfg(not(target_arch = "wasm32"))]
mod app;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = app::App::default();
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    let _ = event_loop.run_app(&mut app);
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");
    let event_loop = EventLoop::new().unwrap();
    let window = event_loop
        .create_window(Window::default_attributes())
        .unwrap();
    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-example")?;
            let canvas = web_sys::Element::from(window.canvas().unwrap());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
    wasm_bindgen_futures::spawn_local(async {
        let mut app_state = State::new(window).await;
        event_loop.run(move |event, active_event_loop| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app_state.window.id() => {
                app_state.process_event(event);
                match event {
                    WindowEvent::CloseRequested => {
                        println!("The close button was pressed; stopping");
                        active_event_loop.exit();
                    }
                    _ => {}
                };
            }
            _ => {}
        });
    });
}
