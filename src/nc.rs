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

type TextCursor = (Option<Index>, usize);
type FrameCursor = (Option<Index>, usize);
type WindowYX = (usize, usize);
type Buffer = Vec<Vec<char>>;
type BufferSlice<'a> = &'a [Vec<char>];

pub struct Editor {
    line_arena: lines::LineArena,
    cursor_text: TextCursor,
    cursor_display: WindowYX,
    cursor_frame: FrameCursor,
    size: WindowYX,
    window: WINDOW
}

impl Editor {
    pub fn new(window: WINDOW) -> Editor {
        // Blank editor instance

        let (_, width) = get_window_dimensions(window);
        Editor::from_line_arena(lines::LineArena::new(width), window)
    }

    pub fn from_file(file: fs::File, window: WINDOW) -> Editor {
        // New Editor instance from a file

        let (_, width) = get_window_dimensions(window);
        Editor::from_line_arena(lines::LineArena::from_file(file, width), window)
    }

    pub fn from_line_arena(line_arena: lines::LineArena, window: WINDOW) -> Editor {
        // New Editor instance from LineArena

        let size = get_window_dimensions(window);
        let head = line_arena.get_head().clone();

        Editor {
            line_arena: line_arena,
            cursor_text: (head, 0),
            cursor_display: (0, 0),
            cursor_frame: (head, 0),
            size: size,
            window: window
        }
    }

    pub fn display(window: WINDOW, buffer_slice: BufferSlice, start_at_beginning: bool, move_back: bool) {
        // Displays the contents of a Vec<Vec<char>>

        // Store the beginning ncurses cursor location in case move_back == true
        let mut start_x = 0;
        let mut start_y = 0;
        getyx(window, &mut start_x, &mut start_y);

        // Move to beginning if start_at_beginning == true
        if start_at_beginning {
            mv(0, 0);
        }

        // Display each line
        for line in buffer_slice.iter() {
            for ch in line.iter() {
                addch(*ch as chtype);
            }
            // At end of each Vec<char>, move cursor to the next line
            let mut cur_x = 0;
            let mut cur_y = 0;
            getyx(window, &mut cur_y, &mut cur_x);
            mv(cur_y + 1, 0);
        }

        if move_back {
            mv(start_x, start_y);
        }
    }

    pub fn display_at_frame_cursor(&mut self) {
        // Display the file, starting at frame cursor

        let (height, width) = self.size;
        let (maybe_index, line_height) = self.cursor_frame;
        if let Some(index) = maybe_index {
            let buffer = self.line_arena.display_frame_from(index, width, height + line_height);
            if buffer.len() > line_height {
                Editor::display(self.window, &buffer[line_height..], false, false);
            }
        }
    }

    pub fn move_cursor_to(&mut self) {
        // Move cursor to cursor_display

        let (cur_y, cur_x) = self.cursor_display;
        mv(cur_y as i32, cur_x as i32);
    }

    pub fn scroll_down(&mut self, display_after: bool) {
        // Scroll the text cursor down, and modify other cursors as appropriate

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos + width >= self.line_arena.get(line_index).len() {
                // We've jumped to the next Line
                match self.line_arena.get(line_index).nextline {
                    Some(next_index) => {
                        self.cursor_text = (Some(next_index), min(self.line_arena.get(next_index).len(), cur_x));
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        return
                    }
                }
            } else {
                self.cursor_text = (maybe_text_line_index, line_pos + width);
            }
        }

        // Update cursor_display and cursor_frame
        if cur_y + 1 == height {
            // Can't scroll past
            if let Some(frame_line_index) = maybe_frame_line_index {
                if line_height + 1 == self.line_arena.get(frame_line_index).height(width) {
                    // Move to the next line
                    self.cursor_frame = (self.line_arena.get(frame_line_index).nextline, 0);
                } else {
                    self.cursor_frame = (maybe_frame_line_index, line_height + 1);
                }
            }
            // cursor_display stays the same
        } else {
            self.cursor_display = (cur_y + 1, cur_x);
        }

        if display_after {
            self.display_at_frame_cursor();
        }
    }

    pub fn scroll_up(&mut self, display_after: bool) {
        // Scroll the text cursor up, and modify other cursors as appropriate

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos < width {
                // We've jumped to the previous Line
                match self.line_arena.get(line_index).prevline {
                    Some(prev_index) => {
                        // Check if the tail of the previous line is less than cur_x
                        let prev_len = self.line_arena.get(prev_index).len();
                        let tail_length = prev_len - (prev_len / width) * width; // Mod width
                        if tail_length < cur_x {
                            self.cursor_text = (Some(prev_index), prev_len);
                        } else {
                            self.cursor_text = (Some(prev_index), (prev_len / width) * width + cur_x);
                        }
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        return
                    }
                }
            } else {
                self.cursor_text = (maybe_text_line_index, line_pos - width);
            }
        }

        // Update cursor_display and cursor_frame
        if cur_y == 0 {
            // Can't scroll past
            if let Some(frame_line_index) = maybe_frame_line_index {
                if line_height == 0 {
                    // Move to the previous line
                    let prevline = self.line_arena.get(frame_line_index).prevline;
                    if let Some(prev) = prevline {
                        let prev_height = self.line_arena.get(prev).height(width);
                        self.cursor_frame = (prevline, prev_height - 1);
                    } else {
                        // We've hit the top
                        return;
                    }
                } else {
                    self.cursor_frame = (maybe_frame_line_index, line_height - 1);
                }
            }
            // cursor_display stays the same
        } else {
            self.cursor_display = (cur_y - 1, cur_x);
        }

        if display_after {
            self.display_at_frame_cursor();
        }
    }

    pub fn scroll_right(&mut self, display_after: bool) {
        // Scroll the text cursor right, and modify other cursors as appropriate

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        let mut next_line_flag = false;

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos + 1 == self.line_arena.get(line_index).len() {
                // We've jumped to the next Line
                match self.line_arena.get(line_index).nextline {
                    Some(next_index) => {
                        self.cursor_text = (Some(next_index), 0);
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        return
                    }
                }
                next_line_flag = true;
            } else {
                self.cursor_text = (maybe_text_line_index, line_pos + 1);
            }
        }

        // Update cursor_display and cursor_frame
        if cur_x + 1 == width && cur_y + 1 == height {
            // Can't scroll past
            if let Some(frame_line_index) = maybe_frame_line_index {
                if line_height + 1 == self.line_arena.get(frame_line_index).height(width) {
                    // Move to the next line
                    self.cursor_frame = (self.line_arena.get(frame_line_index).nextline, 0);
                } else {
                    self.cursor_frame = (maybe_frame_line_index, line_height + 1);
                }
                self.cursor_display = (cur_y, 0);
            }
        } else {
            if cur_x + 1 == width || next_line_flag {
                self.cursor_display = (cur_y + 1, 0);
            } else {
                self.cursor_display = (cur_y, cur_x + 1);
            }
        }

        if display_after {
            self.display_at_frame_cursor();
        }
    }
}

fn get_window_dimensions(window: WINDOW) -> WindowYX {
    // Return dimensions of terminal window (height, width)
    let mut width = 0;
    let mut height = 0;
    getmaxyx(window, &mut height, &mut width);

    (height as usize, width as usize)
}
