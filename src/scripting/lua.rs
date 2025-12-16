use crate::engine::rico::ScreenEngine;
use macros::ScreenEngine;
use mlua::prelude::LuaResult;
use std::rc::Rc;
use std::{cell::RefCell, fmt};

use mlua::UserData;

use crate::{
    engine::{
        game::BASE_FPS,
        rico::{PixelsType, SCREEN_SIZE},
    },
    input::{
        keyboard::{key_from_str, Keyboard},
        mouse::MousePress,
    },
    render::{
        colors::Colors,
        pixels::{
            circle, clear, draw, print_scr, print_scr_mid, print_scr_mini, rect, rect_fill, set_pix,
        },
    },
};

//Not using normal Result type so we can add warnings in the future
pub enum LogTypes {
    Ok(String),
    Err(String),
}

//Just so that I can .to_string() cause they're both gonna be displayed to the console in raw string
//anyway
impl fmt::Display for LogTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogTypes::Err(ref e) => e,
            LogTypes::Ok(ref m) => m,
        };
        write!(f, "{s}")
    }
}

#[derive(ScreenEngine)]
pub struct LuaAPI {
    pub mouse: MousePress,
    pub keyboard: Keyboard,
    pub frame_rate: i32,
    pub pixels: PixelsType,
    pub sprites: Vec<PixelsType>,
    pub logs: Vec<LogTypes>,
}

impl LuaAPI {
    pub fn new(sprite_sheet: Vec<PixelsType>) -> Self {
        LuaAPI {
            frame_rate: BASE_FPS,
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE),
            logs: Vec::new(),
            sprites: sprite_sheet,
            mouse: MousePress::default(),
            keyboard: Keyboard::default(),
        }
    }

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
}

#[derive(Clone)]
pub struct LuaAPIHandle(pub Rc<RefCell<LuaAPI>>);

//Just helper to auto return error if they give a bad color to any draw utils
//Used like everywhere modify carefully
fn col_from_str(col: String) -> LuaResult<Colors> {
    match col.parse::<Colors>() {
        Ok(c) => Ok(c),
        Err(_) => Err(mlua::Error::RuntimeError(format!("{} is not a valid color", col))),
    }
}

impl UserData for LuaAPIHandle {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("log", move |_, this, msg: String| {
            let msg = format!("[Log] {}", msg);
            this.0.borrow_mut().add_log(LogTypes::Ok(msg));
            Ok(())
        });

        methods.add_method("set_pix", move |_, this, (x, y, col): (i32, i32, String)| {
            let val = col_from_str(col)?;
            set_pix(&mut this.0.borrow_mut().pixels, y, x, val);

            Ok(())
        });

        methods.add_method("get_pix", |_, this, (x, y): (usize, usize)| {
            if x >= SCREEN_SIZE || y >= SCREEN_SIZE {
                return Err(mlua::Error::RuntimeError(format!(
                    "Pixel out of bounds: {}, {}",
                    x, y
                )));
            }
            Ok(this.0.borrow().pixels[y][x].to_string())
        });

        methods.add_method_mut(
            "print_scr",
            |_, this, (x, y, col, msg): (i32, i32, String, String)| {
                let c = col_from_str(col)?;
                print_scr(&mut this.0.borrow_mut().pixels, x, y, c, msg);
                Ok(())
            },
        );

        methods.add_method_mut(
            "print_scr_mini",
            |_, this, (x, y, col, msg): (i32, i32, String, String)| {
                let c = col_from_str(col)?;
                print_scr_mini(&mut this.0.borrow_mut().pixels, x, y, c, msg);
                Ok(())
            },
        );

        methods.add_method_mut(
            "print_scr_mid",
            |_, this, (x, y, col, msg): (i32, i32, String, String)| {
                let c = col_from_str(col)?;
                print_scr_mid(&mut this.0.borrow_mut().pixels, x, y, c, msg);
                Ok(())
            },
        );

        methods.add_method_mut("draw", |_, this, (x, y, idx): (i32, i32, usize)| {
            let mut eng = this.0.borrow_mut();

            if idx >= eng.sprites.len() {
                return Err(mlua::Error::RuntimeError("Index too large".to_string()));
            }

            let img = eng.sprites[idx].clone();
            let pixels = &mut eng.pixels;
            draw(pixels, x, y, &img);
            Ok(())
        });

        methods.add_method_mut(
            "rectfill",
            |_, this, (x, y, w, h, col): (i32, i32, i32, i32, String)| {
                let c = col_from_str(col)?;
                rect_fill(&mut this.0.borrow_mut().pixels, x, y, w, h, c);
                Ok(())
            },
        );

        methods.add_method_mut(
            "rect",
            |_, this, (x, y, w, h, col): (i32, i32, i32, i32, String)| {
                let c = col_from_str(col)?;
                rect(&mut this.0.borrow_mut().pixels, x, y, w, h, c);
                Ok(())
            },
        );

        methods.add_method_mut("circle", |_, this, (x, y, r, col): (i32, i32, i32, String)| {
            let c = col_from_str(col)?;
            circle(&mut this.0.borrow_mut().pixels, x, y, r, c);
            Ok(())
        });

        methods.add_method_mut("clear", |_, this, col: String| {
            let c = col_from_str(col)?;
            clear(&mut this.0.borrow_mut().pixels, c);
            Ok(())
        });

        methods.add_method_mut("set_frame_rate", |_, this, rate: i32| {
            this.0.borrow_mut().frame_rate = rate;
            Ok(())
        });

        methods.add_method("mouse", |_, this, ()| {
            let mut m = this.0.borrow().mouse;
            if this.0.borrow().mouse.x == -1 {
                m.pressed = false;
                m.just_pressed = false;
            }
            Ok(m)
        });

        methods.add_method("key_pressed", |_, this, key: String| {
            if let Some(kc) = key_from_str(&key) {
                return Ok(this.0.borrow().keyboard.keys_pressed.contains(&kc));
            } else {
                Err(mlua::Error::RuntimeError(format!("{} is not a valid key", key)))
            }
        });

        methods.add_method("key_just_pressed", |_, this, key: String| {
            if let Some(kc) = key_from_str(&key) {
                return Ok(this.0.borrow().keyboard.keys_just_pressed.contains(&kc));
            } else {
                Err(mlua::Error::RuntimeError(format!("{} is not a valid key", key)))
            }
        });
    }
}
