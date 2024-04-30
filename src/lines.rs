extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;
use std::fs;
use std::io::Read;
use std::cmp::{min, max};

pub struct LineArena {
    arena: Arena<Line>,
    head: Option<Index>,
    length: usize,
    width: usize,
    line_count: usize
}

impl LineArena {
    pub fn new(width: usize) -> LineArena {
        LineArena {
            arena: Arena::new(),
            head: None,
            length: 0,
            width: width,
            line_count: 0
        }
    }

    pub fn from_file(mut file: fs::File, width: usize) -> LineArena {
        // Constructor data
        let mut arena = Arena::<Line>::new();
        let head = Line::new();
        let head = arena.insert(head);
        let mut pointer = head;
        let mut length = 1;

        // Read file all into a buffer
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        // Iterate over each character
        for ch in buffer.chars() {
            // Newline -> add new Line to end of list
            if ch == '\n' {
                let new_line = Line::new();
                let new_line = arena.insert(new_line);
                arena[new_line].prevline = Some(pointer);
                arena[pointer].nextline = Some(new_line);
                pointer = new_line;
                length += 1;
            } else {
                arena[pointer].push_char(ch);
            }
        }
        let line_arena = LineArena {
            arena: arena,
            head: Some(head),
            length: length,
            width: width,
            line_count: 0 // Temporary value; update later
        };
        line_arena.get_line_count();
        line_arena
    }

    pub fn add_empty_line(&mut self, idx: usize) -> Index {
        // Insert an empty line at absolute index
        let line = Line::new();
        self.insert(line, idx)
    }

    pub fn add_empty_line_after(&mut self, index: Index) -> Index {
        // Insert an empty line after index

        let line = Line::new();
        self.insert_after(line, index)
    }

    pub fn add_empty_line_before(&mut self, index: Index) -> Index {
        // Insert an empty line before index
        let line = Line::new();
        self.insert_before(line, index)
    }

    pub fn insert(&mut self, line: Line, idx: usize) -> Index {
        // Inserts line at given absolute index
        // e.g. [1, 2, 4].insert(3, 2) => [1, 2, 3, 4]

        if idx > self.length || idx < 0 {
            panic!();
        }

        let arena = &mut self.arena;

        if idx == 0 {
            self.insert_at_head(line)
        } else {
            let ptr_index = self.seek(idx - 1);
            self.insert_after(line, ptr_index)
        }
    }

    pub fn insert_after(&mut self, line: Line, index: Index) -> Index {
        // Insert line after relative index

        let arena = &mut self.arena;
        let line = arena.insert(line);

        arena[line].nextline = arena[index].nextline.take();
        arena[index].nextline = Some(line);

        if let Some(line_index) = arena[index].nextline {
            arena[line].prevline = arena[line_index].prevline.take();
            arena[line_index].prevline = Some(line_index);
        }
        self.length += 1;
        line
    }

    pub fn insert_before(&mut self, line: Line, index: Index) -> Index {
        // Insert line before relative index

        let arena = &mut self.arena;
        let prevline = arena[index].prevline;
        match prevline {
            Some(before) => self.insert_after(line, before),
            None => self.insert_at_head(line) // index == self.head
        }
    }

    pub fn insert_at_head(&mut self, line: Line) -> Index {
        // Insert line at head

        let arena = &mut self.arena;
        let line = arena.insert(line);

        // Set line.nextline and head.prevline
        let head_index = self.head.take();
        if let Some(index) = head_index {
            // head actually exists -> set its prevline to new line
            arena[index].prevline = Some(line);
        }
        arena[line].nextline = head_index;
        self.head = Some(line);

        self.length += 1;
        line
    }

    pub fn append(&mut self, line: Line) -> Index {
        self.insert(line, self.length)
    }

    pub fn add_empty_line_head(&mut self) -> Index {
        // Insert empty line at beginning
        return self.add_empty_line(0);
    }

