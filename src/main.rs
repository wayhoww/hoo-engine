use std::{
    cell::{Ref, RefCell},
    ops::Deref,
    rc::Rc,
};

use hoo_engine::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let hoo_engine = HooEngine::new_async(&window);
    let hoo_engine = futures::executor::block_on(hoo_engine);

    initialize_hoo_engine(hoo_engine.clone());

    hoo_engine.borrow().prepare();

    {
        let hoo_engine_ref = hoo_engine.borrow();
        let mut renderer_ref = hoo_engine_ref.get_renderer_mut();
        let init_future = renderer_ref.initialize_test_resources();
        futures::executor::block_on(init_future);
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            hoo_engine.borrow().next_frame();
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
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
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    run();
}
