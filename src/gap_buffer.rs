use std::fs;
use std::io::Read;

pub struct GapBuffer {
    pub buffer: Vec<char>,
    pub gap_position: usize,
    gap_size: usize,
    next_alloc_gap_size: usize
}

impl GapBuffer {
    pub fn new_from_file(mut file: fs::File, gap_size: usize) -> GapBuffer {
        // Read a file into a GapBuffer, with a certain starting gap size and
        // with the gap at the very beginning
        let mut buffer =  Vec::<char>::new();
        let mut buffer_str = String::new();
        file.read_to_string(&mut buffer_str);

        for _ in 0..gap_size {
            // Push null char for the gap buffer
            buffer.push('\0');
        }

        for ch in buffer_str.chars() {
            buffer.push(ch);
        }

        GapBuffer { buffer: buffer, gap_position: 0, gap_size: gap_size, next_alloc_gap_size: gap_size * 2 }
    }

    pub fn len(&self) -> usize {
        // Get length of buffer, not counting the gap
        self.buffer.len() - self.gap_size
    }

    pub fn get(&self, idx: usize) -> Option<&char> {
        // Get the character at index, not counting the gap
        if idx >= self.len() {
            None
        } else if idx < self.gap_position {
            Some(&self.buffer[idx])
        } else {
            Some(&self.buffer[idx + self.gap_size])
        }
    }

    pub fn move_gap(&mut self, new_pos: usize) {
        // Moves the gap of the gap buffer
        assert!(new_pos < self.len());

        if new_pos > self.gap_position {
            for i in self.gap_position..new_pos {
                let ch = self.buffer[i];
                self.buffer[i] = self.buffer[i + self.gap_size];
                self.buffer[i + self.gap_size] = ch;
            }
        } else if new_pos < self.gap_position {
            for i in (new_pos..self.gap_position).rev() {
                // Iterate from gap_position down to new_pos
                let ch = self.buffer[i];
                self.buffer[i] = self.buffer[i + self.gap_size];
                self.buffer[i + self.gap_size] = ch;
            }
        } else {
            // Do nothing;
        }
        self.gap_position = new_pos;
    }

    pub fn realloc(&mut self) {
        // Reallocate the buffer (gap ran out)
        let new = vec!['\0'; self.next_alloc_gap_size];
        self.buffer.splice(self.gap_position..self.gap_position, new);

        /*
        for i in 0..self.next_alloc_gap_size {
            self.buffer.insert(self.gap_position + i, '\0');
        }
        */

        // Modify gap sizes
        self.gap_size = self.next_alloc_gap_size;
        self.next_alloc_gap_size = self.next_alloc_gap_size * 2;
    }

    pub fn insert(&mut self, ch: char) {
        // Inserts a character into gap
        if self.gap_size == 0 {
            self.realloc();
        }

        self.buffer[self.gap_position] = ch;
        self.gap_position += 1;
        self.gap_size -= 1;
    }

    pub fn delete(&mut self) {
        // Removes the character behind gap position
        if self.gap_position == 0 {
            panic!("Tried to delete character behind gap_position, but gap_position is 0!");
        }

        self.buffer[self.gap_position - 1] = '\0';
        self.gap_position -= 1;
        self.gap_size += 1;
    }

    pub fn get_left_edge(&self, start: usize) -> usize {
        // Get index of the left edge (last character to the left
        // of start that is not a newline)

        if start == 0 {
            // Edge case: we start at the very beginning
            return 0;
        }

        let mut pointer = start - 1;

        // Loop needs to decrement pointer, only loop until pointer == 0
        while pointer > 0 {
            if let Some(ch) = self.get(pointer) {
                if *ch == '\n' {
                    return pointer + 1;
                }
            }
            pointer -= 1;
        }
        // Edge: last case
        if let Some(ch) = self.get(0) {
            if *ch == '\n' {
                return 1;
            }
        }
        // None of the characters are newlines -> return first index
        0
    }

    pub fn seek_back_n_lines(&self, start: usize, n_lines: usize) -> usize {
        // Seeks back n actual lines, and returns the left edge of
        // the very first line

        let mut pointer = start;
        for i in 1..=n_lines {
            let left_edge = self.get_left_edge(pointer);
            if left_edge == 0 {
                return 0;
            }
            if i < n_lines {
                pointer = left_edge - 1;
            } else {
                pointer = left_edge;
            }
        }
        pointer
    }

    pub fn seek_back_n_display_lines(&self, start: usize, n_lines: usize, width: usize) -> usize {
        // Seek back n display lines, where the displayed line length
        // is given

        if start == 0 {
            // Edge case: start at beginning
            return 0;
        }

        let mut y_count = 0;
        let mut pointer = start - 1;

        // Continue seeking actual lines until we match or
        // overshoot the number of display lines
        while y_count < n_lines {
            let left_edge = self.get_left_edge(start);
            let line_height = 1 + (start - left_edge) / width;
            y_count += line_height;
            if left_edge == 0 {
                // We've reached the top of the viewport, so
                // just return 0
                return 0;
            } else {
                pointer = left_edge - 1;
            }
        }

        if y_count == n_lines {
            // We've successfully seeked to the left edge of
            // the beginning line (except we've decremented by 1)
            pointer + 1
        } else {
            // We've overshot by some amount of display lines, so
            // add the display lines we've overshot back
            pointer + (n_lines - y_count) * width + 1
        }
    }

    pub fn count_yx(&self, start: usize, end: usize, width: usize) -> (usize, usize) {
        // Count the position of `end`, if `start` was at (0, 0)
        assert!(start < self.len() && end < self.len());

        let mut cur_y = 0;
        let mut cur_x = 0;

        for i in start..end {
            if let Some(ch) = self.get(i) {
                if *ch == '\n' {
                    cur_y += 1;
                    cur_x = 0;
                } else {
                    cur_x += 1;
                    if cur_x == width {
                        cur_x = 0;  
                        cur_y += 1;
                    }
                }
            }
        }
        if let Some(ch) = self.get(end) {
            if *ch == '\n' {
                // Edge case: end is a newline
                cur_x = 0;
                cur_y += 1;
            }
        }
        (cur_y, cur_x)
    }
}
