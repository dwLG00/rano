extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;

pub struct LineArena {
    arena: Arena<Line>,
    head: Option<Index>,
    length: usize,
}

impl LineArena {
    pub fn new() -> LineArena {
        LineArena{arena: Arena::new(), head: None, length: 0}
    }

    pub fn add_empty_line(&mut self, idx: usize) -> Index{
        // Insert an empty line at index

        let mut arena = &mut self.arena;

        let line = Line::new();
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
        return line;
    }

    pub fn add_empty_line_head(&mut self) -> Index {
        // Insert empty line at beginning
        return self.add_empty_line(0);
    }

    pub fn add_empty_line_tail(&mut self) -> Index {
        // Insert empty line at end
        return self.add_empty_line(self.length - 1);
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

    pub fn len(&self) -> usize {
        return self.length;
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
}
