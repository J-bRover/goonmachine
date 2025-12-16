use std::time::Instant;

use macros::ScreenEngine;
use winit::event::VirtualKeyCode;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::{keyboard::Keyboard, mouse::MousePress},
    render::{
        colors::{Colors, ALL_COLORS},
        pixels::{
            clear, draw, image_from_tool, image_from_util, print_scr_mid, rect, rect_fill, set_pix,
        },
    },
    scripting::cartridge::update_sprites,
    time::sync,
};

#[derive(Copy, Clone, PartialEq)]
pub enum Tools {
    Pencil,
    Fill,
    Eraser,
    Select,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Utils {
    FlipHor,
    FlipVert,
    Clear,
    Save,
}

//I SWEAR THIS IS BETTER THAN ALL THE MAGIC NUMBERS
pub const BUTTON_WIDTH: i32 = 12;
pub const SPRITE_SIZE: usize = 32;
const DRAW_Y: i32 = 52;
const FRAME_RATE: i32 = 60;
const PIXEL_SIZE: i32 = 3;
const CANVAS_X: i32 = 16;
const COLORS_PER_ROW: i32 = 8;
const COLOR_PALETTE_Y: i32 = 10;
const TOOLS_Y: i32 = 154;
const UTILS_X: i32 = 64;
const SAVE_X: i32 = 112;
const SPRITESHEET_Y: i32 = 174;
const SPRITESHEET_COLS: i32 = 6;
const SPRITESHEET_ROWS: i32 = 4;
const SPRITE_PREVIEW_SIZE: i32 = 16;
const ADD_SPRITE_BUTTON_Y: i32 = 242;
const ADD_SPRITE_BUTTON_SIZE: i32 = 9;
const UNDO_REDO_FRAME_DELAY: i32 = 25;
const UNDO_REDO_CONTINUOUS_FRAME_DIVISOR: i32 = 2;
const FRAME_HASH_MODULO: i32 = 7;
const SPRITES_TO_ADD: usize = 6;

const DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

type MoveInfoType = Option<((i32, i32), (i32, i32, i32, i32))>;

#[derive(ScreenEngine)]
pub struct SpriteEngine {
    pixels: PixelsType,
    selected_color: Colors,
    pub mouse: MousePress,
    pub sprite_sheet: Vec<PixelsType>,
    pub tool: Tools,
    pub keyboard: Keyboard,

    cart_path: String,

    selection: Option<(i32, i32, i32, i32)>,
    selection_start_pos: Option<(i32, i32)>,
    moving_selection_content: Option<PixelsType>,
    move_start_info: MoveInfoType,

    copied_content: Option<PixelsType>,

    new_changes: Vec<(usize, usize, Colors)>,
    undo_stack: Vec<Vec<(usize, usize, Colors)>>,
    redo_stack: Vec<Vec<(usize, usize, Colors)>>,
    last_frame_ur: bool,
    continuous_ur_frames: i32,

