use std::{cmp::min, collections::HashSet, time::Instant};

use macro_procs::ScreenEngine;
use winit::event::VirtualKeyCode;

use crate::{
    engine::rico::{PixelsType, ScreenEngine, SCREEN_SIZE},
    input::keyboard::Keyboard,
    render::{
        colors::Colors,
        pixels::{clear, print_scr_mid, rect_fill},
    },
    time::sync,
};
use regex::Regex;

const IDE_FRAME_RATE: i32 = 30;
const TEXT_HEIGHT: i32 = 6;
const TEXT_WIDTH: i32 = 4;

#[derive(Clone)]
struct Change {
    deleted_text: String,
    deleted_start: (usize, usize),

    added_text: String,
    added_start: (usize, usize),

    cursor_before: (usize, usize),
    cursor_after: (usize, usize),
}

fn highlight_lua_line(line: &str) -> Vec<(String, Colors)> {
    let mut result = Vec::new();
    let mut remaining = line;

    let comment_re = Regex::new(r"^--.*").unwrap();
    let string_re = Regex::new(r#"^"[^"]*"|^'[^']*'"#).unwrap();
    let keyword_re = Regex::new(r"^(local|function|end|if|then|else|elseif|for|while|do|repeat|until|return|break|and|or|not|in)\b").unwrap();
    let number_re = Regex::new(r"^\d+\.?\d*").unwrap();
    let operator_re = Regex::new(r"^[+\-*/%=<>~]+").unwrap();
    let identifier_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap();

    while !remaining.is_empty() {
        if let Some(m) = comment_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Green));
        } else if let Some(m) = string_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Orange));
            remaining = &remaining[m.end()..];
        } else if let Some(m) = keyword_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Blue));
            remaining = &remaining[m.end()..];
        } else if let Some(m) = number_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Yellow));
            remaining = &remaining[m.end()..];
        } else if let Some(m) = operator_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Teal));
            remaining = &remaining[m.end()..];
        } else if let Some(m) = identifier_re.find(remaining) {
            result.push((m.as_str().to_string(), Colors::Silver));
            remaining = &remaining[m.end()..];
        } else {
            result.push((remaining[0..1].to_string(), Colors::Silver));
            remaining = &remaining[1..];
        }
    }

    result
}

#[derive(ScreenEngine)]
pub struct IDEEngine {
    pixels: PixelsType,
    last_time: Instant,
    pub keyboard: Keyboard,

    file: Vec<String>,
    cursor: (usize, usize),
    scroll_offset: (usize, usize),
    selection: Option<((usize, usize), (usize, usize))>,

    undo_stack: Vec<Change>,
    redo_stack: Vec<Change>,
    clipboard: String,
    frame_hash: i32,
}

impl Default for IDEEngine {
    fn default() -> Self {
        let initial_file = vec![
            "Welcome to the R32 IDE!".to_string(),
            "".to_string(),
            "Features:".to_string(),
            "- Arrow keys to move".to_string(),
            "- Shift + Arrows to select".to_string(),
            "- Ctrl+C to copy".to_string(),
            "- Ctrl+V to paste".to_string(),
            "- Ctrl+X to cut".to_string(),
            "- Ctrl+Z to undo".to_string(),
            "- Ctrl+R to redo".to_string(),
            "".to_string(),
            "Enjoy coding!".to_string(),
        ];

        IDEEngine {
            pixels: Colors::pixels(SCREEN_SIZE, SCREEN_SIZE * 2),
            last_time: Instant::now(),
            keyboard: Keyboard::default(),
            file: initial_file,
            cursor: (0, 0),
            scroll_offset: (0, 0),
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: String::new(),
            frame_hash: 0,
        }
    }
}

impl IDEEngine {
    pub fn update(&mut self) {
        self.frame_hash = (self.frame_hash + 1) % 20;
        sync(&mut self.last_time, IDE_FRAME_RATE);
        clear(&mut self.pixels, Colors::Black);

        self.scroll_to_cursor();
        self.render();
        self.handle_input();
    }