    pub fn add_empty_line_tail(&mut self) -> Index {
        // Insert empty line at end
        return self.add_empty_line(self.length);
    }

    pub fn seek(&mut self, idx: usize) -> Index {
        // Retrieves index of line at idx

        if idx < 0 || idx >= self.length {
            panic!("idx out of range");
        }

        if idx == 0 {
            return self.head.unwrap();
        }

        let arena = &mut self.arena;
        let mut pointer = self.head;

        for i in 0..(idx - 1) {
            if let Some(ptr_index) = pointer {
                pointer = arena[ptr_index].nextline;
            }
        }
        pointer.unwrap()
    }

    pub fn get(&mut self, idx: usize) -> Option<&Line> {
        // Get reference to line at index
        if idx < 0 || idx >= self.length {
            panic!();
        }

        let mut pointer = self.head;
        for _ in 0..idx {
            if let Some(ptr_index) = pointer {
                pointer = self.arena[ptr_index].nextline;
            }
        }
        match pointer {
            Some(ptr_index) => Some(&self.arena[ptr_index]),
            None => None
        }
    }

    pub fn pop_index(&mut self, index: Index) {
        // Pops the line at relative index

        let arena = &mut self.arena;

        let prevline = arena[index].prevline.take();
        let nextline = arena[index].nextline.take();
        match prevline {
            Some(prev_index) => {
                arena[prev_index].nextline = nextline;
            }
            None => {}
        }
        match nextline {
            Some(next_index) => {
                arena[next_index].prevline = prevline;
            },
            None => {}
        }
        self.length -= 1;
    }

    pub fn pop(&mut self, idx: usize) -> Option<Index> {
        // Pops the line at index

        if idx == self.length && idx == 0 {
            return None;
        }

        if idx < 0 || idx >= self.length {
            panic!();
        }

        let index = self.seek(idx);
        self.pop_index(index);
        Some(index)
    }

    pub fn swap(&mut self, index1: Index, index2: Index) {
        let arena = &mut self.arena;
        if let (Some(line1), Some(line2)) = arena.get2_mut(index1, index2) {
            // Exchange links
            let line1_next = line1.nextline.take();
            let line1_prev = line1.prevline.take();
            let line2_next = line2.nextline.take(); 
            let line2_prev = line2.prevline.take();

            // Swap the links leading in
            if let Some(l1_next) = line1_next {
                arena[l1_next].prevline = Some(index2);
            }
            if let Some(l2_next) = line2_next {
                arena[l2_next].prevline = Some(index1);
            }
            if let Some(l1_prev) = line1_prev {
                arena[l1_prev].nextline = Some(index2);
            }
            if let Some(l2_prev) = line2_prev {
                arena[l2_prev].nextline = Some(index1);
            }

            // Swap links leading out
            if let (Some(line1), Some(line2)) = arena.get2_mut(index1, index2) {
                line1.nextline = line2_next; 
                line1.prevline = line2_prev;
                line2.nextline = line1_next;
                line2.prevline = line1_prev;
            }

        } else {
            // Indices not found!
            panic!();
        }
    }

