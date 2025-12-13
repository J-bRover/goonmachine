use rayon::prelude::*;
use std::{
    error::Error,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::sync::mpsc::channel;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use super::{game::GameEngine, nav_bar::NavEngine, sprite::SpriteEngine};
use crate::{
    engine::terminal::TerminalEngine, input::{keyboard::Keyboard, mouse::MousePress}, render::colors::Colors, scripting::cartridge::{get_cart, load_cartridge, update_scripts, PATH}
};

pub const SCREEN_SIZE: usize = 128;
pub const SCALE: usize = 4;
pub const NAV_BAR_HEIGHT: usize = 8;
pub const WINDOW_WIDTH: usize = SCREEN_SIZE * SCALE;
pub const WINDOW_HEIGHT: usize = (NAV_BAR_HEIGHT + SCREEN_SIZE * 2) * SCALE;

pub type PixelsType = Vec<Vec<Colors>>;

/* All screen engines must implement
 * Game for now, sprite in the future, maybe IDE
 */
pub trait ScreenEngine {
    fn pixels(&self) -> &PixelsType;

    fn reset_inputs(&mut self);
}

// Make sure to box new engines, just more efficient to just store a pointer
enum StateEngines {
    GameEngine(Box<GameEngine>),
    SpriteEngine(Box<SpriteEngine>),
    TerminalEngine(Box<TerminalEngine>)
}

/* Add bindings for diff engines in this struct in the vector
 * All engines are different screens on the console
 * Screen engines should auto derive the ScreenEngine trait
 */
pub struct RicoEngine {
    cart_path: Arc<Mutex<String>>,
    nav_engine: NavEngine,
    state_engines: Vec<StateEngines>,
}

fn watch_folder(path: Arc<Mutex<String>>) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();

    // Create a debouncer to avoid getting multiple events for the same change
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

    // Add path to be watched
    debouncer.watcher().watch(Path::new(PATH), RecursiveMode::Recursive)?;

    for result in rx {
        match result {
            Ok(events) => {
                let should_update = events
                    .iter()
                    .any(|e| e.kind == DebouncedEventKind::Any && e.path.extension().and_then(|s| s.to_str()) == Some("lua"));

                if should_update {
                    let p = path.lock().unwrap().to_string();
                    dbg!(&p);
                    update_scripts(&p)?;
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

impl RicoEngine {
    pub fn new(path: String) -> Self {
        let cart_path = Arc::from(Mutex::from(path));
        let path_clone = cart_path.clone();
        let mut eng = RicoEngine {
            nav_engine: NavEngine::new(vec!["Game".to_string(), "Sprite".to_string(), "Term".to_string()]),
            state_engines: Vec::new(),
            cart_path,
        };

        let p = eng.cart_path.lock().unwrap().to_string().clone();
        eng.load(p);

        std::thread::spawn(|| match watch_folder(path_clone) {
            Ok(_) => println!("Watcher exited normally"),
            Err(e) => println!("Watcher error: {:?}", e),
        });

        eng
    }

    pub fn load(&mut self, path: String) {
        let p = self.cart_path.lock().unwrap().to_string().clone();
        let cart = load_cartridge(&p).expect("Could not load/create cartridge");
        let sprite_eng = SpriteEngine::new(self.cart_path.clone(), cart.sprite_sheet.clone());
        let game_eng = GameEngine::new(cart);
        if self.state_engines.is_empty() {
            let term_eng = TerminalEngine::default();
            self.state_engines = vec![
                StateEngines::GameEngine(Box::new(game_eng)),
                StateEngines::SpriteEngine(Box::new(sprite_eng)),
                StateEngines::TerminalEngine(Box::new(term_eng)),
            ];
        } else {
            self.state_engines[0] = StateEngines::GameEngine(Box::new(game_eng));
            self.state_engines[1] = StateEngines::SpriteEngine(Box::new(sprite_eng));
        }
        *self.cart_path.lock().unwrap() = path;
    }

    //Base boot function, needs to take in whole self cause borrowing bs
    pub fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("RICO-32")
            .with_resizable(false)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .build(&event_loop)?;

        let surface_texture =
            SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);
        let mut pixels = Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture)?;

        // Event loop: Poll so we run as fast as possible and continuously request redraws
        event_loop.run(move |event, _, control_flow| {
            // Poll loop -> render as fast as possible
            *control_flow = ControlFlow::Poll;

            match event {
                Event::RedrawRequested(_) => {
                    //Pass in buffer and redraw all based pixels every frame
                    let buffer = pixels.frame_mut();
                    self.update(buffer);

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                    }
                }

                //Redraw every frame
                Event::MainEventsCleared => {
                    window.request_redraw();
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            //Use match for finding which engine we're using rn
                            match self.state_engines[self.nav_engine.selected] {
                                StateEngines::GameEngine(ref mut eng) => {
                                    let mut lua_api = eng.lua_api.borrow_mut();
                                    bind_keyboard(&mut lua_api.keyboard, input.state, keycode);
                                }
                                StateEngines::SpriteEngine(ref mut eng) => {
                                    bind_keyboard(&mut eng.keyboard, input.state, keycode);
                                },
                                StateEngines::TerminalEngine(ref mut eng) => {
                                    bind_keyboard(&mut eng.keyboard, input.state, keycode);
                                }
                            }

                            // exit on ESC
                            if keycode == winit::event::VirtualKeyCode::Escape {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        //If let cause we only need scroll wheels for sprite rn, switch to match
                        //later
                        if let StateEngines::SpriteEngine(ref mut eng) =
                            self.state_engines[self.nav_engine.selected]
                        {
                            let scroll_y = match delta {
                                MouseScrollDelta::LineDelta(_, y) => y,
                                MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                            };

                            eng.update_start_row(scroll_y);
                        }
                    }

                    WindowEvent::MouseInput { button, state, .. } => {
                        bind_mouse_input(&mut self.nav_engine.mouse, button, state);
                        match self.state_engines[self.nav_engine.selected] {
                            StateEngines::GameEngine(ref mut eng) => {
                                //Make these binding functions if we need more input
                                bind_mouse_input(
                                    &mut eng.lua_api.borrow_mut().mouse,
                                    button,
                                    state,
                                );
                                bind_mouse_input(&mut eng.console_engine.mouse, button, state);
                            }
                            StateEngines::SpriteEngine(ref mut eng) => {
                                bind_mouse_input(&mut eng.mouse, button, state);
                            },
                            StateEngines::TerminalEngine(_) => {}
                        };
                    }

                    //Cursor moving is complex cause we wanna set pos to -1 if not on the engine
                    WindowEvent::CursorMoved { position, .. } => {
                        let scale = window.scale_factor();
                        //Weird that its different than normal position but wtv
                        let logical = position.to_logical::<f32>(scale);

                        bind_mouse_move(
                            &mut self.nav_engine.mouse,
                            logical,
                            0,
                            0,
                            WINDOW_WIDTH,
                            NAV_BAR_HEIGHT * SCALE,
                        );
                        match self.state_engines[self.nav_engine.selected] {
                            StateEngines::GameEngine(ref mut eng) => {
                                bind_mouse_move(
                                    &mut eng.lua_api.borrow_mut().mouse,
                                    logical,
                                    0,
                                    NAV_BAR_HEIGHT * SCALE,
                                    WINDOW_WIDTH,
                                    WINDOW_WIDTH,
                                );
                                bind_mouse_move(
                                    &mut eng.console_engine.mouse,
                                    logical,
                                    0,
                                    NAV_BAR_HEIGHT * SCALE + WINDOW_WIDTH,
                                    WINDOW_WIDTH,
                                    WINDOW_WIDTH,
                                );
                            }
                            StateEngines::SpriteEngine(ref mut eng) => {
                                bind_mouse_move(
                                    &mut eng.mouse,
                                    logical,
                                    0,
                                    NAV_BAR_HEIGHT * SCALE,
                                    WINDOW_WIDTH,
                                    WINDOW_WIDTH * 2,
                                );
                            },
                            StateEngines::TerminalEngine(_) => {}
                        }
                    }

                    _ => {}
                },
                _ => {}
            }
        });
    }

    //Make sure to update engines here based on which screen it's on
    pub fn update(&mut self, buffer: &mut [u8]) {
        self.nav_engine.update();
        handle_engine_update(buffer, &mut self.nav_engine, 0, 0);

        match self.state_engines[self.nav_engine.selected] {
            StateEngines::GameEngine(ref mut eng) => {
                if self.nav_engine.just_switched {
                    eng.console_engine.last_time = Instant::now();
                }

                eng.update();

                handle_engine_update(
                    buffer,
                    &mut *eng.lua_api.borrow_mut(),
                    0,
                    NAV_BAR_HEIGHT * SCALE,
                );

                let console = &mut eng.console_engine;
                handle_engine_update(buffer, console, 0, WINDOW_WIDTH + (NAV_BAR_HEIGHT * SCALE));

                if console.restart {
                    let cart = get_cart(&self.cart_path.lock().unwrap().to_string())
                        .expect("Could not load/create cartridge");
                    let game_eng = GameEngine::new(cart);
                    **eng = game_eng;
                }
            }
            StateEngines::SpriteEngine(ref mut eng) => {
                eng.update();
                handle_engine_update(buffer, &mut **eng, 0, NAV_BAR_HEIGHT * SCALE);
            },
            StateEngines::TerminalEngine(ref mut eng) => {
                eng.update();
                handle_engine_update(buffer, &mut **eng, 0, NAV_BAR_HEIGHT * SCALE);
            }
        }
    }
}

//Make sure to position correctly with the start x and y
pub fn handle_engine_update(
    buffer: &mut [u8],
    eng: &mut dyn ScreenEngine,
    start_x: usize,
    start_y: usize,
) {
    //Uses screen engine implementations to actually render that specific engine
    let pixels = eng.pixels();
    copy_pixels_into_buffer(pixels, buffer, start_x, start_y);
    eng.reset_inputs();
}

/* IMPORTANT:
 * Currently parallalized with rayon but its lowkey useless
 * It spends half the time just switching mutex locks so we might wanna just single
 * thread this. Shouldn't change too much, we're pretty efficient alr.
 */
pub fn copy_pixels_into_buffer(
    pixels: &PixelsType,
    buffer: &mut [u8],
    start_x: usize,
    start_y: usize,
) {
    let height = pixels.len();
    let width = pixels[0].len();

    let mut buf_tmp = vec![0u8; width * height * SCALE * SCALE * 4];

    buf_tmp.par_chunks_mut(width * SCALE * 4).enumerate().for_each(|(out_y, row)| {
        let src_y = out_y / SCALE;

        for (x, pix) in pixels[src_y].iter().enumerate().take(width) {
            let (r, g, b, a) = pix.rgba();
            let base = x * SCALE * 4;
            for dx in 0..SCALE {
                let i = base + dx * 4;
                row[i..i + 4].copy_from_slice(&[r, g, b, a]);
            }
        }
    });

    for y in 0..height * SCALE {
        let dst_row = ((start_y + y) * WINDOW_WIDTH + start_x) * 4;
        let src_row = y * width * SCALE * 4;

        buffer[dst_row..dst_row + width * SCALE * 4]
            .copy_from_slice(&buf_tmp[src_row..src_row + width * SCALE * 4]);
    }
}

pub fn bind_keyboard(keyboard: &mut Keyboard, state: ElementState, keycode: VirtualKeyCode) {
    match state {
        ElementState::Pressed => {
            //This is weird but idk a better way to do it
            if !keyboard.keys_pressed.contains(&keycode) {
                keyboard.keys_just_pressed.insert(keycode);
            }
            keyboard.keys_pressed.insert(keycode);
        }
        ElementState::Released => {
            keyboard.keys_pressed.remove(&keycode);
        }
    }
}

//Im so sad this doesn't give me access to mouse position, it'd be sm easier to
//do the -1, -1 thing just here
pub fn bind_mouse_input(mouse: &mut MousePress, button: MouseButton, state: ElementState) {
    if button == MouseButton::Left {
        match state {
            ElementState::Pressed => {
                mouse.pressed = true;
                mouse.just_pressed = true;
            }
            ElementState::Released => {
                mouse.pressed = false;
                mouse.just_pressed = false;
            }
        }
    }
}

pub fn check_mouse_bounds(
    mouse: &mut MousePress,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
) -> bool {
    let cur_x = mouse.x as usize;
    let cur_y = mouse.y as usize;
    if cur_x < start_x || cur_x > start_x + width || cur_y < start_y || cur_y > start_y + height {
        mouse.pressed = false;
        mouse.just_pressed = false;
        mouse.x = -1;
        mouse.y = -1;
        return true;
    }

    false
}

pub fn bind_mouse_move(
    mouse: &mut MousePress,
    logical_position: LogicalPosition<f32>,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
) {
    mouse.x = logical_position.x as i32;
    mouse.y = logical_position.y as i32;

    if !check_mouse_bounds(mouse, start_x, start_y, width, height) {
        mouse.x -= start_x as i32;
        mouse.y -= start_y as i32;

        //Wanna switch to the size of the screen instead of the tiny screen pixels
        mouse.x /= SCALE as i32;
        mouse.y /= SCALE as i32;
    };
}