    fn render(&mut self) {
        let visible_rows = SCREEN_SIZE as i32 / TEXT_HEIGHT;
        let visible_cols = SCREEN_SIZE as i32 / TEXT_WIDTH;

        if let Some((start, end)) = self.get_sorted_selection() {
            for row in start.1..=end.1 {
                if row < self.scroll_offset.1 || row >= self.scroll_offset.1 + visible_rows as usize
                {
                    continue;
                }
                let y = (row - self.scroll_offset.1) as i32 * TEXT_HEIGHT + 1;

                let line = &self.file[row];
                let sel_start_col = if row == start.1 { start.0 } else { 0 };
                let sel_end_col = if row == end.1 { end.0 } else { line.len() };

                let x =
                    (sel_start_col.saturating_sub(self.scroll_offset.0)) as i32 * TEXT_WIDTH + 1;
                let w = (sel_end_col - sel_start_col) as i32 * TEXT_WIDTH;

                if x < SCREEN_SIZE as i32 && x + w > 0 {
                    rect_fill(&mut self.pixels, x, y, w, TEXT_HEIGHT, Colors::Teal);
                }
            }
        }

        for (row_idx, line) in self.file.iter().enumerate().skip(self.scroll_offset.1) {
            let y = (row_idx - self.scroll_offset.1) as i32 * TEXT_HEIGHT + 1;
            if y + TEXT_HEIGHT as i32 >= SCREEN_SIZE as i32 {
                break;
            }

            let start_col = self.scroll_offset.0;
            let end_col = min(line.len(), self.scroll_offset.0 + visible_cols as usize);
            if start_col < end_col {
                let line_to_render = &line[start_col..end_col];
                let tokens = highlight_lua_line(line_to_render);

                let mut x = 1;
                for (text, color) in tokens {
                    print_scr_mid(&mut self.pixels, x, y, color, text.clone());
                    x += text.len() as i32 * TEXT_WIDTH; 
                }
            }
        }

        if self.frame_hash > 10 {
            self.render_cursor();
        }

        rect_fill(&mut self.pixels, 0, SCREEN_SIZE as i32, SCREEN_SIZE as i32, 7, Colors::Black);
        let curr_line = self.cursor.1 + 1;
        let total_lines = self.file.len();
        print_scr_mid(
            &mut self.pixels,
            1,
            SCREEN_SIZE as i32 + 1,
            Colors::Silver,
            format!("LINE {curr_line}/{total_lines}").to_string(),
        );
    }

    fn render_cursor(&mut self) {
        let cursor_x = (self.cursor.0.saturating_sub(self.scroll_offset.0)) as i32 * TEXT_WIDTH + 1;
        let cursor_y =
            (self.cursor.1.saturating_sub(self.scroll_offset.1)) as i32 * TEXT_HEIGHT + 1;
        if cursor_y >= 0 && cursor_y < SCREEN_SIZE as i32 * 2 {
            rect_fill(&mut self.pixels, cursor_x, cursor_y, 1, TEXT_HEIGHT, Colors::Yellow);
        }
    }

    fn handle_input(&mut self) {
        let shift = self.keyboard.keys_pressed.contains(&VirtualKeyCode::LShift)
            || self.keyboard.keys_pressed.contains(&VirtualKeyCode::RShift);

        let keys_just = self.keyboard.keys_just_pressed.clone();
        let keys_pressed = self.keyboard.keys_pressed.clone();

        let old_cursor = self.cursor;

        for key in &keys_pressed{
            match key {
                VirtualKeyCode::Left
                | VirtualKeyCode::Right
                | VirtualKeyCode::Up
                | VirtualKeyCode::Down
                | VirtualKeyCode::Back
                | VirtualKeyCode::Return => self.render_cursor(),
                _ => {}
            }
        }

        if self.frame_hash % 3 == 0 {
            self.handle_special_keys(&keys_pressed);
        } else {
            if self.handle_special_keys(&keys_just) {
                self.frame_hash = 0;
            };
        }

        let ctrl = self.keyboard.keys_pressed.contains(&VirtualKeyCode::LControl)
            || self.keyboard.keys_pressed.contains(&VirtualKeyCode::RControl);
        if ctrl {
            if self.frame_hash % 6 == 0 {
                self.handle_shortcuts(&keys_pressed);
            } else {
                if self.handle_shortcuts(&keys_just){
                    self.frame_hash = 0;
                };
            }
        }

        if self.cursor != old_cursor {
            self.selection = shift
                .then(|| {
                    self.selection
                        .map(|(s, _)| (s, self.cursor))
                        .or(Some((old_cursor, self.cursor)))
                })
                .flatten();
        }
    }

