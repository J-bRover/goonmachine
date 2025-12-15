use rayon::prelude::*;
use std::{
    error::Error,
    fs,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::sync::mpsc::channel;

use pixels::Pixels;
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use super::{game::GameEngine, nav_bar::NavEngine, sprite::SpriteEngine};
use crate::{
    engine::{
        ide::IDEEngine,
        terminal::{Commands, TerminalEngine},
    },
    input::{keyboard::Keyboard, mouse::MousePress},
    render::colors::Colors,
    scripting::{
        cartridge::{
            decode, get_cart, load_cartridge, update_scripts, write_cart, Cartridge, PATH,
        },
        lua::LogTypes,
    },
};

#[cfg(target_os = "linux")]
fn add_ext(mut file: String) -> String {
    if !file.ends_with(".linux") {
        file.push_str(".linux");
    }
    file
}

#[cfg(target_os = "windows")]
fn add_ext(mut file: String) -> String {
    if !file.ends_with(".exe") {
        file.push_str(".exe");
    }
    file
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn add_ext(mut file: String) -> String {
    file
}

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
    Game(Box<GameEngine>),
    Sprite(Box<SpriteEngine>),
    Terminal(Box<TerminalEngine>),
    Ide(Box<IDEEngine>),
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
                let should_update = events.iter().any(|e| {
                    e.kind == DebouncedEventKind::Any
                        && e.path.extension().and_then(|s| s.to_str()) == Some("lua")
                });

                if should_update {
                    let p = path.lock().expect("Couldn't resolve path").to_string();
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
            nav_engine: NavEngine::new(vec![
                "Game".to_string(),
                "Sprite".to_string(),
                "IDE".to_string(),
                "Term".to_string(),
            ]),
            state_engines: Vec::new(),
            cart_path,
        };

        let p = eng.cart_path.lock().unwrap().to_string().clone();
        let cart = match load_cartridge(&p) {
            Ok(cart) => cart,
            Err(_) => {
                let cart = Cartridge::default();
                let _ = write_cart(&p, &cart);
                for (file, content) in &cart.scripts {
                    let f_path = PATH.to_owned() + file;
                    if let Some(parent) = Path::new(&f_path).parent() {
                        fs::create_dir_all(parent).expect("Could not initialize r32/ directory");
                    }

                    fs::write(f_path, content).expect("Could not initialize r32/ directory");
                }
                cart
            }
        };
        eng.load(cart, p);

        std::thread::spawn(|| match watch_folder(path_clone) {
            Ok(_) => println!("Watcher exited normally"),
            Err(e) => println!("Watcher error: {:?}", e),
        });

        eng
    }

    pub fn load(&mut self, cart: Cartridge, path: String) {
        let sprite_eng = SpriteEngine::new(path.clone(), cart.sprite_sheet.clone());
        let ide_eng = IDEEngine::default();
        let game_eng = GameEngine::new(cart);
        if self.state_engines.is_empty() {
            let term_eng = TerminalEngine::default();
            self.state_engines = vec![
                StateEngines::Game(Box::new(game_eng)),
                StateEngines::Sprite(Box::new(sprite_eng)),
                StateEngines::Ide(Box::new(ide_eng)),
                StateEngines::Terminal(Box::new(term_eng)),
            ];
        } else {
            self.state_engines[0] = StateEngines::Game(Box::new(game_eng));
            self.state_engines[1] = StateEngines::Sprite(Box::new(sprite_eng));
            self.state_engines[2] = StateEngines::Ide(Box::new(ide_eng));
        }
        *self.cart_path.lock().unwrap() = path;
    }

    //Base boot function, needs to take in whole self cause borrowing bs
    pub fn start(
        mut self,
        event_loop: EventLoop<()>,
        window: Window,
        mut pixels: Pixels,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

                    WindowEvent::ReceivedCharacter(c) => {
                        if !c.is_control() {
                            match self.state_engines[self.nav_engine.selected] {
                                StateEngines::Game(_) => {}
                                StateEngines::Sprite(_) => {}
                                StateEngines::Terminal(ref mut eng) => {
                                    eng.handle_char_input(c);
                                }
                                StateEngines::Ide(ref mut eng) => {
                                    eng.handle_string_input(c.to_string());
                                }
                            }
                        }
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            //Use match for finding which engine we're using rn
                            match self.state_engines[self.nav_engine.selected] {
                                StateEngines::Game(ref mut eng) => {
                                    let mut lua_api = eng.lua_api.borrow_mut();
                                    bind_keyboard(&mut lua_api.keyboard, input.state, keycode);
                                }
                                StateEngines::Sprite(ref mut eng) => {
                                    bind_keyboard(&mut eng.keyboard, input.state, keycode);
                                }
                                StateEngines::Terminal(ref mut eng) => {
                                    bind_keyboard(&mut eng.keyboard, input.state, keycode);
                                }
                                StateEngines::Ide(ref mut eng) => {
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
                        if let StateEngines::Sprite(ref mut eng) =
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
                            StateEngines::Game(ref mut eng) => {
                                //Make these binding functions if we need more input
                                bind_mouse_input(
                                    &mut eng.lua_api.borrow_mut().mouse,
                                    button,
                                    state,
                                );
                                bind_mouse_input(&mut eng.console_engine.mouse, button, state);
                            }
                            StateEngines::Sprite(ref mut eng) => {
                                bind_mouse_input(&mut eng.mouse, button, state);
                            }
                            StateEngines::Terminal(_) => {}
                            StateEngines::Ide(_) => {}
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
                            StateEngines::Game(ref mut eng) => {
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
                            StateEngines::Sprite(ref mut eng) => {
                                bind_mouse_move(
                                    &mut eng.mouse,
                                    logical,
                                    0,
                                    NAV_BAR_HEIGHT * SCALE,
                                    WINDOW_WIDTH,
                                    WINDOW_WIDTH * 2,
                                );
                            }
                            StateEngines::Terminal(_) => {}
                            StateEngines::Ide(_) => {}
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

        let engine = &mut self.state_engines[self.nav_engine.selected];

        let mut to_be_loaded: Option<String> = None;
        match engine {
            StateEngines::Game(eng) => {
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
            StateEngines::Sprite(eng) => {
                eng.update();
                handle_engine_update(buffer, &mut **eng, 0, NAV_BAR_HEIGHT * SCALE);
            }
            StateEngines::Ide(eng) => {
                eng.update();
                handle_engine_update(buffer, &mut **eng, 0, NAV_BAR_HEIGHT * SCALE);
            }
            StateEngines::Terminal(eng) => {
                eng.update();

                for command in eng.commands.clone() {
                    match command {
                        Commands::Help(cmd) => {
                            let cmd = cmd.as_deref();
                            match cmd {
                                Some("load") => {
                                    eng.add_log(LogTypes::Ok("Loads the .r32 or .r32.txt (base64 version) cartridge into memory for editing.".to_string()));
                                    eng.add_log(LogTypes::Ok("Usage: load <filename>".to_string()));
                                }
                                Some("save") => {
                                    eng.add_log(LogTypes::Ok("Saves the currently loaded cartridge into a .r32 or .r32.txt (base64 version) file. Converts type automatically.".to_string()));
                                    eng.add_log(LogTypes::Ok("Usage: save <filename>".to_string()));
                                }
                                Some("export") => {
                                    eng.add_log(LogTypes::Ok("Automatically exports a standalone executable of a game of the current cartridge loaded. Will automatically export to the current operating systems format.".to_string()));
                                    eng.add_log(LogTypes::Ok(
                                        "There is currently only support for windows and linux."
                                            .to_string(),
                                    ));
                                    eng.add_log(LogTypes::Ok(
                                        "Usage: export <filename>".to_string(),
                                    ));
                                }
                                None => {
                                    eng.add_log(LogTypes::Ok(
                                        "load: loads cartridge into console memory".to_string(),
                                    ));
                                    eng.add_log(LogTypes::Ok(
                                        "save: saves current cartridge to any file".to_string(),
                                    ));
                                    eng.add_log(LogTypes::Ok(
                                        "export: exports cartridge to a standalone executable"
                                            .to_string(),
                                    ));
                                    eng.add_log(LogTypes::Ok(
                                        "see help <cmd> for more information.".to_string(),
                                    ));
                                }
                                Some(c) => eng.add_log(LogTypes::Err(
                                    format!("{c} is not a valid command").to_string(),
                                )),
                            }
                        }
                        Commands::Load(file) => {
                            to_be_loaded = Some(file);
                        }
                        Commands::Save(file) => match get_cart(&self.cart_path.lock().unwrap()) {
                            Ok(cart) => match write_cart(&file, &cart) {
                                Ok(_) => eng.add_log(LogTypes::Ok(
                                    format!("Successfully saved cartridge to {file}").to_string(),
                                )),
                                Err(err) => eng.add_log(LogTypes::Err(err.to_string())),
                            },
                            Err(err) => eng.add_log(LogTypes::Err(err.to_string())),
                        },
                        Commands::Export(file) => {
                            let file = add_ext(file);
                            let f_clone = file.clone();
                            let result = (|| -> Result<(), Box<dyn Error>> {
                                let exe = fs::read(std::env::current_exe()?)?;
                                let path = self.cart_path.lock().unwrap().clone();
                                let mut cart = fs::read(&path)?;
                                if path.ends_with(".r32.txt") {
                                    cart = decode(&cart)
                                };
                                let mut out = fs::File::create(f_clone)?;
                                out.write_all(&exe)?;
                                out.write_all(&cart)?;

                                out.write_all(b"R32X")?;
                                out.write_all(&(1u32).to_le_bytes())?;
                                out.write_all(&(cart.len() as u64).to_le_bytes())?;
                                Ok(())
                            })();
                            match result {
                                Err(err) => eng.add_log(LogTypes::Err(err.to_string())),
                                Ok(_) => eng.add_log(LogTypes::Ok(
                                    format!("Successfully exported to {file}").to_string(),
                                )),
                            }
                        }
                    }
                }

                handle_engine_update(buffer, &mut **eng, 0, NAV_BAR_HEIGHT * SCALE);
            }
        }

        if let Some(file) = to_be_loaded {
            match load_cartridge(&file) {
                Ok(cart) => {
                    self.load(cart, file.clone());
                    if let StateEngines::Terminal(ref mut eng) = self.state_engines[3] {
                        eng.add_log(LogTypes::Ok(
                            format!("Successfully loaded cartridge from {file}").to_string(),
                        ));
                    }
                }
                Err(err) => {
                    if let StateEngines::Terminal(ref mut eng) = self.state_engines[3] {
                        eng.add_log(LogTypes::Err(err.to_string()));
                    }
                }
            }
        };
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