    pub fn split(&mut self, index: Index, split_point: usize) {
        // Splits line into [..split_point] and [split_point..]
        // Mutates this Line into [..split_point], and returns Line
        // with [split_point..]

        let arena = &mut self.arena;
        let line_length = arena[index].len();

        if split_point < 0 || split_point > line_length {
            panic!("split_point index out of range");
        }

        if split_point == 0 {
            // Hit enter at beginning -> newline behind
            self.add_empty_line_before(index);
        } else if split_point == line_length {
            // Hit enter at end -> newline in front
            self.add_empty_line_after(index);
        } else {
            // Split off the extra, create new Line, insert after
            let split_off = arena[index].content.split_off(split_point);
            let line = Line::new_from(split_off);
            self.insert_after(line, index);
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get_head(&self) -> Option<Index> {
        self.head
    }

    pub fn display_frame(&mut self, width: usize, height: usize) -> Vec<Vec<char>> {
        // Returns a vector of <= `height` Vec<char>s, each <= `width` wide
        // starting from head

        match self.head {
            Some(head) => self.display_frame_from(head, width, height),
            None => Vec::<Vec<char>>::new() // empty
        }
    }

    pub fn display_frame_from(&mut self, index: Index, width: usize, height: usize) -> Vec<Vec<char>> {
        // Returns a vector of <= `height` Vec<char>s, each <= `width wide
        // counted from `index` to end

        let mut buffer = Vec::<Vec<char>>::new();
        let mut pointer = Some(index);

        while buffer.len() < height {
            match pointer {
                Some(ptr_index) => {
                    let diff = height - buffer.len();
                    let mut line = &mut self.arena[ptr_index];
                    let n_lines = min(diff, line.height(width));
                    let mut slices = line.get_n_lines(width, n_lines);
                    buffer.append(&mut slices);
                    pointer = line.nextline;
                }, None => {
                    break;
                }
            }
        }
        buffer
    }

    pub fn export(&self) -> String {
        // Return a string formed from merging Lines
        let mut pointer = self.head;
        let mut export = String::new();
        while let Some(line) = pointer {
            if self.arena[line].prevline != None {
                // Only add newline for second iteration and beyond
                export.push('\n');
            }
            for c in self.arena[line].content.iter() {
                export.push(*c);
            }
            pointer = self.arena[line].nextline;
        }
        export
    }

    fn get_line_count(&self) -> usize {
        // Count number of display lines (a Line can span multiple display lines)

        let mut line_count: usize = 0;

        let mut pointer = self.head;
        while let Some(ptr_index) = pointer {
            line_count += self.arena[ptr_index].height(self.width);
            pointer = self.arena[ptr_index].nextline;
        }
        line_count
    }

    fn update_line_count(&mut self) {
        // Update line_count

        self.line_count = self.get_line_count();
    }
}

pub struct Line {
    prevline: Option<Index>,
    nextline: Option<Index>,
    content: Vec<char>
}

impl Line {
    pub fn new() -> Line {
        Line { prevline: None, nextline: None, content: Vec::new() }
    }

    pub fn new_from(content: Vec<char>) -> Line {
        Line { prevline: None, nextline: None, content: content }
    }

    pub fn push_char(&mut self, character: char) {
        // Push a char to the contents of line
        self.content.push(character);
    }

    pub fn height(&self, width: usize) -> usize {
        // Get the height of line if displayed
        1 + self.content.len() / width
    }

    pub fn len(&self) -> usize {
        // Get length of self.content
        self.content.len()
    }

    pub fn get_lines(&mut self, width: usize) -> Vec<Vec<char>> {
        // Returns a vector of vectors of slices, each corresponding to a line
        let mut slices = Vec::<Vec<char>>::new();
        let height = self.height(width);

        // Edge case
        if height == 1 {
            slices.push(self.content.clone());
            return slices;
        }

        // All full-length slices
        for i in 0..(height - 2) {
            let begin = i * width;
            let end = begin + width;
            slices.push(self.content[begin..end].to_vec());
        }

        // Tail slice
        let tail_begin = (height - 1) * width;
        slices.push(self.content[tail_begin..].to_vec());

        slices
    }

    pub fn get_n_lines(&mut self, width: usize, n: usize) -> Vec<Vec<char>> {
        // Returns a vector of n lines
        let mut slices = Vec::<Vec<char>>::new();
        let height = self.height(width);

        if height < n || n == 0 {
            panic!("{}", format!("n lines out of bounds (height: {}, n: {})", height, n));
        }

        if height == 1 {
            slices.push(self.content[..].to_vec());
            return slices;
        }

        for i in 0..(n - 2) {
            let begin = i * width;
            let end = begin + width;
            slices.push(self.content[begin..end].to_vec());
        }

        let tail_begin = (n - 1) * width;
        slices.push(self.content[tail_begin..].to_vec());
        slices
    }
}
