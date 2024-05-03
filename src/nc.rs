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
    window: WINDOW,
    smart_cursor: bool, // smart cursor flag
    smart_cursor_pos: usize
}

impl Editor {
    pub fn new(window: WINDOW) -> Editor {
        // Blank editor instance

        let (_, width) = get_window_dimensions(window);
        Editor::from_line_arena(lines::LineArena::new(width), window)
    }

    pub fn blank(window: WINDOW) -> Editor {
        let (_, width) = get_window_dimensions(window);
        let mut line_arena = lines::LineArena::new(width);
        line_arena.append(lines::Line::new());
        Editor::from_line_arena(line_arena, window)
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
            window: window,
            smart_cursor: false,
            smart_cursor_pos: 0
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
            wmove(window, 0, 0);
        }

        // Display each line
        for line in buffer_slice.iter() {
            for ch in line.iter() {
                waddch(window, *ch as chtype);
            }
            // At end of each Vec<char>, move cursor to the next line
            let mut cur_x = 0;
            let mut cur_y = 0;
            getyx(window, &mut cur_y, &mut cur_x);

            if cur_x == 0 && line.len() > 0 {
                // If cur_x ends up being 0 after printing a lot on screen, then
                // it means the cursor wrapped around, so cur_y already got incremented
            } else {
                wmove(window, cur_y + 1, 0);
            }
        }

        if move_back {
            wmove(window, start_x, start_y);
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

    pub fn move_cursor_to(&mut self, window: WINDOW) {
        // Move cursor to cursor_display

        let (cur_y, cur_x) = self.cursor_display;
        wmove(window, cur_y as i32, cur_x as i32);
    }

    pub fn scroll_down(&mut self, display_after: bool) {
        // Scroll the text cursor down, and modify other cursors as appropriate

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        let mut next_display_cursor = 0;
        let mut nextline_len = 0;

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos + width >= self.line_arena.get(line_index).len() {
                // We've jumped to the next Line
                match self.line_arena.get(line_index).nextline {
                    Some(next_index) => {
                        next_display_cursor = min(self.line_arena.get(next_index).len(), cur_x);
                        nextline_len = min(self.line_arena.get(next_index).len(), width - 1);
                        // If smart cursor flag isn't set, then set it and store current x pos
                        if !self.smart_cursor {
                            self.smart_cursor = true;
                            self.smart_cursor_pos = cur_x;
                        }
                        self.cursor_text = (Some(next_index), next_display_cursor);
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        beep();
                        return
                    }
                }
            } else {
                next_display_cursor = cur_x;
                self.cursor_text = (maybe_text_line_index, line_pos + width);
            }
        }

        let mut next_display_pos;
        if self.smart_cursor {
            next_display_pos = min(max(next_display_cursor, self.smart_cursor_pos), nextline_len);
        } else{
            next_display_pos = next_display_cursor;
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
                self.cursor_display = (cur_y, next_display_pos);
            }
            // cursor_display stays the same
        } else {
            self.cursor_display = (cur_y + 1, next_display_pos);
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

        let mut next_display_cursor = 0;
        let mut prevline_len = 0;

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos < width {
                // We've jumped to the previous Line
                match self.line_arena.get(line_index).prevline {
                    Some(prev_index) => {
                        // Check if the tail of the previous line is less than cur_x
                        let prev_len = self.line_arena.get(prev_index).len();
                        let tail_len = self.line_arena.get(prev_index).tail_len(width);
                        next_display_cursor = min(tail_len, cur_x);
                        prevline_len = min(self.line_arena.get(prev_index).len(), width - 1);

                        // If smart cursor flag isn't set, then set it and store current x pos
                        if !self.smart_cursor {
                            self.smart_cursor = true;
                            self.smart_cursor_pos = cur_x;
                        }

                        if tail_len < cur_x {
                            self.cursor_text = (Some(prev_index), prev_len);
                        } else {
                            self.cursor_text = (Some(prev_index), prev_len - tail_len + cur_x);
                        }
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        beep();
                        return
                    }
                }
            } else {
                next_display_cursor = cur_x;
                self.cursor_text = (maybe_text_line_index, line_pos - width);
            }
        }

        let mut next_display_pos;
        if self.smart_cursor {
            next_display_pos = min(max(next_display_cursor, self.smart_cursor_pos), prevline_len);
        } else{
            next_display_pos = next_display_cursor;
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
            self.cursor_display = (cur_y, next_display_pos);
        } else {
            self.cursor_display = (cur_y - 1, next_display_pos);
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

        // Disable flag
        self.smart_cursor = false;

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos + 1 > self.line_arena.get(line_index).len() {
                // We've jumped to the next Line
                match self.line_arena.get(line_index).nextline {
                    Some(next_index) => {
                        self.cursor_text = (Some(next_index), 0);
                    },
                    None => {
                        // We've reached the end of the text => don't change anything
                        beep();
                        return
                    }
                }
                next_line_flag = true;
            } else {
                self.cursor_text = (maybe_text_line_index, line_pos + 1);
            }
        }

