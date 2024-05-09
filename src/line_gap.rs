use crate::gap_buffer;
use std::iter;

pub struct LineBuffer {
    content: gap_buffer::GapBuffer<Line>,
    width: usize, // Width of the physical window
    token_pos: usize
}

pub struct Line {
    content: gap_buffer::GapBuffer<Token>
}

pub struct Token {
    content: Vec<char>
}

impl LineBuffer {
    pub fn new(width: usize) -> LineBuffer {
        LineBuffer { content: gap_buffer::GapBuffer::<Line>::new(), width: width, token_pos: 0 }
    }

    pub fn export(&mut self) -> String {
        let mut buffer = String::new();
        for line in self.content.gap_before.iter_mut() {
            buffer.push_str(&line.export());
            buffer.push('\n');
        }
        for line in self.content.gap_after.iter_mut() {
            buffer.push_str(&line.export());
            buffer.push('\n');
        }
        buffer
    }

}

impl Line {
    pub fn new() -> Line{
        Line { content: gap_buffer::GapBuffer::<Token>::new() }
    }

    pub fn export(&mut self) -> String {
        let mut buffer = String::new();
        for token in self.content.gap_before.iter_mut() {
            buffer.push_str(&token.export());
        }
        for token in self.content.gap_after.iter_mut() {
            buffer.push_str(&token.export());
        }
        buffer
    }
}

impl Token {
    pub fn new() -> Token {
        Token { content: Vec::<char>::new() }
    }

    pub fn export(&mut self) -> String {
        self.content.gap_before.iter().chain(self.content.gap_after.iter()).collect()
    }
    pub fn export_buffer(&mut self) -> Vec<char> {
        self.content.gap_before.iter().chain(self.content.gap_after.iter()).collect();
    }
}
