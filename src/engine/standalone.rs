use crate::{
    engine::{
        game::GameEngine,
        rico::{
            bind_keyboard, bind_mouse_input, bind_mouse_move, handle_engine_update, SCALE,
            SCREEN_SIZE,
        },
    },
    scripting::cartridge::Cartridge,
};
use pixels::Pixels;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub const WINDOW_WIDTH: usize = SCREEN_SIZE * SCALE;
pub const WINDOW_HEIGHT: usize = SCREEN_SIZE * SCALE;

pub fn start(cart: Cartridge, event_loop: EventLoop<()>, window: Window, mut pixels: Pixels) {
    let mut eng = GameEngine::new(cart);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let buffer = pixels.frame_mut();
                eng.update();
                handle_engine_update(buffer, &mut *eng.lua_api.borrow_mut(), 0, 0);

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        let mut lua_api = eng.lua_api.borrow_mut();
                        bind_keyboard(&mut lua_api.keyboard, input.state, keycode);

                        if keycode == winit::event::VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }

                WindowEvent::MouseInput { button, state, .. } => {
                    bind_mouse_input(&mut eng.lua_api.borrow_mut().mouse, button, state);
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let scale = window.scale_factor();
                    let logical = position.to_logical::<f32>(scale);
                    bind_mouse_move(
                        &mut eng.lua_api.borrow_mut().mouse,
                        logical,
                        0,
                        0,
                        WINDOW_WIDTH,
                        WINDOW_WIDTH,
                    );
                }

                _ => {}
            },
            _ => {}
        }
    })
}
