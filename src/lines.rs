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
        //LineArena{arena: Arena::new(), head: None, cursor: None, length: 0, cursor_pos: 0}
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
            /*
            // seek cursor to target index
            self.seek(idx);
            // reborrow self.arena (we used it once for self.seek)
            arena = &mut self.arena;

            if let Some(cursor_index) = self.cursor {
                // cursor <-> line mutual reference exchange
                arena[line].nextline = arena[cursor_index].nextline.take();
                arena[cursor_index].nextline = Some(line);

                // cursor.nextline <-> line mutual reference exchange
                if let Some(line_index) = arena[cursor_index].nextline {
                    arena[line].prevline = arena[line_index].prevline.take();
                    arena[line_index].prevline = Some(line_index);
                }
            }
            */
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

        /*
        if let Some(cursor) = self.cursor {
            if let Some(prev) = arena[cursor].prevline {
                arena[prev].nextline = arena[cursor].nextline;
            } else {
                // cursor is head; move head to the next value
                self.head = arena[cursor].nextline.take();
            }
            if let Some(next) = arena[cursor].nextline {
                arena[next].prevline = arena[cursor].prevline;
                self.cursor = arena[cursor].nextline.take();
                // self.cursor_pos stays the same
            } else {
                self.cursor = arena[cursor].prevline.take();
                if self.cursor_pos > 0 {
                    // Only decrement cursor_pos if it doens't take the implicit None
                    self.cursor_pos -= 1;
                }
            }
            
            arena[cursor].nextline = None;
            arena[cursor].prevline = None;
            // Move over the cursor reference
            Some(cursor)
        } else {
            None
        }
        */
    }

    pub fn len(&self) -> usize {
        return self.length;
    }

    /*
    fn seek(&mut self, idx: usize) {
        // Moves cursor to index

        // Out of bounds
        if idx < 0 || idx >= self.length {
            panic!();
        }

        // Cursor already at index
        if self.cursor_pos == idx {
            return;
        } else if self.cursor_pos < idx {
            // index closer to cursor than beginning
            while self.cursor_pos < idx {
                if let Some(cursor_index) = self.cursor {
                    self.cursor = self.arena[cursor_index].nextline;
                    self.cursor_pos += 1
                }
            }
        } else if self.cursor_pos < idx * 2 {
            // cursor > idx, but index still closer to cursor than beginning
            while self.cursor_pos > idx {
                if let Some(cursor_index) = self.cursor {
                    self.cursor = self.arena[cursor_index].prevline;
                    self.cursor_pos -= 1
                }
            }
        } else {
            // index closer to beginning than cursor
            self.cursor = self.head;
            self.cursor_pos = 0;
            while self.cursor_pos < idx {
                if let Some(cursor_index) = self.cursor {
                    self.cursor = self.arena[cursor_index].nextline;
                    self.cursor_pos += 1
                }
            }
        }
    }
    */
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
