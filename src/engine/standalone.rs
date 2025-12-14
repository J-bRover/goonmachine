use pixels::{Pixels, SurfaceTexture};
use crate::{
    engine::{
        game::GameEngine,
        rico::{bind_keyboard, bind_mouse_input, bind_mouse_move, handle_engine_update, ICON_BYTES, SCALE, SCREEN_SIZE},
    },
    scripting::cartridge::Cartridge,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};

pub const WINDOW_WIDTH: usize = SCREEN_SIZE * SCALE;
pub const WINDOW_HEIGHT: usize = SCREEN_SIZE * SCALE;

pub fn start(cart: Cartridge) {
    let event_loop = EventLoop::new();
    let icon_img = image::load_from_memory(ICON_BYTES).expect("Failed to load icon").to_rgba8();
    let (width, height) = icon_img.dimensions();
    let icon = Icon::from_rgba(icon_img.into_raw(), width, height).expect("Could not load icon");
    let window = WindowBuilder::new()
        .with_title("RICO-32")
        .with_window_icon(Some(icon))
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
        .build(&event_loop)
        .expect("Could not create RICO-32 window!");

    let surface_texture = SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);
    let mut pixels = Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture)
        .expect("Could not start pixels");

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

