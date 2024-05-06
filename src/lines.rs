extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;
use std::fs;
use std::mem;
use std::io::Read;
use std::collections::VecDeque;
use std::cmp::{min, max};

pub struct GapBuffer {
    gap_before: VecDeque<String>,
    gap_after: VecDeque<String>,
    buffer: String,
    pub width: usize
}

impl GapBuffer {
    pub fn new(width: usize) -> GapBuffer {
        GapBuffer { gap_before: VecDeque::<String>::new(), gap_after: VecDeque::<String>::new(), buffer: String::new(), width: width }
    }

    pub fn new_from_file(mut file: fs::File, width: usize) -> Option<GapBuffer> {
        // Construct new gap buffer filled with file. Set initial pointer at head.
        let mut gap_after = VecDeque::<String>::new();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        gap_after.push_back(String::new());

        for ch in buffer.chars() {
            if ch == '\n' {
                gap_after.push_back(String::new());
            } else {
                gap_after.back_mut()?.push(ch);
            }
        }

        let buffer = gap_after.pop_front()?;

        Some(GapBuffer { gap_before: VecDeque::<String>::new(), gap_after: gap_after, buffer: buffer, width: width })
    }

    pub fn insert_snippet(&mut self, snippet: String, pos: usize) {
        // Inserts the snippet (no newlines) to the current buffer at position

        if pos > self.buffer.len() {
            panic!("GapBuffer::insert_snippet(2) - index out of range");
        }
        self.buffer.insert_str(pos, &snippet);
    }

    pub fn delete(&mut self, pos: usize) {
        // Deletes character at position in buffer
        if pos >= self.buffer.len() {
            panic!("GapBuffer::delete(1) - index out of range");
        }

        self.buffer.remove(pos);
    }

    pub fn set_cursor(&mut self, pos: usize) {
        // Set the cursor position to given position
        if pos > self.gap_before.len() + self.gap_after.len() {
            // [0, self.gap_before.len()) + [self.gap_before.len()] + (self.gap_before.len(), self.gap_before.len() + self.gap_after.len()]
            panic!("GapBuffer::set_cursor(1) - index out of range");
        }

        if pos == self.gap_before.len() {
            // Already at this position; do nothing
            return;
        }

        let diff = pos as i32 - self.gap_before.len() as i32;
        self.move_cursor(diff);
    }

    pub fn move_cursor(&mut self, diff: i32) {
        // Move the cursor up or down by diff

        if diff == 0 {
            return;
        } else if diff < 0 {
            // Move cursor down
            for _ in 0..(-diff) {
                self.gap_after.push_front(mem::take(&mut self.buffer));
                if let Some(back) = self.gap_before.pop_back() {
                    self.buffer = back;
                } else {
                    panic!("GapBuffer::move_cursor(1) - index out of range");
                }
            }
        } else if diff > 0 {
            // Move cursor up
            for _ in 0..diff {
                self.gap_before.push_back(mem::take(&mut self.buffer));
                if let Some(front) = self.gap_after.pop_front() {
                    self.buffer = front;
                } else {
                    panic!("GapBuffer::move_cursor(1) - index out of range");
                }
            }
        }
    }

    pub fn get_line(&mut self, pos: usize) -> Option<&String> {
        // Gets the line at position
        if pos > self.gap_before.len() + self.gap_after.len() {
            panic!("GapBuffer::get_line(1) - index out of range");
        }

        if pos < self.gap_before.len() {
            self.gap_before.get(pos)
        } else if pos > self.gap_before.len() {
            self.gap_after.get(pos - self.gap_before.len() + 1)
        } else { // Get the buffer
            Some(&self.buffer)
        }
    }

    pub fn get_frame(&mut self, start_position: (usize, usize), height: usize) -> Option<Vec<String>> {
        // Gets a "frame" of Strings 'height' long

        let (start, start_height) = start_position;
        if start > self.gap_before.len() + self.gap_after.len() {
            panic!("GapBuffer::get_frame(2) - index out of range!");
        }

        let mut buffer = Vec::<String>::new();
        let initial_line = self.get_line(start)?.to_string();
        buffer.extend_from_slice(&GapBuffer::get_display_lines(initial_line, self.width));

        let mut index = start + 1;
        while buffer.len() < height {
            let line = self.get_line(start)?.to_string();
            if (line.len() / self.width) + 1 + buffer.len() > height {
                buffer.extend_from_slice(&GapBuffer::get_display_lines(line, self.width)[..height - buffer.len()]);
            } else {
                buffer.extend_from_slice(&GapBuffer::get_display_lines(line, self.width));
            }
            index += 1;
        }

        Some(buffer)
    }

    fn get_display_lines(line: String, width: usize) -> Vec<String> {
        // Splice a string into a vec of strings each with width <= width
        let mut buffer = Vec::<String>::new();
        let height = 1 + line.len() / width;
        for i in 0..(height - 1) { // iterates (height - 1) times
            buffer.push((&line[i*width..(i+1)*width]).to_string());
        }
        buffer.push((&line[(height-1)*width..]).to_string());
        buffer
    }

