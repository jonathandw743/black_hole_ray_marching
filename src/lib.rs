use cfg_if::cfg_if;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}, monitor::{VideoMode, MonitorHandle},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[macro_use]
mod smart_include;

mod camera;
mod indices;
mod otheruniforms;
mod podbool;
mod settings;
mod texture;
mod time_replacement;
mod uniforms;
mod uniformscontroller;
mod vertex;
mod vertices;

mod state;
use state::State;





pub fn create_window() -> (Window, EventLoop<()>) {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::new(1000, 800));
    window.set_cursor_visible(true);
    // window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1000, 800));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    (window, event_loop)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    let (window, event_loop) = create_window();

    // the browser can't handle the width-4096 bg image
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let background_image = include_bytes!("space-browser.jpg");
        } else {
            let background_image = include_bytes!("space.jpg");
        }
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window, background_image).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == state.window().id() => {
                // only do all these window inputs if the state hasn't already done something with the input
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        // close the window is the cross or Esc is clicked or pressed
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                // state.render();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
                state.update();
                state.sleep();

                // if state.frame_number == 5 {
                //     *control_flow = ControlFlow::Exit;
                // }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            Event::LoopDestroyed => {
                // println!("hello ended");
                flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
            }
            _ => {}
        }
    });
}
