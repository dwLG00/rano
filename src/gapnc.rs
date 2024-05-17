extern crate ncurses;
extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;
use std::char;
use ncurses::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::Path;
use std::cmp::{min, max};
use crate::lines;
use crate::gap_buffer;

type WindowYX = (usize, usize);

const INIT_GAP_SIZE: usize = 1024;

pub struct GapEditor {
    buffer: gap_buffer::GapBuffer,
    // Cursors
    frame_cursor: usize, // Text Cursor is stored in buffer; Text Cursor = Display Cursor
    // Basic editor fields
    size: WindowYX,
    window: WINDOW,
    // Smart cursor fields
    smart_cursor_flag: bool,
    smart_cursor_pos: usize,
    // Select mode fields
    select_mode_flag: bool,
    lmark_pos: usize,
    rmark_pos: usize,
    // Cut/Copy stuff
    cut_buffer: Vec<char>,
    // Save flag
    pub save_flag: bool
}

impl GapEditor {
    pub fn from_file(file: fs::File, window: WINDOW) -> GapEditor {
        // Creates a new GapEditor from a file
        let buffer = gap_buffer::GapBuffer::new_from_file(file, INIT_GAP_SIZE);
        GapEditor::from_buffer(buffer, window)
    }

    pub fn from_buffer(buffer: gap_buffer::GapBuffer, window: WINDOW) -> GapEditor {
        // Creates a new GapEditor from the provided GapBuffer
        let size = get_window_dimensions(window);

        GapEditor {
            buffer: buffer,
            frame_cursor: 0,
            size: size,
            window: window,
            smart_cursor_flag: false,
            smart_cursor_pos: 0,
            select_mode_flag: false,
            lmark_pos: 0,
            rmark_pos: 0,
            cut_buffer: Vec::<char>::new(),
            save_flag: true
        }
    }

    pub fn display(&self, start: usize) {
        // Starts with the char at start, and outputs all characters that will fit in window
        let (height, width) = self.size;
        let mut start_x = 0;
        let mut start_y = 0;
        getyx(self.window, &mut start_y, &mut start_x);

        wmove(self.window, 0, 0);

        let mut cur_y = 0;
        let mut cur_x = 0;
        let mut new_y = 0;
        let mut new_x = 0;

        for i in start..self.buffer.len() {
            let ch = self.buffer.get(i);
            match ch {
                '\n' => {
                    getyx(self.window, &mut cur_y, &mut cur_x);
                    if cur_x == 0 {
                    } else {
                        wmove(self.window, cur_y + 1, 0);
                    }
                },
                _ => {
                    waddch(self.window, *ch as chtype);
                }
            }
            getyx(self.window, &mut cur_y, &mut cur_x);
            if cur_y == new_y && cur_x == new_x {
                break;
            }
            new_y = cur_y;
            new_x = cur_x;
        }
    }

    pub fn display_at_cursor(&self) {
        self.display(self.frame_cursor);
    }

    pub fn deselect_all(&mut self) {
        // Sets select_mode_flag to false, and resets all marks

        self.select_mode_flag = false;
        self.lmark_pos = 0;
        self.rmark_pos = 0;
    }

    pub fn move_cursor_to(&mut self) {
        // Move the ncurses cursor to the same location as the text cursor
        if self.buffer.gap_position < self.frame_cursor {
            // text cursor is out of frame -> move the frame cursor to the text cursor!
            self.frame_cursor = self.buffer.gap_position;
        } else {
            let mut new_x: i32 = 0;
            let mut new_y: i32 = 0;
            // Loop over all characters between frame cursor and target position
            for pos in self.frame_cursor..self.buffer.gap_position {
                if self.buffer.get(pos) == '\n' {
                    new_x = 0;
                    new_y += 1;
                } else {
                    new_x += 1;
                    if new_x == width {
                        new_x = 0;
                        new_y += 1;
                    }
                }
            }
            if new_y >= height {
                // Text cursor is out of frame
                if let Some(new_pos) = self.put_on_last_line() {
                    let new_y, new_x = new_pos;
                    wmove(self.window, new_y, new_x);
                }
            } else {
                wmove(self.window, new_y, new_x);
            }
        }
    }

    pub fn put_on_last_line(&mut self) -> Option<(usize, usize)> {
        // Considering the text cursor is on the last line,
        // rewrite the frame cursor and return the display location
        // if applicable
    }

}


fn get_window_dimensions(window: WINDOW) -> WindowYX {
    // Return dimensions of terminal window (height, width)
    let mut width = 0;
    let mut height = 0;
    getmaxyx(window, &mut height, &mut width);

    (height as usize, width as usize)
}
