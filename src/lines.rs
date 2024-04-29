extern crate generational_arena;
use generational_arena::Arena;
use generational_arena::Index;

pub struct LineArena {
    arena: Arena<Line>,
    head: Option<Index>,
    cursor: Option<Index>,
    length: usize,
    cursor_pos: usize
}

impl LineArena {
    pub fn new() -> LineArena {
        LineArena{arena: Arena::new(), head: None, cursor: None, length: 0, cursor_pos: 0}
    }

    pub fn add_empty_line(&mut self, idx: usize) -> Index{
        // Insert an empty line at index

        let arena = &mut self.arena;

        let line = Line::new();
        let line = arena.insert(line);

        if idx > self.length {
            panic!();
        } else if idx == 0 {
            let head_index = self.head.take();

            if let Some(index) = head_index {
                arena[index].prevline = Some(line);
            }
            arena[line].nextline = head_index;
            self.head = Some(line);
        } else {
            if idx > self.cursor_pos {
                while self.cursor_pos + 1 < idx {
                    if let Some(cursor_index) = self.cursor {
                        self.cursor = arena[cursor_index].nextline;
                        self.cursor_pos += 1;
                    }
                }
            } else if self.cursor_pos < idx * 2 {
                while self.cursor_pos > idx {
                    if let Some(cursor_index) = self.cursor {
                        self.cursor = arena[cursor_index].prevline;
                        self.cursor_pos -= 1;
                    }
                }
            } else {
                self.cursor_pos = 0;
                self.cursor = self.head;
                while self.cursor_pos + 1 < idx {
                    if let Some(cursor_index) = self.cursor {
                        self.cursor = arena[cursor_index].nextline;
                        self.cursor_pos += 1;
                    }
                }
            }

            if let Some(cursor_index) = self.cursor {
                arena[line].nextline = arena[cursor_index].nextline.take();
                arena[cursor_index].nextline = Some(line);

                if let Some(line_index) = arena[cursor_index].nextline {
                    arena[line].prevline = arena[line_index].prevline.take();
                    arena[line_index].prevline = Some(line_index);
                }
            }
        }
        self.length += 1;
        return line;
    }

    pub fn add_empty_line_head(&mut self) -> Index {
        return self.add_empty_line(0);
    }

    pub fn add_empty_line_tail(&mut self) -> Index {
        return self.add_empty_line(self.length - 1);
    }
}

pub struct Line {
    prevline: Option<Index>,
    nextline: Option<Index>,
    content: Vec<char>
}

impl Line {
    pub fn new() -> Line {
        Line{ prevline: None, nextline: None, content: Vec::new() }
    }
}
