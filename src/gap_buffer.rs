use std::fs;
use std::io::Read;

pub struct GapBuffer {
    pub buffer: Vec<char>,
    pub gap_position: usize,
    pub current_line: usize,
    pub n_lines: usize,
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

        let mut n_lines = 0;

        for _ in 0..gap_size {
            // Push null char for the gap buffer
            buffer.push('\0');
        }

        for ch in buffer_str.chars() {
            buffer.push(ch);
            if ch == '\n' {
                n_lines += 1;
            }
        }

        GapBuffer {
            buffer: buffer,
            gap_position: 0,
            current_line: 0,
            n_lines: n_lines,
            gap_size: gap_size,
            next_alloc_gap_size: gap_size * 2
        }
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
        assert!(new_pos <= self.len());

        if new_pos > self.gap_position {
            for i in self.gap_position..new_pos {
                let ch = self.buffer[i + self.gap_size];
                self.buffer[i + self.gap_size] = self.buffer[i];
                self.buffer[i] = ch;
                if ch == '\n' {
                    // We need to check the char after the gap,
                    // because ch is null
                    self.current_line += 1;
                }
            }
        } else if new_pos < self.gap_position {
            for i in (new_pos..self.gap_position).rev() {
                // Iterate from gap_position down to new_pos
                let ch = self.buffer[i];
                self.buffer[i] = self.buffer[i + self.gap_size];
                self.buffer[i + self.gap_size] = ch;
                if ch == '\n' {
                    self.current_line -= 1;
                }
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
        if ch == '\n' {
            self.current_line += 1;
            self.n_lines += 1;
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        // Removes the character behind gap position
        if self.gap_position == 0 {
            //panic!("Tried to delete character behind gap_position, but gap_position is 0!");
            return None;
        }

        let ch = self.buffer[self.gap_position - 1];

        if ch == '\n' {
            self.current_line -= 1;
            self.n_lines -= 1;
        }

        self.buffer[self.gap_position - 1] = '\0';
        self.gap_position -= 1;
        self.gap_size += 1;
        Some(ch)
    }

    pub fn cut(&mut self, left: usize, right: usize, moveto: usize) -> Vec<char> {
        // Cuts out the portion of the buffer given, and
        // moves the cursor to the given position
        assert!(left <= right);
        assert!(left <= self.len());
        assert!(right <= self.len());

        // Don't count the very last selectable character
        let l = if left == self.len() { self.len() - 1 } else { left };
        let r = if right == self.len() { self.len() - 1 } else { right };

        let mut buffer = Vec::<char>::new();

        for i in l..=r {
            self.move_gap(l + 1);
            match self.pop() {
                Some(c) => { buffer.push(c); },
                None => { return buffer; }
            }
        }
        self.move_gap(moveto);
        buffer
    }

    pub fn copy(&self, left: usize, right: usize) -> Vec<char> {
        // Copies out the portion of the buffer given
        assert!(left <= right);
        assert!(left <= self.len());
        assert!(right <= self.len());

        // Don't count the very last selectable character
        let l = if left == self.len() { self.len() - 1 } else { left };
        let r = if right == self.len() { self.len() - 1 } else { right };

        let mut buffer = Vec::<char>::new();

        for i in l..=r {
            match self.get(i) {
                Some(c) => { buffer.push(*c) },
                None => { return buffer; }
            }
        }
        buffer

    }

    pub fn insert_buffer(&mut self, buffer: &Vec<char>) {
        // Inserts the char buffer into the gap buffer
        // at the current cursor's location

        for ch in buffer.iter() {
            self.insert(*ch);
        }
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

    pub fn get_right_edge(&self, start: usize) -> usize {
        // Gets the next newline (could be out of range)
        let mut pointer = start;
        while pointer < self.len() {
            if let Some(ch) = self.get(pointer) {
                if *ch == '\n' {
                    break;
                }
            }
            pointer += 1;
        }
        pointer
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
            let left_edge = self.get_left_edge(pointer);
            let line_height = 1 + (pointer - left_edge) / width;
            y_count += line_height;
            if left_edge == 0 && y_count >= n_lines {
                // We would break out of the loop at this stage
                // and add 1 back, so start from the top
                pointer = 0;

            } else if left_edge == 0 {
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

    pub fn get_next_display_line_head(&self, start: usize, width: usize) -> Option<usize> {
        // Gets the beginning of next display line, or None if doesn't exist
        let right_edge = self.get_right_edge(start);
        if right_edge == self.len() || right_edge + 1 == self.len() {
            // We're already on the last line
            return None;
        }

        if right_edge - start > width {
            // The head of the next line is > 1 display line away
            return Some(start + width);
        } else {
            // Left edge of the next line is right after the right_edge
            return Some(right_edge + 1);
        }
        None
    }

    pub fn get_prev_display_line_head(&self, start: usize, width: usize) -> Option<usize> {
        // Gets the beginning of previous display line, or None if doesn't exist
        let left_edge = self.get_left_edge(start);
        if left_edge == 0 && start < width {
            // We're already at the very top line
            return None;
        } else if start - left_edge >= width {
            // We're on the 2nd+ display line of a single line
            return Some(start - width);
        } else {
            // Seek the previous line, get the very last display line head
            let pl_right_edge = left_edge - 1;
            let pl_left_edge = self.get_left_edge(pl_right_edge);
            let pl_full_lines = (pl_right_edge - pl_left_edge) / width;
            return Some(pl_left_edge + pl_full_lines * width); // Add length of the fully-filled display lines
        }
        None
        
    }

    pub fn count_yx(&self, start: usize, end: usize, width: usize) -> (usize, usize) {
        // Count the position of `end`, if `start` was at (0, 0)
        assert!(start <= self.len());
        assert!(end <= self.len());

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

    pub fn xpos(&self, idx: usize, width: usize) -> usize {
        // Get the x position of the index in the
        // display line in the viewport
        let left_edge = self.get_left_edge(idx);
        let line_len = (idx + 1 - left_edge);
        if line_len / width * width == line_len {
            // Line_len is divisible by width
            width - 1
        } else {
            line_len - (line_len / width) * width - 1 // Acts as finding the remainder
        }
    }

    pub fn seek_next_line(&self, width: usize) -> Option<(usize, bool)> {
        // Returns the index of the cursor one
        // display line down, along with whether
        // the new cursor position's x-position is
        // less than the previous line

        let xpos = self.xpos(self.gap_position, width);
        self.seek_next_line_with_xpos(width, xpos)
    }

    pub fn seek_prev_line(&self, width: usize) -> Option<(usize, bool)> {
        // Returns the index of the cursor one
        // display line up, along with whether
        // the new cursor position's x-position is
        // less than the next line

        let xpos = self.xpos(self.gap_position, width);
        self.seek_prev_line_with_xpos(width, xpos)
    }

    pub fn seek_next_line_with_xpos(&self, width: usize, xpos: usize) -> Option<(usize, bool)> {
        // Returns the index of the cursor one
        // display line down, along with whether
        // the new cursor position's x-position is
        // less than the previous line

        if self.gap_position == self.len() {
            return None;
        }

        let left_edge = self.get_left_edge(self.gap_position);
        let right_edge = self.get_right_edge(self.gap_position);
        // What xpos the cursor is at
        //let xpos = self.xpos(self.gap_position, width);

        if right_edge - self.gap_position > width && self.gap_position + width < self.len() {
            // next display line is the same actual line (and is in range)
            return Some((self.gap_position + width, false));
        } else if right_edge < self.len() - 1 {
            let nl_left_edge = right_edge + 1;
            if let nl_right_edge = self.get_right_edge(nl_left_edge) {
                // Next line is a hanging newline
                if nl_right_edge == nl_left_edge {
                    return Some((nl_left_edge, xpos != 0));
                } else if nl_right_edge - nl_left_edge <= xpos {
                    // Next display position is less than the starting position
                    return Some((nl_right_edge - 1, true));
                } else {
                    return Some((nl_left_edge + xpos, false));
                }
            }
        } else if xpos != 0 {
            // We're on the last line
            return Some((right_edge + 1, true));
        } else {
            return Some((right_edge + 1, false));
        }
        None
    }

    pub fn seek_prev_line_with_xpos(&self, width: usize, xpos: usize) -> Option<(usize, bool)> {
        // Returns the index of the cursor one
        // display line up, along with whether
        // the new cursor position's x-position is
        // less than the next line

        let left_edge = self.get_left_edge(self.gap_position);
        let right_edge = self.get_right_edge(self.gap_position);
        // What xpos the cursor is at

        if self.gap_position - left_edge > width {
            // previous display line is the same actual line
            return Some((self.gap_position - width, false));
        } else if left_edge != 0 {
            let pl_right_edge = left_edge - 1;
            let pl_left_edge = self.get_left_edge(pl_right_edge);
            let pl_height = 1 + (pl_right_edge - pl_left_edge) / width;
            let pl_xpos = (pl_right_edge - pl_left_edge) - (pl_height - 1) * width;

            if pl_right_edge == pl_left_edge {
                return Some((pl_right_edge, xpos != 0));
            } else if pl_xpos < xpos {
                return Some((pl_right_edge - 1, true));
            } else {
                return Some((pl_left_edge + (pl_height - 1) * width + xpos, false));
            }

            /*
            let pl_left_edge = right_edge + 1;
            if let nl_right_edge = self.get_right_edge(nl_left_edge) {
                if nl_right_edge <= xpos {
                    // Next display position is less than the starting position
                    return Some((nl_right_edge - 1, true));
                } else {
                    return Some((nl_left_edge + xpos, false));
                }
            }
            */
        }
        // We're on the last line
        None
    }

    pub fn export(&self) -> String {
        let mut first: String = self.buffer[..self.gap_position].iter().collect();
        let second: String = self.buffer[self.gap_position + self.gap_size..].iter().collect();
        first.push_str(&second);
        first
    }
}

