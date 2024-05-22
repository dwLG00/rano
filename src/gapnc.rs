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
use regex::Regex;
//use crate::lines;
use crate::gap_buffer;
use crate::colors;
use crate::undo;

type WindowYX = (usize, usize);
type Range = (usize, usize); // Dijkstra range: [a, b)

const INIT_GAP_SIZE: usize = 1024; // This is probably good enough for us to last us for a while
const TAB_SIZE: usize = 4; // Move this into a config file soon

// Enum for increment/decrement - used to adjust highlight regions
pub enum Adjust {
    Increment(usize),
    Decrement(usize)
}

// Gap Editor
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
    select_mode_flag: bool, // This is true if there is text being actively or inactively selected
    select_active: bool, // This is true only if we have just one anchor selected
    select_shift: bool, // This checks if selecting using shift is active
    lmark: usize,
    rmark: usize,
    // Cut/Copy stuff
    clipboard: Vec<Vec<char>>,
    clipboard_cursor: Option<usize>,
    // Save flag
    pub save_flag: bool,
    // Search highlighting
    search_hits: Vec<Range>,
    // History
    history: Vec<undo::ActionGroup>,
    // Other configurations
    tab_size: usize
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
            select_active: false,
            select_shift: false,
            lmark: 0,
            rmark: 0,
            clipboard: Vec::<Vec<char>>::new(),
            clipboard_cursor: None,
            save_flag: true,
            search_hits: Vec::<Range>::new(),
            history: Vec::<undo::ActionGroup>::new(),
            tab_size: TAB_SIZE
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

        // If select mode is on, get the range on which we need to highlight
        let (lmark, rmark) = if self.select_mode_flag {
            if self.select_active {
                if self.buffer.gap_position < self.lmark {
                    (self.buffer.gap_position, self.lmark)
                } else {
                    (self.lmark, self.buffer.gap_position)
                }
            } else {
                (self.lmark, self.rmark)
            }
        } else {
            (0, 0)
        };

        // Display the characters
        for i in start..self.buffer.len() {
            if let Some(ch) = self.buffer.get(i) {
                match ch {
                    '\n' => {
                        getyx(self.window, &mut cur_y, &mut cur_x);
                        wmove(self.window, cur_y + 1, 0);
                    },
                    _ => {
                        if self.select_mode_flag && lmark <= i && i <= rmark {
                            waddch_with_highlight(self.window, *ch as chtype);
                        } else if self.index_in_search_hits(i) {
                            waddch_with_search(self.window, *ch as chtype);
                        } else {
                            waddch(self.window, *ch as chtype);
                        }
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
    }

    pub fn display_at_frame_cursor(&self) {
        self.display(self.frame_cursor);
    }

    pub fn deselect_marks(&mut self) {
        // Sets select_mode_flag to false, and resets all marks

        self.select_mode_flag = false;
        self.select_active = false;
        self.select_shift = false;
        self.lmark = 0;
        self.rmark = 0;
    }

    pub fn move_cursor_to(&mut self) {
        // Move the ncurses cursor to the same location as the text cursor
        let (height, width) = self.size;
        if self.buffer.gap_position < self.frame_cursor {
            // text cursor is out of frame -> move the frame cursor to the text cursor!
            self.frame_cursor = self.buffer.get_left_edge(self.buffer.gap_position);
        } else {
            let mut new_x: i32 = 0;
            let mut new_y: i32 = 0;
            // Loop over all characters between frame cursor and target position
            for pos in self.frame_cursor..self.buffer.gap_position {
                if let Some(ch) = self.buffer.get(pos) {
                if *ch == '\n' {
                        new_x = 0;
                        new_y += 1;
                    } else {
                        new_x += 1;
                        if new_x == width.try_into().unwrap() {
                            new_x = 0;
                            new_y += 1;
                        }
                    }
                }
            }
            if new_y >= height.try_into().unwrap() {
                // Text cursor is out of frame
                if let Some(new_pos) = self.put_on_last_line() {
                    let (new_y, new_x) = new_pos;
                    wmove(self.window, new_y, new_x);
                }
            } else {
                wmove(self.window, new_y, new_x);
            }
        }
    }

    pub fn put_on_last_line(&mut self) -> Option<(i32, i32)> {
        // Considering the text cursor is on the last line,
        // rewrite the frame cursor and return the display location
        // if applicable

        let (height, width) = self.size;

        /*
        self.frame_cursor = self.buffer.seek_back_n_display_lines(self.buffer.gap_position, height, width);
        let (cur_y, cur_x) = self.buffer.count_yx(self.frame_cursor, self.buffer.gap_position, width);
        Some((cur_y as i32, cur_x as i32))
        */
        self.put_on_nth_line(height - 1)
    }

    pub fn put_on_nth_line(&mut self, linecount: usize) -> Option<(i32, i32)> {
        let (height, width) = self.size;
        self.frame_cursor = self.buffer.seek_back_n_display_lines(self.buffer.gap_position, linecount, width);
        let (cur_y, cur_x) = self.buffer.count_yx(self.frame_cursor, self.buffer.gap_position, width);
        Some((cur_y as i32, cur_x as i32))
    }

    pub fn pos(&self) -> usize {
        self.buffer.gap_position
    }

    // Arrow keys
    pub fn scroll_down(&mut self) {
        // Handle the cursor changes for scrolling down

        let (height, width) = self.size;
        let current_xpos = self.buffer.xpos(self.buffer.gap_position, width);

        // Get the new text cursor's position
        let new_text_cursor = if self.smart_cursor_flag {
            self.buffer.seek_next_line_with_xpos(width, self.smart_cursor_pos)
        } else {
            self.buffer.seek_next_line(width)
        };

        match new_text_cursor {
            Some((next_text_pos, less_than_xpos)) => {
                // Move the gap buffer (i.e. text cursor to the right pos)
                self.buffer.move_gap(next_text_pos);
                if less_than_xpos && !self.smart_cursor_flag {
                    // Set the smart cursor flag to the x position if needed
                    self.smart_cursor_flag = true;
                    self.smart_cursor_pos = current_xpos;
                }
            },
            None => {
                // We've hit the bottom display line
                return;
            }
        }
        // Check if we have to move the frame cursor
        if cursor_bottom(self.window, height) {
            // The cursor is on the bottom of the viewport,
            // so we must move the display frame down one
            if let Some(new_frame_cursor) = self.buffer.get_next_display_line_head(self.frame_cursor, width) {
                self.frame_cursor = new_frame_cursor;
            }
        }
        // Move the ncurses display cursor
        self.move_cursor_to();
    }

    pub fn scroll_up(&mut self) {
        // Handle the cursor changes for scrolling up
        let (height, width) = self.size;
        let current_xpos = self.buffer.xpos(self.buffer.gap_position, width);

        let new_text_cursor = if self.smart_cursor_flag {
            self.buffer.seek_prev_line_with_xpos(width, self.smart_cursor_pos)
        } else {
            self.buffer.seek_prev_line(width)
        };

        match new_text_cursor {
            Some((new_text_pos, less_than_xpos)) => {
                self.buffer.move_gap(new_text_pos);
                if less_than_xpos && !self.smart_cursor_flag {
                    self.smart_cursor_flag = true;
                    self.smart_cursor_pos = current_xpos;
                }
            },
            None => {
                // We've hit the top display line
                return;
            }
        }

        // If ncurses cursor is at the top, then try to scroll the entire viewframe up one line
        if cursor_top(self.window) {
            if let Some(new_frame_cursor) = self.buffer.get_prev_display_line_head(self.frame_cursor, width) {
                self.frame_cursor = new_frame_cursor;
            }
        }
        self.move_cursor_to();
    }

    pub fn scroll_right(&mut self) {
        self.smart_cursor_flag = false;
        let (height, width) = self.size;
        let is_right_edge = self.buffer.gap_position == self.buffer.get_right_edge(self.buffer.gap_position);

        if self.buffer.gap_position < self.buffer.len() {
            // Increment gap buffer if the next position is in bound
            self.buffer.move_gap(self.buffer.gap_position + 1);
        }

        // If ncurses cursor is at the bottom right corner, or on the bottom line
        // and at the end of the display line, then try to scroll the entire viewframe
        // down one line
        if cursor_end(self.window, height, width) || cursor_bottom(self.window, height) && is_right_edge {
            if let Some(new_frame_cursor) = self.buffer.get_next_display_line_head(self.frame_cursor, width) {
                self.frame_cursor = new_frame_cursor;
            }
        }
        self.move_cursor_to();
    }

    pub fn scroll_left(&mut self) {
        self.smart_cursor_flag = false;
        let (height, width) = self.size;
        let is_left_edge = self.buffer.get_left_edge(self.buffer.gap_position) == self.buffer.gap_position;

        if self.buffer.gap_position > 0 {
            self.buffer.move_gap(self.buffer.gap_position - 1);
        }

        if cursor_beginning(self.window) || cursor_top(self.window) && is_left_edge {
            if let Some(new_frame_cursor) = self.buffer.get_prev_display_line_head(self.frame_cursor, width) {
                self.frame_cursor = new_frame_cursor;
            }
        }
        self.move_cursor_to();
    }

    // Basic character insert/delete
    pub fn type_character(&mut self, character: char) -> undo::ActionGroup {
        // Handles typing a character
        self.smart_cursor_flag = false;

        // Handle moving the selected regions
        /*
        if self.select_mode_flag && self.select_active {
            self.deselect_marks();
        } else if self.select_mode_flag {
            if self.buffer.gap_position <= self.lmark {
                self.lmark += 1;
            }
            if self.buffer.gap_position <= self.rmark {
                self.rmark += 1;
            }
        }
        */
        self.fix_regions(Adjust::Increment(1));

        let start_gap_position = self.buffer.gap_position; // for history

        self.buffer.insert(character);
        self.move_cursor_to();

        let end_gap_position = self.buffer.gap_position; // for history

        self.set_save(); // Modified the buffer, set flag
        undo::ActionGroup::Singleton(undo::Action::TypeChar(start_gap_position, character, end_gap_position))
    }

    pub fn newline(&mut self) -> undo::ActionGroup {
        self.smart_cursor_flag = false;

        // Handle moving the selected regions
        /*
        if self.select_mode_flag && self.select_active {
            self.deselect_marks();
        } else if self.select_mode_flag {
            if self.buffer.gap_position <= self.lmark {
                self.lmark += 1;
            }
            if self.buffer.gap_position <= self.rmark {
                self.rmark += 1;
            }
        }
        */
        self.fix_regions(Adjust::Increment(1));

        let start_gap_position = self.buffer.gap_position; // for history        

        self.buffer.insert('\n');
        self.move_cursor_to();

        let end_gap_position = self.buffer.gap_position; // for history

        self.set_save(); // Modified the buffer, set flag
        undo::ActionGroup::Singleton(undo::Action::Newline(start_gap_position, end_gap_position))
    }

    pub fn tab(&mut self) -> undo::ActionGroup {
        self.smart_cursor_flag = false;
        let (_, width) = self.size;
        // Calculate cursor position on current line
        let line_pos = self.buffer.gap_position - self.buffer.get_left_edge(self.buffer.gap_position);
        // Calculate displayed x position of cursor
        let display_line_pos = line_pos - (line_pos / width) * width;
        // Calculate number of spaces since last tab "fencepost"
        let mod_tabs = display_line_pos - (display_line_pos / self.tab_size) * self.tab_size;
        // tab_size - mod_tabs = # of spaces left until next fencepost

        // Merge the actions
        let mut actions = Vec::<undo::ActionGroup>::new();
        for i in 0..(self.tab_size - mod_tabs) {
            actions.push(self.type_character(' ')); // We want spaces instead of tabs
        }
        undo::merge_action_groups(actions)
    }

    pub fn backspace(&mut self) -> Option<undo::ActionGroup> {
        // IMPORTANT this method returns an Option<ActionGroup>
        self.smart_cursor_flag = false;

        // Handle moving the selected regions
        /*
        if self.select_mode_flag && self.select_active {
            self.deselect_marks();
        } else if self.select_mode_flag {
            if self.buffer.gap_position <= self.lmark {
                self.lmark -= 1;
            }
            if self.buffer.gap_position <= self.rmark {
                self.rmark -= 1;
            }
        }
        */
        self.fix_regions(Adjust::Decrement(1));

        let start_gap_position = self.buffer.gap_position;

        let ch: char = match self.buffer.pop() {
            Some(c) => {c},
            None => { beep(); return None; } // Trying to delete at head
        }; // Capture the deleted char
        self.move_cursor_to();

        let end_gap_position = self.buffer.gap_position;

        self.set_save(); // Modified the buffer, set flag

        Some(undo::ActionGroup::Singleton(undo::Action::Delete(start_gap_position, ch, end_gap_position)))
    }

    // Advanced navigation
    pub fn next_word(&mut self) {
        // Move cursor to beginning of the next word
        match self.buffer.get(self.buffer.gap_position) {
            Some(c) => {
                let mut ch: char = *c;
                if ch == '\n' || ch == '\t' || ch == ' ' || ch == '\r' {
                    // if cursor is whitespace, jump to the next non-whitespace position
                    while (ch == '\n' || ch == '\t' || ch == ' ' || ch == '\r') && self.buffer.gap_position < self.buffer.len() {
                        self.buffer.move_gap(self.buffer.gap_position + 1);
                        ch = match self.buffer.get(self.buffer.gap_position) {
                            Some(c) => *c,
                            None => { break; }
                        }
                    }
                } else {
                    // if cursor is not whitespace, jump to the next whitespace position
                    while (ch != '\n' && ch != '\t' && ch != ' ' && ch != '\r') && self.buffer.gap_position < self.buffer.len() {
                        self.buffer.move_gap(self.buffer.gap_position + 1);
                        ch = match self.buffer.get(self.buffer.gap_position) {
                            Some(c) => *c,
                            None => { break; }
                        }
                    }
                }
            },
            None => {}
        }
        self.move_cursor_to();
    }

    pub fn prev_word(&mut self) {
        // Move cursor to beginning of the next word
        match self.buffer.get(self.buffer.gap_position) {
            Some(c) => {
                let mut ch: char = *c;
                if ch == '\n' || ch == '\t' || ch == ' ' || ch == '\r' {
                    // if cursor is whitespace, jump to the previous non-whitespace position
                    while (ch == '\n' || ch == '\t' || ch == ' ' || ch == '\r') && self.buffer.gap_position > 0 {
                        self.buffer.move_gap(self.buffer.gap_position - 1);
                        ch = match self.buffer.get(self.buffer.gap_position) {
                            Some(c) => *c,
                            None => { break; }
                        }
                    }
                } else {
                    // if cursor is not whitespace, jump to the previous whitespace position
                    while (ch != '\n' && ch != '\t' && ch != ' ' && ch != '\r') && self.buffer.gap_position > 0 {
                        self.buffer.move_gap(self.buffer.gap_position - 1);
                        ch = match self.buffer.get(self.buffer.gap_position) {
                            Some(c) => *c,
                            None => { break; }
                        }
                    }
                }
            },
            None => {}
        }
        self.move_cursor_to();
    }

    pub fn fast_up(&mut self) {
        // Scrolls up half a screen's worth
        let (height, width) = self.size;
        let attempted_scroll_distance = height / 2;
        let actual_scroll_distance = min(attempted_scroll_distance, self.buffer.current_line);
        for _ in 0..actual_scroll_distance {
            self.scroll_up();
        }

        if self.buffer.current_line == 0 {
            // If we reach the very top, beep
            beep();
        }
    }

    pub fn fast_down(&mut self) {
        // Scrolls up half a screen's worth
        let (height, width) = self.size;
        let attempted_scroll_distance = height / 2;
        let actual_scroll_distance = min(attempted_scroll_distance, max(self.buffer.n_lines - self.buffer.current_line, 1) - 1);
        for _ in 0..actual_scroll_distance {
            self.scroll_down();
        }
        if self.buffer.current_line == self.buffer.n_lines - 1 {
            // If we reach the very top, beep
            beep();
        }
    }

    pub fn fast_right(&mut self) {
        // Scrolls right a tab's worth
        let attempted_scroll_distance = self.tab_size;
        let actual_scroll_distance = min(attempted_scroll_distance, max(self.buffer.len() - self.buffer.gap_position, 1));
        for _ in 0..actual_scroll_distance {
            self.scroll_right();
        }
        if self.buffer.gap_position == self.buffer.len() {
            // If we reach the very top, beep
            beep();
        }
    }

    pub fn fast_left(&mut self) {
        // Scrolls right a tab's worth
        let attempted_scroll_distance = self.tab_size;
        let actual_scroll_distance = min(attempted_scroll_distance, self.buffer.gap_position);
        for _ in 0..actual_scroll_distance {
            self.scroll_left();
        }
        if self.buffer.gap_position == 0 {
            // If we reach the very top, beep
            beep();
        }
    }

    pub fn go_to_line(&mut self, n: usize) {
        // Moves the cursor to the nth line
        if n > self.buffer.n_lines {
            // Out of range!
            beep();
            return;
            panic!("line number out of range!");
        }

        if n == self.buffer.current_line {
            // Edge case: do nothing
            return;
        }

        if n < self.buffer.current_line {
            let mut pointer = self.buffer.get_left_edge(self.buffer.gap_position);
            for i in n..self.buffer.current_line {
                pointer = self.buffer.get_left_edge(pointer - 1);
            }
            self.buffer.move_gap(pointer);
            /*
            match self.buffer.get(self.buffer.gap_position) {
                Some(c) => {panic!("{}", c);},
                _ => {}
            }
            */
        } else {
            let mut pointer = self.buffer.gap_position;
            for i in self.buffer.current_line..n {
                pointer = self.buffer.get_right_edge(pointer) + 1;
            }
            self.buffer.move_gap(pointer);
        }

        let (height, _) = self.size;
        self.put_on_nth_line(height / 2);
        self.move_cursor_to();

    }

    // Selection
    pub fn set_mark(&mut self) {
        // Sets the highlight mark

        if !self.select_mode_flag {
            // No selections right now
            self.lmark = self.buffer.gap_position;
            self.select_mode_flag = true;
            self.select_active = true; // We are actively selecting (i.e. the cursor acts as the second anchor)
        } else if self.select_mode_flag && self.select_active {
            // We are selecting with just one marker -> register
            // the cursor position as the second marker
            if self.lmark == self.buffer.gap_position {
                // We're trying to re-select the mark -> toggle the selection
                self.select_mode_flag = false;
                self.select_active = false;
                return;
            }
            self.rmark = self.buffer.gap_position; // Set the right marker
            if self.rmark < self.lmark {
                // Our cursor's position is before the left marker
                // -> swap the two
                let temp = self.lmark;
                self.lmark = self.rmark;
                self.rmark = temp;
            }
            self.select_active = false;
        } else if self.select_mode_flag && !self.select_active {
            // We already have two markers

            // Remove all anchors and start selecting from
            // the cursor position
            self.lmark = self.buffer.gap_position;
            self.rmark = 0; // erase
            self.select_active = true;

            /*
            if self.buffer.gap_position == self.lmark {
                self.select_active = true;
                self.lmark = self.rmark;
            } else if self.buffer.gap_position == self.rmark {
                self.select_active = true;
            } else if self.buffer.gap_position > self.rmark {
                self.rmark = self.buffer.gap_position;
            } else if self.buffer.gap_position < self.lmark {
                self.lmark = self.buffer.gap_position;
            } else {
                // The cursor is between the left and right marks
                // Temporary design choice: just set the rmark to
                // the cursor
                self.rmark = self.buffer.gap_position;
            }
            */
        }
    }

    pub fn get_select_region(&self) -> (usize, usize) {
        // Get the (inclusive) left and right
        // bounds that are "selected"

        if self.select_mode_flag && self.select_active {
            // 1 mark -> use cursor as the other mark
            if self.lmark < self.buffer.gap_position {
                (self.lmark, self.buffer.gap_position - 1)
            } else {
                (self.buffer.gap_position, self.lmark)
            }
        } else if self.select_mode_flag && !self.select_active {
            // 2 marks -> just use the marks
            (self.lmark, self.rmark)
        } else {
            // 0 marks -> Use the left and right edges
            (self.buffer.get_left_edge(self.buffer.gap_position), self.buffer.get_right_edge(self.buffer.gap_position))
        }
    }

    pub fn set_select_shift(&mut self) {
        self.select_shift = true;
    }

    pub fn is_shift_selected(&self) -> bool {
        // Checks if current selection is shift-selected
        self.select_mode_flag && self.select_shift
    }

    pub fn cut_raw(&mut self, lmark: usize, rmark: usize, new_cursor_pos: usize) -> Vec<char> {
        // Cuts out the selected region, and cuts the text

        // Cleans up highlighting
        let region_size = rmark - lmark + 1;
        self.pop_search_regions(lmark, rmark); // Delete any highlighted regions that would be cut
        self.fix_regions(Adjust::Decrement(region_size));

        let cut_vector = self.buffer.cut(lmark, rmark, new_cursor_pos);

        // Cleanup
        self.smart_cursor_flag = false;
        self.set_save();
        self.deselect_marks();
        self.move_cursor_to();

        cut_vector
    }

    // Cut/Copy/Paste
    pub fn cut(&mut self) -> undo::ActionGroup {
        // Cuts the selected text, or if no text is
        // selected, the current line

        let (lmark, rmark) = self.get_select_region();

        // Get the new cursor position
        let new_cursor_pos = if self.buffer.gap_position < lmark {
            self.buffer.gap_position
            // Don't do anything
        } else if self.buffer.gap_position > rmark {
            // Shift the cursor back the width of
            // the selected region
            self.buffer.gap_position - (rmark - lmark + 1)
        } else {
            // Cursor is in between the two marks
            lmark
        };

        let start_gap_position = self.buffer.gap_position;

        //let cut_vector = self.buffer.cut(lmark, rmark, new_cursor_pos);
        let cut_vector = self.cut_raw(lmark, rmark, new_cursor_pos);
        let cut_string = cut_vector.clone().iter().collect();

        self.clipboard.push(cut_vector);
        self.clipboard_cursor = Some(self.clipboard.len() - 1);

        let end_gap_position = self.buffer.gap_position;

        undo::ActionGroup::Singleton(undo::Action::Cut(start_gap_position, lmark, cut_string, end_gap_position))
    }

    pub fn copy(&mut self) {
        // Copies the current selected region into buffer

        if !self.select_mode_flag {
            // If no text is selected, just beep
            beep();
            return;
        }

        let (lmark, rmark) = self.get_select_region();
        self.clipboard.push(self.buffer.copy(lmark, rmark));
        self.clipboard_cursor = Some(self.clipboard.len() - 1);
    }

    pub fn insert_buffer(&mut self, buffer: &Vec<char>) -> undo::ActionGroup {
        // Inserts buffer at cursor

        // Handle moving the selected regions
        /*
        if self.select_mode_flag && self.select_active {
            self.deselect_marks();
        } else if self.select_mode_flag {
            if self.buffer.gap_position <= self.lmark {
                self.lmark += buffer.len();
            }
            if self.buffer.gap_position <= self.rmark {
                self.rmark += buffer.len();
            }
        }
        */
        self.fix_regions(Adjust::Increment(buffer.len()));

        let start_gap_position = self.buffer.gap_position;
        let paste_string: String = buffer.to_vec().iter().collect();

        self.buffer.insert_buffer(buffer);

        let end_gap_position = self.buffer.gap_position;

        // Cleanup
        self.smart_cursor_flag = false;
        self.set_save();
        self.deselect_marks();

        undo::ActionGroup::Singleton(undo::Action::Insert(start_gap_position, start_gap_position, paste_string, end_gap_position))
    }

    pub fn paste(&mut self) -> Option<undo::ActionGroup> {
        // Pastes the cut buffer at the cursor position
        if let Some(clipboard_cursor) = self.clipboard_cursor {
            match self.clipboard.get(clipboard_cursor) {
                Some(buffer) => { return Some(self.insert_buffer(&((*buffer).clone()))); },
                None => { beep(); return None; }
            }
        }
        None
        //self.insert_buffer(*buffer.clone());
    }

    // Clipboard
    pub fn set_clipboard_cursor(&mut self, pos: usize) {
        // Sets the clipboard cursor
        assert!(pos < self.clipboard.len());
        self.clipboard_cursor = Some(pos);
    }

    pub fn get_clipboard_cursor(&self) -> Option<usize> {
        // Getter
        self.clipboard_cursor
    }

    pub fn get_clipboard(&self, pos: usize) -> Option<&Vec<char>> {
        // Get the clipboard string at position
        assert!(pos < self.clipboard.len());
        self.clipboard.get(pos)
    }

    pub fn clipboard_len(&self) -> usize {
        self.clipboard.len()
    }

    pub fn disable_clipboard(&mut self) {
        // Sets the clipboard cursor to None
        self.clipboard_cursor = None;
    }

    // Search / Find
    pub fn find_raw(&self, search_string: String, start: usize) -> Option<Range> {
        // Searches the buffer from `start` and finds
        // the range of first search result
        let re = match Regex::new(&search_string) {
            Ok(regex) => regex,
            Err(e) => { return None; }
        };
        match re.find(&self.export()[start..]) {
            Some(m) => Some((start + m.start(), start + m.end())),
            None => None
        }
    }

    pub fn find_all(&mut self, search_string: String, start: usize) {
        // Searches the buffer from `start` and finds
        // every range and adds it to search_hits
        let re = match Regex::new(&search_string) {
            Ok(regex) => regex,
            Err(e) => { return; }
        };
        let mut flag = false;
        for m in re.find_iter(&self.export()[start..]) {
            if !flag {
                let s = start + m.start();
                self.buffer.move_gap(s);
                self.move_cursor_to();
                flag = true;
            }
            let range = (start + m.start(), start + m.end());
            self.search_hits.push(range);
        }
    }

    pub fn find(&mut self, search_string: String) {
        // Searches the entire buffer for string
        // and adds range to the search_hits vector
        match self.find_raw(search_string, self.buffer.gap_position) {
            Some((start, end)) => {
                self.buffer.move_gap(start);
                self.search_hits.push((start, end));
                self.move_cursor_to();
            },
            None => {}
        }
    }

    pub fn replace(&mut self, range: (usize, usize), replace_with: String) -> undo::ActionGroup {
        // Replaces the selected range with the given string
        // These are Dijkstra ranges, unlike the select range..

        let (range_l, range_r) = range;
        assert!(range_l <= range_r);
        assert!(range_r < self.buffer.len());

        // This is the new cursor position after cutting AND pasting
        let new_cursor_pos = if self.buffer.gap_position < range_l { // Before the replace region -> do nothing
            self.buffer.gap_position
        } else if self.buffer.gap_position >= range_r { // After the replace region -> Add the difference in lengths
            self.buffer.gap_position + replace_with.len() - (range_r - range_l)
        } else { // Cursor between the replace regions -> move cursor to the end of the replaced string
            range_l + replace_with.len()
        };

        // Move the cursor to range_l after so that we don't have to move the cursor when pasting
        let buffer = self.buffer.cut(range_l, range_r - 1, range_l); // Dijkstra to inclusive range
        let replaced_string = buffer.iter().collect();
        self.buffer.insert_buffer(&replace_with.chars().collect());
        self.buffer.move_gap(new_cursor_pos);
        self.move_cursor_to();

        // Cleanup
        self.smart_cursor_flag = false;
        self.set_save();
        self.deselect_marks();

        undo::ActionGroup::Singleton(undo::Action::Replace(range_l, replaced_string, replace_with.clone()))        
    }

    pub fn replace_all(&mut self, replace_with: String) -> Option<undo::ActionGroup> {
        // Replace all regions in the search_hits vector
        // with the given string

        // Clone search_hits so we don't run into mut borrow issues
        let search_hits = self.search_hits.clone();
        if search_hits.len() == 0 {
            return None;
        }

        let mut pos_diff: i32 = 0; // Use this to adjust the difference in position
        let mut action_groups = Vec::<undo::ActionGroup>::new();
        for (l, r) in search_hits {
            let adj_l = (l as i32 + pos_diff) as usize;
            let adj_r = (r as i32 + pos_diff) as usize;
            self.buffer.move_gap(adj_r); // This makes moving the cursor to the end easier
            action_groups.push(self.replace((adj_l, adj_r), replace_with.clone()));
            pos_diff += replace_with.len() as i32 - (r - l) as i32;

            // Debug
            /*
            werase(self.window);
            self.display_at_frame_cursor();
            self.move_cursor_to();
            wrefresh(self.window);
            debug_wait_for_input(self.window);
            */
        }
        // Delete search highlights
        self.clear_search();

        // After replacing, "search" for the replace term
        let cpos = self.buffer.gap_position; // Store the cursor position, as find_all is movey
        self.find_all(replace_with, 0);
        self.buffer.move_gap(cpos); // Move the cursor back
        //TODO Move find_all logic to a separate non-movey method and call that instead
        self.move_cursor_to();

        // Cleanup
        self.smart_cursor_flag = false;
        self.set_save();
        self.deselect_marks();

        Some(undo::merge_action_groups(action_groups))
    }

    pub fn clear_search(&mut self) {
        // Clears the search_hits vector
        self.search_hits.clear();
    }

    pub fn index_in_search_hits(&self, index: usize) -> bool {
        // Checks if index is in a search_hits
        // range
        for (a, b) in self.search_hits.iter() {
            if *a <= index && index < *b {
                return true;
            }
        }
        false
    }

    // Misc helper functions (aides functions above)
    pub fn fix_regions(&mut self, adjust: Adjust) {
        // Fixes the select and highlight regions after making an edit

        // Handle 
        match adjust {
            Adjust::Increment(amount) => {
                // Handle selection
                if self.select_mode_flag && self.select_active {
                    self.deselect_marks();
                } else if self.select_mode_flag {
                    if self.buffer.gap_position <= self.lmark {
                        self.lmark += amount;
                    }
                    if self.buffer.gap_position <= self.rmark {
                        self.rmark += amount;
                    }
                }

                // Handle highlighted regions
                for i in 0..self.search_hits.len() {
                    let (mut lmark, mut rmark) = self.search_hits[i];
                    if self.buffer.gap_position <= lmark {
                        lmark += amount;
                    }
                    if self.buffer.gap_position <= rmark {
                        rmark += amount;
                    }
                    self.search_hits[i] = (lmark, rmark);
                }
            },
            Adjust::Decrement(amount) => {
                // Handle selection
                if self.select_mode_flag && self.select_active {
                    self.deselect_marks();
                } else if self.select_mode_flag {
                    if self.buffer.gap_position <= self.lmark {
                        self.lmark -= amount;
                    }
                    if self.buffer.gap_position <= self.rmark {
                        self.rmark -= amount;
                    }
                }

                // Handle highlighted regions
                for i in 0..self.search_hits.len() {
                    let (mut lmark, mut rmark) = self.search_hits[i];
                    if self.buffer.gap_position <= lmark {
                        lmark -= amount;
                    }
                    if self.buffer.gap_position <= rmark {
                        rmark -= amount;
                    }
                    self.search_hits[i] = (lmark, rmark);
                }
            }
        }
    }

    pub fn pop_search_regions(&mut self, lmark: usize, rmark: usize) {
        // Removes all search results with bounds within [lmark, rmark]
        self.search_hits = self.search_hits.iter().filter(|(l, r)| *l < lmark || *r > rmark).cloned().collect();
    }

    // History
    pub fn execute_action_group(&mut self, actions: undo::ActionGroup) {
        // Executes an action group
        match actions {
            undo::ActionGroup::Singleton(action) => {
                self.execute_action(action);
            },
            undo::ActionGroup::Multiple(actions) => {
                for action in actions { 
                    self.execute_action(action);
                }
            }
        }
    }

    pub fn execute_action(&mut self, action: undo::Action) {
        // Executes an action
        match action {
            undo::Action::TypeChar(start, ch, end) => {
                self.buffer.move_gap(start);
                self.type_character(ch);
                self.buffer.move_gap(end);
            },
            undo::Action::Newline(start, end) => {
                self.buffer.move_gap(start);
                self.newline();
                self.buffer.move_gap(end);
            },
            undo::Action::Delete(start, _, end) => {
                self.buffer.move_gap(start);
                self.backspace();
                self.buffer.move_gap(end);
            },
            undo::Action::Replace(range_l, replaced, replacing) => {
                let range_r = range_l + replaced.len();
                self.replace((range_l, range_r), replacing.clone());
                self.buffer.move_gap(range_l);
            },
            undo::Action::Cut(start, range_l, cut_string, end) => {
                let range_r = range_l + cut_string.len() - 1;
                self.cut_raw(range_l, range_r, range_l);
                self.buffer.move_gap(end);
            },
            undo::Action::Insert(start, range_l, paste_string, end) => {
                self.buffer.move_gap(range_l);
                self.insert_buffer(&paste_string.chars().collect());
                self.buffer.move_gap(end);
            }
            _ => {}
        }
        self.move_cursor_to();
    }

    pub fn revert(&mut self) {
        // Undos one change

        match self.history.pop() {
            Some(action_group) => {
                self.execute_action_group(action_group.undo());
            },
            None => {
                // history is empty
                beep();
            }
        }
        self.clear_search();
    }

    pub fn push_history(&mut self, action_group: undo::ActionGroup) {
        // Pushes action group to history
        self.history.push(action_group);
    }

    pub fn clear_history(&mut self) {
        // Clears the history
        self.history.clear();
    }

    // Actions that modify history
    pub fn type_character_h(&mut self, ch: char) {
        let ag = self.type_character(ch);
        self.push_history(ag);
    }

    pub fn newline_h(&mut self) {
        let ag = self.newline();
        self.push_history(ag);
    }

    pub fn tab_h(&mut self) {
        let ag = self.tab();
        self.push_history(ag);
    }

    pub fn backspace_h(&mut self) {
        let maybe_ag = self.backspace();
        match maybe_ag {
            Some(ag) => { self.push_history(ag); },
            None => {}
        };
    }

    pub fn cut_h(&mut self) {
        let ag = self.cut();
        self.push_history(ag);
    }

    pub fn paste_h(&mut self) {
        let maybe_ag = self.paste();
        match maybe_ag {
            Some(ag) => { self.push_history(ag); },
            None => { beep(); }
        };
    }

    pub fn replace_h(&mut self, range: (usize, usize), replace_with: String) {
        let ag = self.replace(range, replace_with);
        self.push_history(ag);
    }

    pub fn replace_all_h(&mut self, replace_with: String) {
        let maybe_ag = self.replace_all(replace_with);
        match maybe_ag {
            Some(ag) => { self.push_history(ag); },
            None => {}
        };
    }

    // Saving
    pub fn export(&self) -> String {
        self.buffer.export()
    }

    pub fn set_save(&mut self) {
        // Set the save flag
        self.save_flag = false;
    }

    pub fn unset_save(&mut self) {
        // Unset the save flag
        self.save_flag = true;
    }
}

// Helper functions for ncurses
fn get_window_dimensions(window: WINDOW) -> WindowYX {
    // Return dimensions of terminal window (height, width)
    let mut width = 0;
    let mut height = 0;
    getmaxyx(window, &mut height, &mut width);

    (height as usize, width as usize)
}

fn cursor_top(window: WINDOW) -> bool {
    // Returns whether the cursor is on the top line or not
    let mut cur_x = 0;
    let mut cur_y = 0;

    getyx(window, &mut cur_y, &mut cur_x);
    cur_y == 0
}

fn cursor_bottom(window: WINDOW, height: usize) -> bool {
    // Returns whether the cursor is on the bottom line or not
    let mut cur_x = 0;
    let mut cur_y = 0;
    getyx(window, &mut cur_y, &mut cur_x);
    cur_y == (height - 1) as i32
}

fn cursor_beginning(window: WINDOW) -> bool {
    // Returns whether the cursor is in the top left corner or not
    let mut cur_x = 0;
    let mut cur_y = 0;

    getyx(window, &mut cur_y, &mut cur_x);
    cur_y == 0 && cur_x == 0
}

fn cursor_end(window: WINDOW, height: usize, width: usize) -> bool {
    // Returns whether the cursor is in the bottom right corner or not
    let mut cur_x = 0;
    let mut cur_y = 0;

    getyx(window, &mut cur_y, &mut cur_x);
    cur_y == (height - 1) as i32 && cur_x == (width - 1) as i32
}

fn waddch_with_highlight(window: WINDOW, ch: chtype) {
    // Add character with background highlighting
    //wattron(window, COLOR_PAIR(1)); // CP_HIGHLIGHT
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    waddch(window, ch);
    //wattroff(window, COLOR_PAIR(1));
    wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
}

fn waddch_with_search(window: WINDOW, ch: chtype) {
    // Add character with background search highlighting
    wattron(window, COLOR_PAIR(colors::CP_SEARCH));
    waddch(window, ch);
    wattroff(window, COLOR_PAIR(colors::CP_SEARCH));
}

fn debug_wait_for_input(window: WINDOW) {
    let ch = wget_wch(window);
    match ch {
        Some(WchResult::Char(char_code)) => {
            let c = char::from_u32(char_code as u32).expect("Invalid char");
            match c {
                '\u{0003}' => {
                    panic!();
                },
                _ => {}
            }
        },
        _ => {}
    }
}
