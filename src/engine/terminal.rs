use std::time::Instant;

use macro_procs::ScreenEngine;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::{keyboard::{str_from_key, Keyboard}, mouse::MousePress},
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
    cursor: usize,
    frame_hash: i32,
    pub keyboard: Keyboard,
}

impl Default for TerminalEngine {
    fn default() -> Self {
        TerminalEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE * 2),
            last_time: Instant::now(),
            logs: Vec::new(),
            input: String::new(),
            cursor: 0,
            frame_hash: 0,
            keyboard: Keyboard::default()
        }
    }
}

impl TerminalEngine {
    pub fn add_log(&mut self, res: String) {
        for chunk in res.as_bytes().chunks(30) {
            let chunk_string = String::from_utf8(chunk.to_vec()).unwrap();
            self.logs.push(chunk_string);
        }
    }

    pub fn update(&mut self) {
        self.frame_hash += 1;
        self.frame_hash %= 20;
        sync(&mut self.last_time, TERMINAL_FRAME_RATE);
        clear(&mut self.pixels, Colors::Gray);

        if !self.keyboard.keys_just_pressed.is_empty() {
            for key in &self.keyboard.keys_just_pressed {
                match str_from_key(key) {
                    "" | "Up" | "Down" => continue,
                    "Right" => {
                        self.cursor = (self.cursor + 1).min(self.input.len())
                    },
                    "Left" => {
                        if self.cursor != 0 {
                            self.cursor -= 1;
                        }
                    },
                    "Back" => {
                        if self.cursor != 0 {
                            self.input.remove(self.cursor - 1);
                            self.cursor -= 1;
                        }
                    },
                    "Enter" => {
                        for chunk in self.input.as_bytes().chunks(30) {
                            let chunk_string = String::from_utf8(chunk.to_vec()).unwrap();
                            self.logs.push(chunk_string);
                        }
                        self.input.clear();
                        self.cursor = 0;
                    },
                    other => {
                        self.input.insert_str(self.cursor, other);
                        self.cursor += 1;
                    }
                };
            }
        }

        for (i, log) in self.logs[self.logs.len().saturating_sub(38)..].iter().enumerate() {
            print_scr_mid(&mut self.pixels, 1, 6 * i as i32 + 20, Colors::Black, log.to_string());
        }
        if self.frame_hash > 10 {
            rect_fill(&mut self.pixels, 1 + 4 * self.cursor as i32, SCREEN_SIZE as i32 * 2 - 6, 1, 5, Colors::White);
        }
        print_scr_mid(&mut self.pixels, 1, SCREEN_SIZE as i32 * 2 - 6, Colors::Black, self.input.to_string());
    }
}
