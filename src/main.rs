#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::io::{Read, Seek, SeekFrom};

use pixels::{Pixels, SurfaceTexture};
use rico_32::{
    engine::{
        rico::{RicoEngine, WINDOW_HEIGHT, WINDOW_WIDTH},
        standalone::{self, WINDOW_HEIGHT as WH_ST, WINDOW_WIDTH as WW_ST},
    },
    scripting::cartridge::{make_cart, Cartridge},
};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Icon, WindowBuilder},
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

pub const ICON_BYTES: &[u8] = include_bytes!("../assets/logo.png");

fn main() {
    let event_loop = EventLoop::new();
    let icon_img = image::load_from_memory(ICON_BYTES).expect("Failed to load icon").to_rgba8();
    let (width, height) = icon_img.dimensions();
    let icon = Icon::from_rgba(icon_img.into_raw(), width, height).expect("Could not load icon");
    let window = WindowBuilder::new()
        .with_title("RICO-32")
        .with_window_icon(Some(icon))
        .with_resizable(false);

    match load_embedded_cart() {
        Some(cart) => {
            let window = window
                .with_inner_size(LogicalSize::new(WW_ST as f64, WH_ST as f64))
                .build(&event_loop)
                .expect("Could not start the RICO-32 engine!");

            let surface_texture = SurfaceTexture::new(WW_ST as u32, WH_ST as u32, &window);
            let pixels = Pixels::new(WW_ST as u32, WH_ST as u32, surface_texture)
                .expect("Could not initialize pixels");
            standalone::start(cart, event_loop, window, pixels);
        }
        None => {
            let window = window
                .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
                .build(&event_loop)
                .expect("Could not start the RICO-32 engine!");

            let surface_texture =
                SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);
            let pixels = Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture)
                .expect("Could not initialize pixels!");

            let engine = RicoEngine::new("main.r32".to_string());
            engine.start(event_loop, window, pixels).expect("Couldn't start the RICO-32 Engine!");
        }
    }
}
