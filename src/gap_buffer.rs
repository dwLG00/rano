use std::fs;
use std::io::Read;

struct GapBuffer {
    buffer: Vec<char>,
    gap_position: usize,
    gap_size: usize,
    next_alloc_gap_size: usize
}

impl GapBuffer {
    pub fn new_from_file(mut file: fs::File, gap_size: usize) -> Gap Buffer {
        // Read a file into a GapBuffer, with a certain starting gap size and
        // with the gap at the very beginning
        let mut buffer: Vec::<char>::new();
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

    pub fn get(&self, idx: usize) -> &char {
        // Get the character at index, not counting the gap
        assert!(idx < self.len());

        if idx < self.gap_position {
            &self.buffer[idx]
        } else {
            &self.buffer[idx + self.gap_size]
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
        let new = ['\0'; self.next_alloc_gap_size];
        self.buffer.splice(self.gap_pos..self.gap_pos, new);

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
        self.gap_pos += 1;
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
}
