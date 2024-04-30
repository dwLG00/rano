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
    size: WindowYX
}

impl Editor {
    pub fn new() -> Editor {
        // Blank editor instance

        let (_, width) = get_window_dimensions();
        Editor::from_line_arena(lines::LineArena::new(width))
    }

    pub fn from_file(file: fs::File) -> Editor {
        // New Editor instance from a file

        let (_, width) = get_window_dimensions();
        Editor::from_line_arena(lines::LineArena::from_file(file, width))
    }

    pub fn from_line_arena(line_arena: lines::LineArena) -> Editor {
        // New Editor instance from LineArena

        let size = get_window_dimensions();
        let head = line_arena.get_head().clone();

        Editor {
            line_arena: line_arena,
            cursor_text: (head, 0),
            cursor_display: (0, 0),
            cursor_frame: (head, 0),
            size: size
        }
    }

    pub fn display(buffer_slice: BufferSlice, start_at_beginning: bool, move_back: bool) {
        // Displays the contents of a Vec<Vec<char>>

        // Store the beginning ncurses cursor location in case move_back == true
        let mut start_x = 0;
        let mut start_y = 0;
        getyx(stdscr(), &mut start_x, &mut start_y);

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
            getyx(stdscr(), &mut cur_y, &mut cur_x);
            mv(cur_y + 1, 0);
        }

        if move_back {
            mv(start_x, start_y);
        }
    }

    pub fn display_at_cursor(&mut self) {
        // Display the file, starting at frame cursor

        let (height, width) = self.size;
        let (maybe_index, line_height) = self.cursor_frame;
        if let Some(index) = maybe_index {
            let buffer = self.line_arena.display_frame_from(index, width, height + line_height);
            if buffer.len() > line_height {
                Editor::display(&buffer[line_height..], false, false);
            }
        }
    }

    pub fn scroll_down(&mut self, display_after: bool) {
        // Scroll the text cursor down, and modify other cursors as appropriate
    }
}

fn get_window_dimensions() -> WindowYX {
    // Return dimensions of terminal window (height, width)
    let mut width = 0;
    let mut height = 0;
    getmaxyx(stdscr(), &mut height, &mut width);

    (height as usize, width as usize)
}
