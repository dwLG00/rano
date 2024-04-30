extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;
use std::fs;
use std::io::Read;

pub struct LineArena {
    arena: Arena<Line>,
    head: Option<Index>,
    length: usize,
}

impl LineArena {
    pub fn new() -> LineArena {
        LineArena{arena: Arena::new(), head: None, length: 0}
    }

    pub fn from_file(mut file: fs::File) -> LineArena {
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
        LineArena{arena: arena, head: Some(head), length: length}
    }

    pub fn add_empty_line(&mut self, idx: usize) -> Index{
        // Insert an empty line at index

        let line = Line::new();
        self.insert(line, idx)
    }

    pub fn insert(&mut self, line: Line, idx: usize) -> Index {
        // Inserts line at given position

        let arena = &mut self.arena;
        let line = arena.insert(line);

        if idx > self.length || idx < 0 {
            // index out of range
            panic!();
        } else if idx == 0 {
            // add line to head position
            let head_index = self.head.take();

            if let Some(index) = head_index {
                // head actually exists -> set its prevline to new line
                arena[index].prevline = Some(line);
            }
            arena[line].nextline = head_index;
            self.head = Some(line);
        } else {
            let mut pointer = self.head;
            for _ in 0..(idx - 1) {
                if let Some(ptr_index) = pointer {
                    pointer = arena[ptr_index].nextline;
                }
            }
            if let Some(ptr_index) = pointer {
                arena[line].nextline = arena[ptr_index].nextline.take();
                arena[ptr_index].nextline = Some(line);

                if let Some(line_index) = arena[ptr_index].nextline {
                    arena[line].prevline = arena[line_index].prevline.take();
                    arena[line_index].prevline = Some(line_index);
                }
            }
        }
        self.length += 1;
        line
    }

    pub fn add_empty_line_head(&mut self) -> Index {
        // Insert empty line at beginning
        return self.add_empty_line(0);
    }

    pub fn add_empty_line_tail(&mut self) -> Index {
        // Insert empty line at end
        return self.add_empty_line(self.length - 1);
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

    pub fn pop(&mut self, idx: usize) -> Option<Index> {
        // Pops the line at index

        if idx < 0 || idx >= self.length {
            panic!();
        }

        //self.seek(idx);
        let arena = &mut self.arena;

        if idx == 0 {
            if let Some(head) = self.head {
                let next = arena[head].nextline.take();
                self.head = next;
                return Some(head);
            } else {
                return None;
            }
        } else {
            let mut pointer = self.head;
            for _ in 0..idx {
                if let Some(ptr_index) = pointer {
                    pointer = arena[ptr_index].nextline;
                }
            }
            if let Some(ptr_index) = pointer {
                let prevline = arena[ptr_index].prevline.take();
                let nextline = arena[ptr_index].nextline.take();
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
                return Some(ptr_index);
            } else {
                return None;
            }
        }
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

    pub fn len(&self) -> usize {
        return self.length;
    }

    pub fn export(&self) -> String {
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
        self.content.push(character)
    }
}
