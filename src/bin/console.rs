use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::io::{Read, Seek, SeekFrom};
use rico_32::{
    engine::{
        game::GameEngine,
        rico::{bind_keyboard, bind_mouse_input, bind_mouse_move, handle_engine_update},
    },
    scripting::cartridge::{make_cart, Cartridge}
};

fn load_embedded_cart() -> Option<Cartridge> {
    let mut exe = std::fs::File::open(std::env::current_exe().ok()?).ok()?;
    let len = exe.metadata().ok()?.len();
    exe.seek(SeekFrom::Start(len - 16)).ok()?;
    let mut magic = [0u8; 4];
    exe.read_exact(&mut magic).ok()?;
    if &magic != b"R32X" {
        return None;
    }
    let mut ver = [0u8; 4];
    let mut size = [0u8; 8];
    exe.read_exact(&mut ver).ok()?;
    exe.read_exact(&mut size).ok()?;
    let cart_size = u64::from_le_bytes(size);
    exe.seek(SeekFrom::Start(len - 16 - cart_size)).ok()?;
    let mut cart = vec![0; cart_size as usize];
    exe.read_exact(&mut cart).ok()?;
    let cart = make_cart(&cart).expect("Could not find cart in exe");
    Some(cart)
}


pub const SCREEN_SIZE: usize = 128;
pub const SCALE: usize = 4;
pub const WINDOW_WIDTH: usize = SCREEN_SIZE * SCALE;
pub const WINDOW_HEIGHT: usize = SCREEN_SIZE * SCALE;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("RICO-32")
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
        .build(&event_loop)
        .expect("Could not create RICO-32 window!");

    let cart = load_embedded_cart().expect("Could not load cart");

    let mut eng = GameEngine::new(cart);

    let surface_texture = SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);
    let mut pixels = Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture)
        .expect("Could not start pixels");

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
