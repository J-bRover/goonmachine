use std::{error::Error, time::Instant};

use macro_procs::ScreenEngine;
use winit::event::VirtualKeyCode;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::keyboard::Keyboard,
    render::{
        colors::Colors,
        pixels::{clear, print_scr_mid, rect_fill},
    },
    scripting::lua::LogTypes,
    time::sync,
};

const TERMINAL_FRAME_RATE: i32 = 30;

#[derive(Clone)]
pub enum Commands {
    Help(Option<String>),
    Load(String),
    Save(String),
    Export(String),
}

#[derive(ScreenEngine)]
pub struct TerminalEngine {
    pixels: PixelsType,
    last_time: Instant,
    logs: Vec<LogTypes>,
    input: String,
    cursor: usize,
    frame_hash: i32,
    pub keyboard: Keyboard,
    pub commands: Vec<Commands>,
}

impl Default for TerminalEngine {
    fn default() -> Self {
        TerminalEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE * 2),
            last_time: Instant::now(),
            commands: Vec::new(),
            logs: Vec::new(),
            input: String::new(),
            cursor: 0,
            frame_hash: 0,
            keyboard: Keyboard::default(),
        }
    }
}

impl TerminalEngine {
    pub fn add_log(&mut self, log: LogTypes) {
        let msg = log.to_string();

        //Useful for wrapping lines I dont wanna implement scrolling in logs :/
        for chunk in msg.as_bytes().chunks(30) {
            let chunk_string = String::from_utf8(chunk.to_vec()).unwrap();
            let part: LogTypes = match log {
                LogTypes::Ok(_) => LogTypes::Ok(chunk_string),
                LogTypes::Err(_) => LogTypes::Err(chunk_string),
            };
            self.logs.push(part);
        }
    }

    fn parse_command(&mut self, cmd: &str) -> Result<Commands, Box<dyn Error>> {
        let tokens: Vec<&str> = cmd.split_ascii_whitespace().collect();

        match tokens.first().ok_or("Not a valid command")?.to_lowercase().as_str() {
            "help" => match tokens.get(1) {
                Some(cmd) => Ok(Commands::Help(Some(cmd.to_string().to_lowercase()))),
                None => Ok(Commands::Help(None)),
            },
            "load" => {
                let file = tokens.get(1).ok_or("Must pass in a file")?;
                if !file.ends_with(".r32") && !file.ends_with(".r32.txt") {
                    return Err("Must pass in a .r32 or .r32.txt cartridge to load".into());
                }
                Ok(Commands::Load(file.to_string()))
            }
            "save" => {
                let file = tokens.get(1).ok_or("Must pass in a file")?;
                if !file.ends_with(".r32") && !file.ends_with(".r32.txt") {
                    return Err("Must pass in a .r32 or .r32.txt cartridge to save to".into());
                }
                Ok(Commands::Save(file.to_string()))
            }
            "export" => {
                let file = tokens.get(1).ok_or("Must pass in file name to export to")?;
                Ok(Commands::Export(file.to_string()))
            }
            _ => Err("Not a valid command".into()),
        }
    }

    pub fn handle_char_input(&mut self, s: char) {
        let letter = s.to_string();
        self.input.insert_str(self.cursor, &letter);
        self.cursor += 1;
    }

    pub fn update(&mut self) {
        self.frame_hash += 1;
        self.frame_hash %= 20;
        sync(&mut self.last_time, TERMINAL_FRAME_RATE);
        clear(&mut self.pixels, Colors::Gray);

        self.commands.clear();

        for key in self.keyboard.keys_just_pressed.clone() {
            match key {
                VirtualKeyCode::Up | VirtualKeyCode::Down => continue,
                VirtualKeyCode::Right => self.cursor = (self.cursor + 1).min(self.input.len()),
                VirtualKeyCode::Left => {
                    if self.cursor != 0 {
                        self.cursor -= 1;
                    }
                }
                VirtualKeyCode::Back => {
                    if self.cursor != 0 {
                        self.input.remove(self.cursor - 1);
                        self.cursor -= 1;
                    }
                }
                VirtualKeyCode::Return => {
                    let cmd = self.input.clone();
                    self.add_log(LogTypes::Ok(">".to_string() + &cmd));

                    match self.parse_command(&cmd) {
                        Ok(cmd) => self.commands.push(cmd),
                        Err(err) => self.add_log(LogTypes::Err(err.to_string())),
                    }

                    self.input.clear();
                    self.cursor = 0;
                }
                _ => {}
            };
        }

        for (i, log) in self.logs[self.logs.len().saturating_sub(38)..].iter().enumerate() {
            let col = match log {
                LogTypes::Err(_) => Colors::Maroon,
                LogTypes::Ok(_) => Colors::Black,
            };
            print_scr_mid(&mut self.pixels, 1, 6 * i as i32 + 20, col, log.to_string());
        }
        if self.frame_hash > 10 {
            rect_fill(
                &mut self.pixels,
                1 + 4 * self.cursor as i32,
                SCREEN_SIZE as i32 * 2 - 6,
                1,
                5,
                Colors::White,
            );
        }
        print_scr_mid(
            &mut self.pixels,
            1,
            SCREEN_SIZE as i32 * 2 - 6,
            Colors::Black,
            self.input.to_string(),
        );
    }
}