        // Update cursor_display and cursor_frame
        if (next_line_flag || cur_x + 1 == width) && cur_y + 1 == height {
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

    pub fn scroll_left(&mut self, display_after: bool) {
        // Scroll the text cursor left, and modify other cursors as appropriate

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        let mut prev_line_flag = false;
        let mut next_display_cursor = 0;

        // Disable flag
        self.smart_cursor = false;

        // Update cursor_text
        if let Some(line_index) = maybe_text_line_index {
            if line_pos == 0 {
                // We've jumped to the previous Line
                match self.line_arena.get(line_index).prevline {
                    Some(prev_index) => {
                        let prev_len = self.line_arena.get(prev_index).len();
                        let tail_len = self.line_arena.get(prev_index).tail_len(width);
                        next_display_cursor = tail_len; // This is where the display cursor cur_x will be set to

                        self.cursor_text = (Some(prev_index), prev_len);
                    },
                    None => {
                        // We've reached the beginning of the text => don't change anything
                        beep();
                        return
                    }
                }
                prev_line_flag = true;
            } else {
                next_display_cursor = width - 1; // Wrap around
                self.cursor_text = (maybe_text_line_index, line_pos - 1);
            }
        }

        // Update cursor_display and cursor_frame
        if cur_x == 0 && cur_y == 0 {
            // Can't scroll past
            if let Some(frame_line_index) = maybe_frame_line_index {
                if line_height == 0 {
                    // Move to the previous line
                    let prevline = self.line_arena.get(frame_line_index).prevline;
                    if let Some(prev) = prevline {
                        let prev_height = self.line_arena.get(prev).height(width);
                        self.cursor_frame = (prevline, prev_height - 1);
                    }
                } else {
                    self.cursor_frame = (maybe_frame_line_index, line_height - 1);
                }
                self.cursor_display = (cur_x, next_display_cursor);
            }
        } else {
            if cur_x == 0 || prev_line_flag {
                self.cursor_display = (cur_y - 1, next_display_cursor);
            } else {
                self.cursor_display = (cur_y, cur_x - 1);
            }
        }

        if display_after {
            self.display_at_frame_cursor();
        }
    }

    pub fn type_character(&mut self, character: char, display_after: bool) {
        // Handles typing a character
        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        // Insert the character
        if let Some(text_line_index) = maybe_text_line_index {
            self.line_arena.get_mut(text_line_index).insert_char(line_pos, character);
        }

        // Move the cursor right
        self.scroll_right(display_after);

    }

    pub fn newline(&mut self, display_after: bool) {
        // Handles newline
        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        // Disable flag
        self.smart_cursor = false;

        // handle frame and display cursors
        if let Some(text_line_index) = maybe_text_line_index {
            if line_pos == 0 && self.line_arena.get(text_line_index).len() != 0 {
                let is_head = match self.line_arena.get(text_line_index).prevline {
                    None => true,
                    _ => false
                }; // If is_head, then we move the frame cursor back
                let behind = self.line_arena.split(text_line_index, line_pos);

                // Check if we need to shift frame down
                if cur_y + 1 == height {
                    if let Some(frame_line_index) = maybe_frame_line_index {
                        if line_height + 1 >= self.line_arena.get(frame_line_index).height(width) {
                            // Move frame cursor
                            self.cursor_frame = (self.line_arena.get(frame_line_index).nextline, 0);
                        } else {
                            self.cursor_frame = (maybe_frame_line_index, line_height + 1);
                        }
                        // Don't change the display cursor
                    }
                } else {
                    if is_head {
                        if let Some(frame_line_index) = maybe_frame_line_index {
                            self.cursor_frame = (self.line_arena.get(frame_line_index).prevline, 0);
                        }
                    }
                    self.cursor_display = (cur_y + 1, cur_x);
                }
                if display_after {
                    self.display_at_frame_cursor();
                }
            } else {
                self.line_arena.split(text_line_index, line_pos);
                self.scroll_right(display_after);
            }
        }
    }

    pub fn backspace(&mut self, display_after: bool) {
        // Handles hitting backspace

        let (cur_y, cur_x) = self.cursor_display; // Display cursor position
        let (height, width) = self.size;
        let (maybe_frame_line_index, line_height) = self.cursor_frame; // Line and display line at top of window
        let (maybe_text_line_index, line_pos) = self.cursor_text; // Line and position of cursor (internal representation)

        // Disable flag
        self.smart_cursor = false;

        if let Some(text_line_index) = maybe_text_line_index {
            if line_pos > 0 { // We are in the middle of a line
                // The cursor position is one space in front of the character
                // we want to delete

                self.line_arena.get_mut(text_line_index).pop_char(line_pos - 1);
                self.cursor_text = (maybe_text_line_index, line_pos - 1);

                if cur_y == 0 && cur_x == 0 { // At top of screen -> move frame cursor, wrap around
                    self.cursor_frame = (maybe_frame_line_index, line_height - 1);
                    self.cursor_display = (cur_y, width - 1);
                } else if cur_x == 0 { // At left -> wrap around
                    self.cursor_display = (cur_y - 1, width - 1);
                } else { // Just move left
                    self.cursor_display = (cur_y, cur_x - 1);
                }
            } else { // We're at the front of the line, which means we need to merge this line with the line behind it
                match self.line_arena.get(text_line_index).prevline {
                    Some(prev) => {
                        let new_line_pos = self.line_arena.get(prev).len();
                        let new_display_pos = self.line_arena.get(prev).tail_len(width);
                        self.line_arena.merge(prev);

                        self.cursor_text = (Some(prev), new_line_pos);

                        if cur_y == 0 { // We're at the top -> prevline should be the new frame cursor
                            self.cursor_frame = (Some(prev), self.line_arena.get(prev).height(width) - 1);
                            self.cursor_display = (cur_y, new_display_pos);
                        } else {
                            self.cursor_display = (cur_y - 1, new_display_pos);
                        }
                    },
                    None => { // We're at the very top of the file => do nothing
                        return;
                    }
                }
            }
        }
        if display_after {
            self.display_at_frame_cursor();
        }
    }

    pub fn export(&self) -> String {
        // Returns the contents as string
        self.line_arena.export()
    }
}

fn get_window_dimensions(window: WINDOW) -> WindowYX {
    // Return dimensions of terminal window (height, width)
    let mut width = 0;
    let mut height = 0;
    getmaxyx(window, &mut height, &mut width);

    (height as usize, width as usize)
}
