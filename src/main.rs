#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use rico_32::engine::rico::RicoEngine;

fn main() {
    let engine = RicoEngine::new("main.r32".to_string());
    engine.start().expect("Couldn't start the RICO-32 Engine!");
}
