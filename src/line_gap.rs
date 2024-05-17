use crate::gap_buffer;
use crate::gap_buffer::GapBuffer;
use core::slice::Iter;
use std::iter;

pub struct LineBuffer {
    content: GapBuffer<Line>,
    width: usize, // Width of the physical window
    token_pos: usize
}

pub struct Line {
    content: GapBuffer<Token>
}

pub struct Token {
    content: Vec<char>
}

impl LineBuffer {
    pub fn new(width: usize) -> LineBuffer {
        LineBuffer { content: GapBuffer::<Line>::new(), width: width, token_pos: 0 }
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

    pub fn get(&mut self, idx: usize) -> Option<&Line> {
        self.content.get(idx)
    }

    pub fn iter_range<'a>(&'a self, start: usize, stop: usize) -> gap_buffer::GapBufferIntoIter<'a, Line> {
        self.content.iter_range(start, stop)
    }
}

impl Line {
    pub fn new() -> Line{
        Line { content: GapBuffer::<Token>::new() }
    }

    pub fn export(&mut self) -> String {
        let mut buffer = String::new();
        for token in self.content.into_iter() {
            buffer.push_str(&token.export());
        }
        /*
        for token in self.content.gap_before.iter_mut() {
            buffer.push_str(&token.export());
        }
        for token in self.content.gap_after.iter_mut() {
            buffer.push_str(&token.export());
        }
        */
        buffer
    }

    pub fn export_buffer<'a, I>(&'a self) -> I where I: Iterator<Item=char> {
        //self.content.into_iter().map(|tok| tok.export_buffer()).join()
        self.content.into_iter().map(|tok| tok.export_buffer()).reduce(|acc, iter| acc.chain(iter))
        /*
        let iter = self.content.into_iter();
        if let Some(tok) = iter.next() {
            let mut buffer = tok.export_buffer();
            for tok in iter {
                buffer = buffer.chain(tok.export_buffer());
            }
            buffer
        } else {
            Vec::<char>::new().iter()
        }
        */
    }
}

impl Token {
    pub fn new() -> Token {
        Token { content: Vec::<char>::new() }
    }

    pub fn export(&self) -> String {
        self.content.iter().collect()
    }
    pub fn export_buffer<'a>(&'a self) -> Iter<'a, char> {
        self.content.iter()
    }
}