    pub fn export(&mut self) -> String {
        // export entire gap buffer as a single string
        let mut buffer = String::new();
        for string in self.gap_before.iter() {
            buffer.push_str(&string);
        }
        buffer.push_str(&self.buffer);
        for string in self.gap_after.iter() {
            buffer.push_str(&string);
        }
        buffer
    }
}

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

        // Read filee all into a buffer
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
        let mut line_arena = LineArena {
            arena: arena,
            head: Some(head),
            length: length,
            width: width,
            line_count: 0 // Temporary value; update later
        };
        line_arena.update_line_count();
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

        if idx > self.length {
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

    pub fn insert_block(&mut self, buffer: Vec<char>) -> (Index, Index) {
        // Inserts a whole buffer into line arena, and returns the first and last index
        let arena = &mut self.arena;

        let line = Line::new();
        let start = arena.insert(line);
        let mut end = start;
        self.length += 1;


        for ch in buffer.iter() {
            if *ch == '\n' { // New line
                let new_line = Line::new();
                let new_line = arena.insert(new_line);
                self.line_count += arena[end].height(self.width);
                arena[new_line].prevline = Some(end);
                arena[end].nextline = Some(new_line);
                end = new_line;
                self.length += 1;
            } else {
                arena[end].push_char(*ch);
            }
        }

        self.line_count += arena[end].height(self.width);

        (start, end)
    }

    pub fn insert_after(&mut self, line: Line, index: Index) -> Index {
        // Insert line after relative index

        let arena = &mut self.arena;
        let line = arena.insert(line);

        // Set links
        arena[line].nextline = arena[index].nextline.take();
        arena[index].nextline = Some(line);

        if let Some(nextline_index) = arena[line].nextline {
            arena[line].prevline = arena[nextline_index].prevline.take();
            arena[nextline_index].prevline = Some(line);
        } else {
            arena[line].prevline = Some(index);
        }

        // Update line_count and length
        self.line_count += arena[line].height(self.width);
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

        // Update length and line_count
        self.length += 1;
        self.line_count += arena[line].height(self.width);

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

        if idx >= self.length {
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

    pub fn get_mut(&mut self, index: Index) -> &mut Line {
        // Get mutable reference to Line with Index

        &mut self.arena[index]
    }

    pub fn get(&self, index: Index) -> &Line {
        // Get reference to Line with Index

        &self.arena[index]
    }

    pub fn get_idx(&mut self, idx: usize) -> Option<&Line> {
        // Get reference to line at index

        if idx >= self.length {
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

        // Unlink Line at index
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

        // Update length and line_count
        self.length -= 1;
        self.line_count -= arena[index].height(self.width);
    }

    pub fn pop(&mut self, idx: usize) -> Option<Index> {
        // Pops the line at index

        if idx == self.length && idx == 0 {
            return None;
        }

        if idx >= self.length {
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

    pub fn split(&mut self, index: Index, split_point: usize) -> Index {
        // Splits line into [..split_point] and [split_point..]
        // Mutates this Line into [..split_point], and returns Line
        // with [split_point..]

        let arena = &mut self.arena;
        let line_length = arena[index].len();

        if split_point > line_length {
            panic!("split_point index out of range");
        }

        if split_point == 0 && split_point != line_length {
            // Hit enter at beginning -> newline behind
            self.add_empty_line_before(index)
        } else if split_point == line_length {
            // Hit enter at end -> newline in front
            self.add_empty_line_after(index)
        } else {
            // Split off the extra, create new Line, insert after
            let current_height = arena[index].height(self.width);
            let split_off = arena[index].content.split_off(split_point);
            let new_height = arena[index].height(self.width);
            let line = Line::new_from(split_off);
            // Update line_count so that the adjusted line_count after insert_after() is correct
            self.line_count -= current_height - new_height; // Guarantees (current - new) >= 0
            self.insert_after(line, index)
        }
    }

    pub fn merge(&mut self, index: Index) {
        // Merges Line at index with Line after it.
        // Does nothing if nextline is None

        let arena = &mut self.arena;
        if let Some(nextline) = arena[index].nextline {
            // Move the contents of the next line into current line,
            // and then pop the next line

            let content = arena[nextline].content.clone();
            arena[index].extend(&content);
            self.pop_index(nextline);
        }
    }

    pub fn link(&mut self, left_index: Index, right_index: Index) {
        // Link the left and right indices
        // IMPORTANT: this breaks left_index.nextline and right_index.prevline links!

        let arena = &mut self.arena;

        arena[left_index].nextline = Some(right_index);
        arena[right_index].prevline = Some(left_index);
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
    pub prevline: Option<Index>,
    pub nextline: Option<Index>,
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

    pub fn insert_char(&mut self, pos: usize, character: char) {
        // Inserts char so that pos is the new index for the character
        self.content.insert(pos, character);
    }

    pub fn pop_char(&mut self, pos: usize) -> char {
        self.content.remove(pos)
    }

    pub fn split_off(&mut self, pos: usize) -> Vec<char> {
        // Calls split_off on own content
        self.content.split_off(pos)
    }

    pub fn extend(&mut self, target: &Vec<char>) {
        // Merges target content into own content
        self.content.extend(target);
    }

    pub fn height(&self, width: usize) -> usize {
        // Get the height of line if displayed
        1 + self.content.len() / width
    }

    pub fn len(&self) -> usize {
        // Get length of self.content
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.len() == 0
    }

    pub fn tail_len(&self, width: usize) -> usize {
        // Get length of the "tail" - the last bit of overflow line
        self.content.len() - (self.content.len() / width) * width
    }

    pub fn to_string(&self) -> String {
        String::from_iter(&self.content)
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
        for i in 0..(height - 1) {
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

        if n == 1 {
            slices.push(self.content[..width].to_vec());
            return slices;
        }

        for i in 0..(n - 1) {
            let begin = i * width;
            let end = begin + width;
            slices.push(self.content[begin..end].to_vec());
        }

        let tail_begin = (n - 1) * width;
        slices.push(self.content[tail_begin..].to_vec());
        slices
    }
}
