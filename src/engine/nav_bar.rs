use macro_procs::ScreenEngine;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, NAV_BAR_HEIGHT, SCREEN_SIZE},
    input::mouse::MousePress,
    render::{
        colors::Colors,
        pixels::{clear, print_scr_mid, rect_fill},
    },
};

#[derive(ScreenEngine)]
pub struct NavEngine {
    pixels: PixelsType,
    options: Vec<String>,
    pub mouse: MousePress,
    pub selected: usize,
    pub just_switched: bool,
}

impl NavEngine {
    pub fn new(options: Vec<String>) -> Self {
        NavEngine {
            pixels: Colors::pixels(SCREEN_SIZE, NAV_BAR_HEIGHT),
            mouse: MousePress::default(),
            options,
            selected: 0,
            just_switched: false,
        }
    }

    pub fn update(&mut self) {
        clear(&mut self.pixels, Colors::Gray);
        let mut cur_x = 1;
        if self.just_switched {
            self.just_switched = false
        };

        for (i, option) in self.options.iter().enumerate() {
            if self.mouse.just_pressed
                && self.mouse.x != -1
                && self.mouse.x >= cur_x
                && self.mouse.x <= cur_x + option.len() as i32 * 4 + 5
            {
                self.just_switched = true;
                self.selected = i;
            }

            if i == self.selected {
                rect_fill(
                    &mut self.pixels,
                    cur_x,
                    0,
                    option.len() as i32 * 4 + 5,
                    8,
                    Colors::Black,
                );
                print_scr_mid(&mut self.pixels, cur_x + 2, 2, Colors::White, option.to_string());
            } else {
                print_scr_mid(&mut self.pixels, cur_x + 2, 2, Colors::Black, option.to_string());
            }
            cur_x += option.len() as i32 * 4 + 5;
        }
    }
}
