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
type DisplayCursor = (usize, usize);
type Buffer = Vec<Vec<char>>;

pub struct Editor {
    line_arena: lines::LineArena,
    cursor_text: TextCursor,
    cursor_display: DisplayCursor
}

impl Editor {
    pub fn new() -> Editor {
        Editor{ line_arena: lines::LineArena::new(), cursor_text: (None, 0), cursor_display: (0, 0)}
    }

    pub fn display(buffer: Buffer, width: usize, height: usize, start_at_beginning: bool, move_back: bool) {
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
        for line in buffer.iter() {
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
}
