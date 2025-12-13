use std::time::Instant;

use macro_procs::ScreenEngine;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::{keyboard::Keyboard, mouse::MousePress},
    render::{
        colors::Colors,
        pixels::{circle, clear, draw, print_scr_mid, rect_fill},
    },
    scripting::lua::LogTypes, time::sync,
};

const TERMINAL_FRAME_RATE: i32 = 30;

enum Commands {
    Load(String),
    Save(String),
    Export(String)
}

#[derive(ScreenEngine)]
pub struct TerminalEngine {
    pixels: PixelsType,
    last_time: Instant,
    logs: Vec<String>,
    input: String,
    pub keyboard: Keyboard,
}

impl Default for TerminalEngine {
    fn default() -> Self {
        TerminalEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE * 2),
            last_time: Instant::now(),
            logs: Vec::new(),
            input: String::new(),
            keyboard: Keyboard::default()
        }
    }
}

impl TerminalEngine {
    pub fn add_log(&mut self, log: String) {
        for chunk in log.as_bytes().chunks(30) {
            let chunk_string = String::from_utf8(chunk.to_vec()).unwrap();
            self.logs.push(chunk_string);
        }
    }

    pub fn update(&mut self) {
        sync(&mut self.last_time, TERMINAL_FRAME_RATE);
        clear(&mut self.pixels, Colors::Gray);
        
        self.add_log("A".to_string());

        for (i, log) in self.logs[self.logs.len().saturating_sub(38)..].iter().enumerate() {
            print_scr_mid(&mut self.pixels, 1, SCREEN_SIZE as i32 * 2 - (6 * i as i32 + 20), Colors::Black, log.to_string());
        }
    }
}
