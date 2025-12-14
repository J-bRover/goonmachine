#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::io::{Read, Seek, SeekFrom};

use rico_32::{engine::{rico::RicoEngine, standalone}, scripting::cartridge::{make_cart, Cartridge}};

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


fn main() {
    match load_embedded_cart() {
        Some(cart) => standalone::start(cart),
        None => {
            let engine = RicoEngine::new("main.r32".to_string());
            engine.start().expect("Couldn't start the RICO-32 Engine!");
        }
    } 
}
