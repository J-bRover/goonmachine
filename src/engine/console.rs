use std::time::Instant;

use macros::ScreenEngine;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::mouse::MousePress,
    render::{
        colors::Colors,
        pixels::{circle, clear, draw, print_scr_mid, rect_fill},
    },
    scripting::lua::LogTypes,
};

#[derive(ScreenEngine)]
pub struct ConsoleEngine {
    pixels: PixelsType,
    pub last_time: Instant,
    pub halted: bool,
    pub mouse: MousePress,
    pub restart: bool,
}

const HALT_BUTTON: (i32, i32, i32, i32) = (50, 2, 13, 9);
const RESTART_BUTTON: (i32, i32, i32, i32) = (66, 2, 13, 9);
const G: Colors = Colors::Gray;
const B: Colors = Colors::Blank;
const RESTART_IMAGE: [[Colors; 7]; 7] = [
    [B, B, G, G, G, G, B],
    [B, G, G, B, G, G, G],
    [G, G, B, B, B, G, B],
    [G, B, B, B, B, B, B],
    [G, G, B, B, B, B, G],
    [B, G, G, B, B, G, G],
    [B, B, G, G, G, G, B],
];

impl Default for ConsoleEngine {
    fn default() -> Self {
        ConsoleEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE),
            last_time: Instant::now(),
            halted: false,
            mouse: MousePress::default(),
            restart: false,
        }
    }
}

impl ConsoleEngine {
    //Just UI stuff
    fn draw_game_control(&mut self) {
        rect_fill(
            &mut self.pixels,
            HALT_BUTTON.0,
            HALT_BUTTON.1,
            HALT_BUTTON.2,
            HALT_BUTTON.3,
            Colors::Silver,
        );

        if self.halted {
            circle(&mut self.pixels, 56, 6, 2, Colors::Green);
        } else {
            rect_fill(&mut self.pixels, 54, 4, 5, 5, Colors::Red);
        }

        rect_fill(
            &mut self.pixels,
            RESTART_BUTTON.0,
            RESTART_BUTTON.1,
            RESTART_BUTTON.2,
            RESTART_BUTTON.3,
            Colors::Silver,
        );

        draw(&mut self.pixels, RESTART_BUTTON.0 + 3, RESTART_BUTTON.1 + 1, &RESTART_IMAGE);
    }

    //I should really make a helper button class or just a few funcs huh
    fn assess_game_control(&mut self) {
        if self.mouse.just_pressed
            && self.mouse.x >= HALT_BUTTON.0
            && self.mouse.x <= HALT_BUTTON.0 + HALT_BUTTON.2
            && self.mouse.y >= HALT_BUTTON.1
            && self.mouse.y <= HALT_BUTTON.1 + HALT_BUTTON.3
        {
            let curr = self.halted;
            if curr {
                self.last_time = Instant::now();
            }
            self.halted = !curr;
        }
        if self.restart {
            self.restart = false
        };
        if self.mouse.just_pressed
            && self.mouse.x >= RESTART_BUTTON.0
            && self.mouse.x <= RESTART_BUTTON.0 + RESTART_BUTTON.2
            && self.mouse.y >= RESTART_BUTTON.1
            && self.mouse.y <= RESTART_BUTTON.1 + RESTART_BUTTON.3
        {
            self.restart = true;
        }
    }

    pub fn update(&mut self, logs: &[LogTypes]) {
        clear(&mut self.pixels, Colors::Gray);
        self.draw_game_control();
        self.assess_game_control();
        //19 definitely should be dynamically calculated but wtv for now
        for (i, log) in logs[logs.len().saturating_sub(19)..].iter().enumerate() {
            //Maybe make constants for colors in future
            let col = match log {
                LogTypes::Err(_) => Colors::Maroon,
                LogTypes::Ok(_) => Colors::Black,
            };
            print_scr_mid(&mut self.pixels, 1, 6 * i as i32 + 2 + 3 * 6, col, log.to_string());
        }
    }
}
