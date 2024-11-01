use state::State;
use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{
    CommandEncoder, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration,
    TextureFormat, TextureView,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    error::EventLoopError,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
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
// mod podbool;
mod scene;
mod settings;
mod texture;
mod time_replacement;
mod uniforms;
// mod uniformscontroller;
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
mod gui;
mod black_holes_profile;

enum UserEvent {
    ApplicationCreated(State<'static>),
}

struct ApplicationWindow {
    app: Option<State<'static>>,
    window: Option<Arc<Window>>,
    close_requested: bool,
    event_proxy: EventLoopProxy<UserEvent>,
}

impl ApplicationHandler<UserEvent> for ApplicationWindow {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.is_some() {
            return;
        }

        let window_attributes =
            Window::default_attributes().with_inner_size(LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap()); // needed for resize closure on web
        self.window = Some(window.clone());

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;
            console_log::init().expect("could not initialize logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            // On wasm, append the canvas to the document body
            let window = window.clone();
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-target")?;
                    let canvas =
                        web_sys::Element::from(window.canvas().expect("couldn't retrieve canvas"));
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
            // let canvas = window.canvas().expect("couldn't retrieve canvas");
            // let web_window = web_sys::window().expect("couldn't retrieve website window");
            // let body = web_window
            //     .document()
            //     .and_then(|doc| doc.body())
            //     .expect("couldn't retrieve document body");
            // body.append_child(&web_sys::Element::from(canvas))
            //     .ok()
            //     .expect("couldn't append canvas to body");
            // let _ = window.request_inner_size(winit::dpi::LogicalSize::new(
            //     body.client_width(),
            //     body.client_height(),
            // ));
            // let resize_closure =
            //     wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            //         let _ = window.request_inner_size(winit::dpi::LogicalSize::new(
            //             body.client_width(),
            //             body.client_height(),
            //         ));
            //     }) as Box<dyn FnMut(_)>);
            // web_window
            //     .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
            //     .unwrap();
            // resize_closure.forget();
        }

        let event_proxy = self.event_proxy.clone();
        let app_closure = async move {
            let app = State::new(window)
                .await
                .expect("creation of application failed");
            event_proxy
                .send_event(UserEvent::ApplicationCreated(app))
                .map_err(|_| "sending created application failed")
                .unwrap();
        };

        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(app_closure);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(app_closure);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ApplicationCreated(application) => {
                self.app = Some(application);
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(app), Some(window)) = (&mut self.app, &self.window) else {
            return;
        };
        app.process_event(&event);

        match event {
            WindowEvent::Resized(size) => app.resize(size),
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(KeyCode::Escape),
            //             state: ElementState::Pressed,
            //             ..
            //         },
            //     ..
            // } => {
            //     self.close_requested = true;
            // }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F11),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(_) = window.fullscreen() {
                    window.set_fullscreen(None);
                } else {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
            WindowEvent::RedrawRequested => {
                app.update();
                if let Err(e) = app.render() {
                    if e == wgpu::SurfaceError::Outdated {
                        let size = window.inner_size();
                        app.resize(size);
                    } else {
                        panic!("{}", e);
                    }
                }
                app.sleep();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
            return;
        }
        let (Some(app), Some(window)) = (&mut self.app, &self.window) else {
            return;
        };

        window.request_redraw()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = ApplicationWindow {
        window: None,
        app: None,
        close_requested: false,
        event_proxy: event_loop.create_proxy(),
    };
    event_loop.run_app(&mut app).unwrap();

    // Ok(())
}