    last_time: Instant,
    idx: usize,
    upto_date: bool,
    start_row: i32,
    frame_hash: i32,
}

impl SpriteEngine {
    pub fn new(cart_path: String, sprite_sheet: Vec<PixelsType>) -> Self {
        SpriteEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE * 2),
            mouse: MousePress::default(),
            selected_color: Colors::Black,
            sprite_sheet,
            cart_path,
            tool: Tools::Pencil,
            selection: None,
            selection_start_pos: None,
            moving_selection_content: None,
            move_start_info: None,
            keyboard: Keyboard::default(),
            copied_content: None,
            new_changes: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_frame_ur: false,
            continuous_ur_frames: 0,
            last_time: Instant::now(),
            idx: 0,
            upto_date: true,
            start_row: 0,
            frame_hash: 0,
        }
    }

    fn set_pix(&mut self, y: usize, x: usize, col: Colors) {
        if self.sprite_sheet[self.idx][y][x] == col {
            return;
        }
        self.new_changes.push((y, x, self.sprite_sheet[self.idx][y][x]));
        self.upto_date = false;
        self.sprite_sheet[self.idx][y][x] = col;
    }

    fn stamp_selection(&mut self) {
        if let (Some(mut content), Some((x1, y1, _, _))) =
            (self.moving_selection_content.take(), self.selection)
        {
            let h = content.len();
            let w = content[0].len();

            for (r, row) in content.iter_mut().enumerate().take(h) {
                for (c, col) in row.iter_mut().enumerate().take(w) {
                    let target_y = y1 as isize + r as isize;
                    let target_x = x1 as isize + c as isize;
                    if target_y >= 0
                        && target_y < SPRITE_SIZE as isize
                        && target_x >= 0
                        && target_x < SPRITE_SIZE as isize
                    {
                        self.set_pix(target_y as usize, target_x as usize, *col);
                    }
                }
            }
        }
        self.moving_selection_content = None;
    }

    fn handle_click(&mut self, y: usize, x: usize) {
        match self.tool {
            Tools::Pencil => {
                self.set_pix(y, x, self.selected_color);
            }
            Tools::Fill => {
                let col = self.sprite_sheet[self.idx][y][x];
                let mut q: Vec<(i32, i32)> = vec![(y as i32, x as i32)];
                let mut visited: [[bool; SPRITE_SIZE]; SPRITE_SIZE] =
                    [[false; SPRITE_SIZE]; SPRITE_SIZE];
                while let Some(t) = q.pop() {
                    if visited[t.0 as usize][t.1 as usize] {
                        continue;
                    }
                    visited[t.0 as usize][t.1 as usize] = true;
                    self.set_pix(t.0 as usize, t.1 as usize, self.selected_color);

                    for dir in DIRS {
                        let ny = t.0 + dir.0;
                        let nx = t.1 + dir.1;
                        if ny >= 0
                            && ny < SPRITE_SIZE as i32
                            && nx >= 0
                            && nx < SPRITE_SIZE as i32
                            && col == self.sprite_sheet[self.idx][ny as usize][nx as usize]
                        {
                            q.push((ny, nx));
                        }
                    }
                }
            }
            Tools::Eraser => {
                self.set_pix(y, x, Colors::Blank);
            }
            Tools::Select => {}
        };
    }

    fn draw_canvas(&mut self) {
        for y in 0..SPRITE_SIZE as i32 {
            for x in 0..SPRITE_SIZE as i32 {
                let mut col = self.sprite_sheet[self.idx][y as usize][x as usize];
                if col == Colors::Blank {
                    col = if (y + x) % 2 == 0 { Colors::Silver } else { Colors::White };
                }
                rect_fill(
                    &mut self.pixels,
                    CANVAS_X + x * PIXEL_SIZE,
                    DRAW_Y + y * PIXEL_SIZE,
                    PIXEL_SIZE,
                    PIXEL_SIZE,
                    col,
                );
            }
        }

        let on_canvas = self.mouse.x >= CANVAS_X
            && self.mouse.x < CANVAS_X + (SPRITE_SIZE as i32 * PIXEL_SIZE)
            && self.mouse.y >= DRAW_Y
            && self.mouse.y < DRAW_Y + (SPRITE_SIZE as i32 * PIXEL_SIZE);
        let grid_x = ((self.mouse.x - CANVAS_X) / PIXEL_SIZE).max(0).min(SPRITE_SIZE as i32 - 1);
        let grid_y = ((self.mouse.y - DRAW_Y) / PIXEL_SIZE).max(0).min(SPRITE_SIZE as i32 - 1);

        if self.tool == Tools::Select {
            if self.mouse.just_pressed && on_canvas {
                if let Some((x1, y1, x2, y2)) = self.selection {
                    if grid_x >= x1 && grid_x <= x2 && grid_y >= y1 && grid_y <= y2 {
                        self.move_start_info = Some(((grid_x, grid_y), self.selection.unwrap()));

                        if self.moving_selection_content.is_none() {
                            let w = (x2 - x1 + 1) as usize;
                            let h = (y2 - y1 + 1) as usize;
                            let mut content = Colors::pixels(w, h);
                            for (r, row) in content.iter_mut().enumerate().take(h) {
                                for (c, col) in row.iter_mut().enumerate().take(w) {
                                    *col = self.sprite_sheet[self.idx][y1 as usize + r]
                                        [x1 as usize + c];
                                    self.set_pix(y1 as usize + r, x1 as usize + c, Colors::Blank);
                                }
                            }
                            self.moving_selection_content = Some(content);
                        }
                    } else {
                        self.stamp_selection();
                        self.selection = None;
                        self.selection_start_pos = Some((grid_x, grid_y));
                    }
                } else {
                    self.stamp_selection();
                    self.selection = None;
                    self.selection_start_pos = Some((grid_x, grid_y));
                }
            }

            if self.mouse.pressed {
                if let Some((start_grid, start_rect)) = self.move_start_info {
                    let dx = grid_x - start_grid.0;
                    let dy = grid_y - start_grid.1;
                    self.selection = Some((
                        start_rect.0 + dx,
                        start_rect.1 + dy,
                        start_rect.2 + dx,
                        start_rect.3 + dy,
                    ));
                } else if let Some(start_pos) = self.selection_start_pos {
                    let x1 = start_pos.0.min(grid_x);
                    let y1 = start_pos.1.min(grid_y);
                    let x2 = start_pos.0.max(grid_x);
                    let y2 = start_pos.1.max(grid_y);
                    self.selection = Some((x1, y1, x2, y2));
                }
            }
            if !self.mouse.pressed {
                self.move_start_info = None;
                self.selection_start_pos = None;
            }

            if let (Some(content), Some((x1, y1, _, _))) =
                (self.moving_selection_content.as_ref(), self.selection)
            {
                let h = content.len();
                let w = content[0].len();
                for (r, row) in content.iter().enumerate().take(h) {
                    for (c, col) in row.iter().enumerate().take(w) {
                        let mut color = col;
                        let draw_x = CANVAS_X + (x1 + c as i32) * PIXEL_SIZE;
                        let draw_y = DRAW_Y + (y1 + r as i32) * PIXEL_SIZE;

                        if *color == Colors::Blank {
                            color = if (draw_y + draw_x) % 2 == 0 {
                                &Colors::Silver
                            } else {
                                &Colors::White
                            };
                        }
                        rect_fill(&mut self.pixels, draw_x, draw_y, PIXEL_SIZE, PIXEL_SIZE, *color);
                    }
                }
            }

            if let Some((x1, y1, x2, y2)) = self.selection {
                let x = CANVAS_X + x1 * PIXEL_SIZE;
                let y = DRAW_Y + y1 * PIXEL_SIZE;
                let w = (x2 - x1 + 1) * PIXEL_SIZE;
                let h = (y2 - y1 + 1) * PIXEL_SIZE;
                rect(&mut self.pixels, x, y, w, h, Colors::White);
                rect(&mut self.pixels, x - 1, y - 1, w + 2, h + 2, Colors::Black);
            }
        } else {
            if self.mouse.pressed && on_canvas {
                self.handle_click(grid_y as usize, grid_x as usize);
            }
            self.stamp_selection();
            self.selection = None;
            self.selection_start_pos = None;
            self.move_start_info = None;
        }
    }

    fn tool_button(&mut self, x: i32, y: i32, tool: Tools) {
        draw(&mut self.pixels, x + 1, y + 1, &image_from_tool(tool));

        if self.mouse.x >= x
            && self.mouse.x < x + BUTTON_WIDTH
            && self.mouse.y >= y
            && self.mouse.y < y + BUTTON_WIDTH
        {
            rect(&mut self.pixels, x, y, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, Colors::Silver);
            if self.mouse.just_pressed {
                self.tool = tool;
            }
        }

        if self.tool == tool {
            rect(&mut self.pixels, x, y, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, Colors::White);
        }
    }

    fn color_button(&mut self, x: i32, y: i32, col: Colors) {
        rect_fill(&mut self.pixels, x + 1, y + 1, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, col);

        rect(&mut self.pixels, x, y, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, Colors::Gray);
        if self.selected_color == col {
            rect(&mut self.pixels, x, y, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, Colors::White);
        }

        if self.mouse.x != -1
            && self.mouse.x >= x
            && self.mouse.x < x + BUTTON_WIDTH
            && self.mouse.y >= y
            && self.mouse.y < y + BUTTON_WIDTH
        {
            rect_fill(&mut self.pixels, x, y, BUTTON_WIDTH, BUTTON_WIDTH, col);
            if self.mouse.just_pressed {
                self.selected_color = col;
            }
        }
    }

    fn handle_undo_redo(&mut self) {
        let mut t = 0;

        if self.keyboard.keys_pressed.contains(&VirtualKeyCode::LControl)
            && self.keyboard.keys_pressed.contains(&VirtualKeyCode::Z)
        {
            t = 1;
        } else if self.keyboard.keys_pressed.contains(&VirtualKeyCode::LControl)
            && self.keyboard.keys_pressed.contains(&VirtualKeyCode::R)
        {
            t = -1;
        }

        if t != 0 {
            if self.last_frame_ur {
                self.continuous_ur_frames += 1
            } else {
                self.continuous_ur_frames = 0
            };
            self.last_frame_ur = true;
            if (self.continuous_ur_frames < UNDO_REDO_FRAME_DELAY && self.continuous_ur_frames > 0)
                || (self.continuous_ur_frames % UNDO_REDO_CONTINUOUS_FRAME_DIVISOR != 0)
            {
                return;
            }

            let popped = if t == 1 { self.undo_stack.pop() } else { self.redo_stack.pop() };
            if let Some(changes) = popped {
                self.selection = None;
                self.selection_start_pos = None;
                self.move_start_info = None;
                let mut pushing: Vec<(usize, usize, Colors)> = Vec::new();
                for change in changes {
                    pushing.push((
                        change.0,
                        change.1,
                        self.sprite_sheet[self.idx][change.0][change.1],
                    ));
                    self.sprite_sheet[self.idx][change.0][change.1] = change.2;
                }
                if t == 1 {
                    self.redo_stack.push(pushing)
                } else {
                    self.undo_stack.push(pushing)
                };
            }
        } else {
            self.last_frame_ur = false;
        }
    }

    fn handle_copy_paste(&mut self) {
        if self.keyboard.keys_pressed.contains(&VirtualKeyCode::LControl)
            && self.keyboard.keys_just_pressed.contains(&VirtualKeyCode::C)
        {
            if self.moving_selection_content.is_some() {
                self.copied_content = self.moving_selection_content.clone();
            } else if let Some((x1, y1, x2, y2)) = self.selection {
                let w = (x2 - x1 + 1) as usize;
                let h = (y2 - y1 + 1) as usize;
                let mut content = Colors::pixels(w, h);
                for (r, row) in content.iter_mut().enumerate().take(h) {
                    for (c, col) in row.iter_mut().enumerate().take(w) {
                        *col = self.sprite_sheet[self.idx][y1 as usize + r][x1 as usize + c];
                    }
                }
                self.copied_content = Some(content);
            }
        }
        if self.keyboard.keys_pressed.contains(&VirtualKeyCode::LControl)
            && self.keyboard.keys_just_pressed.contains(&VirtualKeyCode::V)
        {
            if let Some(content) = self.copied_content.clone() {
                self.stamp_selection();
                let w = content[0].len() as i32;
                let h = content.len() as i32;
                self.selection = Some((0, 0, w - 1, h - 1));
                self.selection_start_pos = None;
                self.moving_selection_content = Some(content.clone());
                self.move_start_info = None;
            }
        }
    }

    fn util_button(&mut self, x: i32, y: i32, util: Utils) {
        draw(&mut self.pixels, x + 1, y + 1, &image_from_util(util));

        if self.mouse.x >= x
            && self.mouse.x < x + BUTTON_WIDTH
            && self.mouse.y >= y
            && self.mouse.y < y + BUTTON_WIDTH
        {
            rect(&mut self.pixels, x, y, BUTTON_WIDTH - 1, BUTTON_WIDTH - 1, Colors::Silver);
            if !self.mouse.just_pressed {
                return;
            }
            match util {
                Utils::FlipVert => {
                    if self.moving_selection_content.is_some() {
                        self.moving_selection_content.as_mut().unwrap().reverse();
                    } else if let Some((x1, y1, x2, y2)) = self.selection {
                        let w = (x2 - x1 + 1) as usize;
                        let h = (y2 - y1 + 1) as usize;
                        let cloned = self.sprite_sheet[self.idx].clone();
                        for r in 0..h {
                            for c in 0..w {
                                self.set_pix(
                                    y1 as usize + r,
                                    x1 as usize + c,
                                    cloned[y2 as usize - r][x1 as usize + c],
                                );
                            }
                        }
                    }
                }
                Utils::FlipHor => {
                    if self.moving_selection_content.is_some() {
                        let h = self.moving_selection_content.as_ref().unwrap().len();
                        for i in 0..h {
                            self.moving_selection_content.as_mut().unwrap()[i].reverse();
                        }
                    } else if let Some((x1, y1, x2, y2)) = self.selection {
                        let w = (x2 - x1 + 1) as usize;
                        let h = (y2 - y1 + 1) as usize;
                        let cloned = self.sprite_sheet[self.idx].clone();
                        for r in 0..h {
                            for c in 0..w {
                                self.set_pix(
                                    y1 as usize + r,
                                    x1 as usize + c,
                                    cloned[y1 as usize + r][x2 as usize - c],
                                );
                            }
                        }
                    }
                }
                Utils::Clear => {
                    if self.moving_selection_content.is_some() {
                        let h = self.moving_selection_content.as_ref().unwrap().len();
                        for i in 0..h {
                            self.moving_selection_content.as_mut().unwrap()[i].fill(Colors::Blank);
                        }
                    } else if let Some((x1, y1, x2, y2)) = self.selection {
                        let w = (x2 - x1 + 1) as usize;
                        let h = (y2 - y1 + 1) as usize;
                        for r in 0..h {
                            for c in 0..w {
                                self.set_pix(y1 as usize + r, x1 as usize + c, Colors::Blank);
                            }
                        }
                    } else {
                        for i in 0..SPRITE_SIZE {
                            for j in 0..SPRITE_SIZE {
                                self.set_pix(i, j, Colors::Blank);
                            }
                        }
                    }
                }
                Utils::Save => {
                    self.upto_date = true;
                    let p = self.cart_path.clone();
                    let _ = update_sprites(&p, &self.sprite_sheet);
                }
            }
        }
    }

    fn sprite_small(&mut self, idx: i32, true_idx: i32) {
        let y = SPRITESHEET_Y + (idx / SPRITESHEET_COLS) * SPRITE_PREVIEW_SIZE;
        let x = CANVAS_X + (idx % SPRITESHEET_COLS) * SPRITE_PREVIEW_SIZE;
        for i in 0..SPRITE_PREVIEW_SIZE {
            for j in 0..SPRITE_PREVIEW_SIZE {
                set_pix(
                    &mut self.pixels,
                    y + i,
                    x + j,
                    self.sprite_sheet[true_idx as usize][i as usize * 2][j as usize * 2],
                );
            }
        }

        rect(&mut self.pixels, x, y, SPRITE_PREVIEW_SIZE, SPRITE_PREVIEW_SIZE, Colors::Gray);

        if self.mouse.just_pressed
            && self.mouse.x >= x
            && self.mouse.x < x + SPRITE_PREVIEW_SIZE
            && self.mouse.y >= y
            && self.mouse.y < y + SPRITE_PREVIEW_SIZE
        {
            self.idx = true_idx as usize;
            self.selection = None;
            self.selection_start_pos = None;
            self.move_start_info = None;
        }
    }

    pub fn update_start_row(&mut self, delta: f32) {
        if self.frame_hash == 0
            && self.mouse.x >= CANVAS_X
            && self.mouse.x < CANVAS_X + SPRITE_PREVIEW_SIZE * SPRITESHEET_COLS
            && self.mouse.y >= SPRITESHEET_Y
            && self.mouse.y < SPRITESHEET_Y + SPRITE_PREVIEW_SIZE * SPRITESHEET_ROWS
        {
            if delta > 0.0 {
                self.start_row -= 1;
            } else if delta < 0.0 {
                self.start_row += 1;
            }
            self.start_row = self
                .start_row
                .max(0)
                .min(self.sprite_sheet.len() as i32 / SPRITESHEET_COLS - SPRITESHEET_ROWS);
        }
    }

    fn draw_sprite_sheet(&mut self) {
        let sprites_to_show = (SPRITESHEET_COLS * SPRITESHEET_ROWS) as usize;
        let start_idx = self.start_row * SPRITESHEET_COLS;
        for i in start_idx..start_idx + sprites_to_show as i32 {
            self.sprite_small(i - start_idx, i);
        }

        let start_idx_usize = start_idx as usize;
        if self.idx >= start_idx_usize && self.idx < start_idx_usize + sprites_to_show {
            let y = SPRITESHEET_Y
                + ((self.idx - start_idx_usize) as i32 / SPRITESHEET_COLS) * SPRITE_PREVIEW_SIZE;
            let x = CANVAS_X
                + ((self.idx - start_idx_usize) as i32 % SPRITESHEET_COLS) * SPRITE_PREVIEW_SIZE;
            rect(&mut self.pixels, x, y, SPRITE_PREVIEW_SIZE, SPRITE_PREVIEW_SIZE, Colors::White);
        }

        let scroll_height = (SPRITE_PREVIEW_SIZE * SPRITESHEET_ROWS) as f32;
        let scroll_start = scroll_height * (start_idx as f32 / self.sprite_sheet.len() as f32);
        let scroll_end = scroll_height
            * ((start_idx_usize + sprites_to_show) as f32 / self.sprite_sheet.len() as f32);
        rect_fill(
            &mut self.pixels,
            CANVAS_X + SPRITE_PREVIEW_SIZE * SPRITESHEET_COLS,
            SPRITESHEET_Y + scroll_start as i32,
            PIXEL_SIZE,
            (scroll_end - scroll_start) as i32,
            Colors::White,
        );

        let add_x = CANVAS_X + SPRITE_PREVIEW_SIZE * (SPRITESHEET_COLS - 1) + 8;
        rect_fill(
            &mut self.pixels,
            add_x,
            ADD_SPRITE_BUTTON_Y,
            ADD_SPRITE_BUTTON_SIZE,
            ADD_SPRITE_BUTTON_SIZE,
            Colors::Gray,
        );

        if self.mouse.x >= add_x
            && self.mouse.x < add_x + ADD_SPRITE_BUTTON_SIZE
            && self.mouse.y > ADD_SPRITE_BUTTON_Y
            && self.mouse.y < ADD_SPRITE_BUTTON_Y + ADD_SPRITE_BUTTON_SIZE
        {
            rect_fill(
                &mut self.pixels,
                add_x,
                ADD_SPRITE_BUTTON_Y,
                ADD_SPRITE_BUTTON_SIZE,
                ADD_SPRITE_BUTTON_SIZE,
                Colors::Silver,
            );
            if self.mouse.just_pressed {
                let adding = vec![Colors::pixels(SPRITE_SIZE, SPRITE_SIZE); SPRITES_TO_ADD];
                self.sprite_sheet.extend(adding);
                let p = self.cart_path.clone();
                let _ = update_sprites(&p, &self.sprite_sheet);
            }
        }

        for y in ADD_SPRITE_BUTTON_Y + 2..ADD_SPRITE_BUTTON_Y + 7 {
            set_pix(&mut self.pixels, y, add_x + 4, Colors::Black);
        }
        for x in 2..7 {
            set_pix(&mut self.pixels, ADD_SPRITE_BUTTON_Y + 4, add_x + x, Colors::Black);
        }
    }

    pub fn update(&mut self) {
        self.frame_hash = (self.frame_hash + 1) % FRAME_HASH_MODULO;
        sync(&mut self.last_time, FRAME_RATE);
        clear(&mut self.pixels, Colors::Black);
        //clear(&mut self.pixels, COLORS::GRAY);

        // rect(&mut self.pixels, 14, 8, COLOR_BUTTON_WIDTH*8 + 3, COLOR_BUTTON_WIDTH * 2 + 3, COLORS::WHITE);
        for (i, col) in ALL_COLORS.iter().enumerate().skip(1) {
            let idx = i as i32 - 1;
            self.color_button(
                CANVAS_X + (idx % COLORS_PER_ROW) * BUTTON_WIDTH,
                COLOR_PALETTE_Y + BUTTON_WIDTH * (idx >= COLORS_PER_ROW) as i32,
                *col,
            );
        }

        for (i, tool) in
            [Tools::Pencil, Tools::Eraser, Tools::Fill, Tools::Select].iter().enumerate()
        {
            let idx = i as i32;
            self.tool_button(4 + (idx % COLORS_PER_ROW) * BUTTON_WIDTH, TOOLS_Y, *tool);
        }

        for (i, util) in [Utils::FlipHor, Utils::FlipVert, Utils::Clear].iter().enumerate() {
            let idx = i as i32;
            self.util_button(UTILS_X + (idx % COLORS_PER_ROW) * BUTTON_WIDTH, TOOLS_Y, *util);
        }
        self.util_button(SAVE_X, TOOLS_Y, Utils::Save);

        let mut sprite_text = "Editing sprite ".to_owned() + &self.idx.to_string();
        if !self.upto_date {
            sprite_text += "*"
        };
        print_scr_mid(&mut self.pixels, CANVAS_X, DRAW_Y - 8, Colors::Gray, sprite_text);
        self.draw_canvas();
        self.handle_copy_paste();

        self.handle_undo_redo();
        self.draw_sprite_sheet();

        if !self.new_changes.is_empty() {
            self.undo_stack.push(self.new_changes.clone());
            self.redo_stack.clear();
            self.new_changes = Vec::new();
        }
    }
}