    fn handle_special_keys(&mut self, keys: &HashSet<VirtualKeyCode>) -> bool {
        let mut matched = false;
        for key in keys {
            let mut match_this = true;
            match key {
                VirtualKeyCode::Tab => {
                    self.handle_string_input("    ".to_string());
                }
                VirtualKeyCode::Left => {
                    if self.cursor.0 > 0 {
                        self.cursor.0 -= 1;
                    } else if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                        self.cursor.0 = self.file[self.cursor.1].len();
                    }
                }
                VirtualKeyCode::Right => {
                    if self.cursor.0 < self.file[self.cursor.1].len() {
                        self.cursor.0 += 1;
                    } else if self.cursor.1 < self.file.len() - 1 {
                        self.cursor.1 += 1;
                        self.cursor.0 = 0;
                    }
                }
                VirtualKeyCode::Up => {
                    if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                        self.cursor.0 = min(self.cursor.0, self.file[self.cursor.1].len());
                    }
                }
                VirtualKeyCode::Down => {
                    if self.cursor.1 < self.file.len() - 1 {
                        self.cursor.1 += 1;
                        self.cursor.0 = min(self.cursor.0, self.file[self.cursor.1].len());
                    }
                }
                VirtualKeyCode::Back => {
                    if self.selection.is_some() {
                        self.cut_selection();
                        self.clipboard.clear();
                    } else {
                        let cursor_before = self.cursor;
                        let (deleted_text, deleted_start) = if self.cursor.0 > 0 {
                            let (col, row) = self.cursor;
                            let text = self.file[row][col - 1..col].to_string();
                            self.file[row].remove(col - 1);
                            self.cursor.0 -= 1;
                            (text, (col - 1, row))
                        } else if self.cursor.1 > 0 {
                            let line = self.file.remove(self.cursor.1);
                            self.cursor.1 -= 1;
                            self.cursor.0 = self.file[self.cursor.1].len();
                            self.file[self.cursor.1].push_str(&line);
                            ("\n".to_string(), (self.cursor.0, self.cursor.1))
                        } else {
                            continue;
                        };

                        self.push_undo(Change {
                            deleted_text,
                            deleted_start,
                            added_text: String::new(),
                            added_start: deleted_start,
                            cursor_before,
                            cursor_after: self.cursor,
                        });
                    }
                }
                VirtualKeyCode::Return => {
                    let cursor_before = self.cursor;
                    let deleted_text = self.get_selection_text();
                    let deleted_start =
                        self.get_sorted_selection().unwrap_or((self.cursor, self.cursor)).0;
                    self.delete_selection();

                    let added_start = self.cursor;
                    let (col, row) = self.cursor;
                    let line = &self.file[row];
                    let new_line = line[col..].to_string();
                    self.file[row].truncate(col);
                    self.file.insert(row + 1, new_line);
                    self.cursor = (0, row + 1);

                    self.push_undo(Change {
                        deleted_text,
                        deleted_start,
                        added_text: "\n".to_string(),
                        added_start,
                        cursor_before,
                        cursor_after: self.cursor,
                    });
                }
                _ => {
                    match_this = false;
                }
            }
            if match_this { matched = true };
        }

        matched
    }

    //DOES NOT check for ctrl be careful
    fn handle_shortcuts(&mut self, keys: &HashSet<VirtualKeyCode>) -> bool {
        let mut matched = false;
        for key in keys {
            let mut this_matched = true;
            match key {
                VirtualKeyCode::V => {
                    let cursor_before = self.cursor;
                    let deleted_text = self.get_selection_text();
                    let deleted_start =
                        self.get_sorted_selection().unwrap_or((self.cursor, self.cursor)).0;
                    self.delete_selection();

                    let added_start = self.cursor;
                    let added_text = self.clipboard.clone();
                    self.insert_text(&added_text, added_start);

                    let (col, row) = added_start;
                    let lines: Vec<&str> = added_text.split('\n').collect();
                    if lines.len() == 1 {
                        self.cursor = (col + lines[0].len(), row);
                    } else {
                        self.cursor = (lines.last().unwrap().len(), row + lines.len() - 1);
                    }
                    let cursor_after = self.cursor;

                    self.push_undo(Change {
                        deleted_text,
                        deleted_start,
                        added_text,
                        added_start,
                        cursor_before,
                        cursor_after,
                    });
                }
                VirtualKeyCode::Z => {
                    if let Some(change) = self.undo_stack.pop() {
                        self.apply_change(&change.added_text, change.added_start, true);
                        self.apply_change(&change.deleted_text, change.deleted_start, false);
                        self.cursor = change.cursor_before;
                        self.redo_stack.push(change);
                    }
                }
                VirtualKeyCode::R => {
                    if let Some(change) = self.redo_stack.pop() {
                        self.apply_change(&change.deleted_text, change.deleted_start, true);
                        self.apply_change(&change.added_text, change.added_start, false);
                        self.cursor = change.cursor_after;
                        self.undo_stack.push(change);
                    }
                }
                VirtualKeyCode::C => self.clipboard = self.get_selection_text(),
                VirtualKeyCode::X => self.cut_selection(),
                VirtualKeyCode::A => {
                    let end_row = self.file.len() - 1;
                    let end_col = self.file[end_row].len();
                    self.selection = Some(((0, 0), (end_col, end_row)));
                    self.cursor = (end_col, end_row);
                }
                _ => {
                    this_matched = false;
                }
            }
            if this_matched { matched = true };
        }
        
        matched
    }

    pub fn handle_string_input(&mut self, s: String) {
        let cursor_before = self.cursor;
        let deleted_text = self.get_selection_text();
        let deleted_start = self.get_sorted_selection().unwrap_or((self.cursor, self.cursor)).0;
        self.delete_selection();

        let added_start = self.cursor;
        self.insert_text(&s, added_start);
        self.cursor.0 += 1;

        self.push_undo(Change {
            deleted_text,
            deleted_start,
            added_text: s,
            added_start,
            cursor_before,
            cursor_after: self.cursor,
        });
    }

    fn push_undo(&mut self, change: Change) {
        self.undo_stack.push(change);
        self.redo_stack.clear();
    }

    fn apply_change(&mut self, text: &str, start: (usize, usize), is_delete: bool) {
        let lines: Vec<&str> = text.split('\n').collect();
        if is_delete {
            let end = if lines.len() == 1 {
                (start.0 + lines[0].len(), start.1)
            } else {
                (lines.last().unwrap().len(), start.1 + lines.len() - 1)
            };
            self.delete_range(start, end);
        } else {
            self.insert_text(text, start);
        }
    }

    fn get_selection_text(&self) -> String {
        if let Some((start, end)) = self.get_sorted_selection() {
            if start.1 == end.1 {
                self.file[start.1][start.0..end.0].to_string()
            } else {
                let mut text = String::new();
                text.push_str(&self.file[start.1][start.0..]);
                text.push('\n');
                for row in (start.1 + 1)..end.1 {
                    text.push_str(&self.file[row]);
                    text.push('\n');
                }
                text.push_str(&self.file[end.1][..end.0]);
                text
            }
        } else {
            String::new()
        }
    }

    fn get_sorted_selection(&self) -> Option<((usize, usize), (usize, usize))> {
        self.selection.map(|(start, end)| {
            if start.1 > end.1 || (start.1 == end.1 && start.0 > end.0) {
                (end, start)
            } else {
                (start, end)
            }
        })
    }

    fn cut_selection(&mut self) {
        let cursor_before = self.cursor;
        let deleted_text = self.get_selection_text();
        if deleted_text.is_empty() {
            return;
        }

        self.clipboard = deleted_text.clone();
        let deleted_start = self.get_sorted_selection().unwrap().0;
        self.delete_selection();
        let cursor_after = self.cursor;

        self.push_undo(Change {
            deleted_text,
            deleted_start,
            added_text: String::new(),
            added_start: deleted_start,
            cursor_before,
            cursor_after,
        });
    }

    fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) {
        if start.1 == end.1 {
            self.file[start.1].drain(start.0..end.0);
        } else {
            let end_line_content = self.file[end.1][end.0..].to_string();
            self.file[start.1].truncate(start.0);
            self.file[start.1].push_str(&end_line_content);

            let start_line = start.1 + 1;
            let end_line = end.1;
            if start_line <= end_line {
                self.file.drain(start_line..=end_line);
            }
        }
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.get_sorted_selection() {
            self.delete_range(start, end);
            self.cursor = start;
            self.selection = None;
        }
    }

    fn insert_text(&mut self, text: &str, pos: (usize, usize)) {
        let lines: Vec<&str> = text.split('\n').collect();
        let (col, row) = pos;

        if lines.len() == 1 {
            self.file[row].insert_str(col, lines[0]);
        } else {
            let rest_of_line = self.file[row][col..].to_string();
            self.file[row].truncate(col);
            self.file[row].push_str(lines[0]);

            for (i, line) in lines.iter().enumerate().skip(1) {
                self.file.insert(row + i, line.to_string());
            }

            let last_line_index = row + lines.len() - 1;
            self.file[last_line_index].push_str(&rest_of_line);
        }
    }

    fn scroll_to_cursor(&mut self) {
        let visible_rows = SCREEN_SIZE as usize / TEXT_HEIGHT as usize;
        let visible_cols = SCREEN_SIZE as usize / TEXT_WIDTH as usize;

        if self.cursor.1 < self.scroll_offset.1 {
            self.scroll_offset.1 = self.cursor.1;
        }
        if self.cursor.1 >= self.scroll_offset.1 + visible_rows {
            self.scroll_offset.1 = self.cursor.1 - visible_rows + 1;
        }
        if self.cursor.0 < self.scroll_offset.0 {
            self.scroll_offset.0 = self.cursor.0;
        }
        if self.cursor.0 >= self.scroll_offset.0 + visible_cols {
            self.scroll_offset.0 = self.cursor.0 - visible_cols + 1;
        }
    }
}
