use std::collections::VecDeque;

pub struct GapBuffer<T> {
    pub gap_before: VecDeque<T>,
    pub gap_after: VecDeque<T>,
}

impl<T> GapBuffer<T> {
    pub fn new() -> GapBuffer<T> {
        GapBuffer { gap_before: VecDeque::<T>::new(), gap_after: VecDeque::<T>::new() }
    }

    pub fn shift_l(&mut self) {
        // Shifts the contents of gap_after into gap_before
        if let Some(after) = self.gap_after.pop_front() {
            self.gap_before.push_back(after);
        } else {
            panic!();
        }
    }

    pub fn shift_r(&mut self) {
        if let Some(before) = self.gap_before.pop_back() {
            self.gap_after.push_front(before);
        } else {
            panic!();
        }
    }

    pub fn shift_ln(&mut self, n: usize) {
        for _ in 0..n {
            self.shift_l();
        }
    }

    pub fn shift_rn(&mut self, n: usize) {
        for _ in 0..n {
            self.shift_r();
        }
    }

    pub fn pop_r(&mut self) -> Option<T> {
        self.gap_after.pop_front()
    }

    pub fn pop_l(&mut self) -> Option<T> {
        self.gap_before.pop_back()
    }

    pub fn get(&mut self, index: usize) -> Option<&T> {
        if index < self.gap_before.len() {
            self.gap_before.get(index)
        } else if index < self.gap_before.len() + self.gap_after.len() {
            self.gap_after.get(index - self.gap_before.len())
        } else {
            None
        }
    }

    pub fn push_r(&mut self, item: T) {
        self.gap_after.push_front(item);
    }

    pub fn push_l(&mut self, item: T) {
        self.gap_before.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.gap_before.len() + self.gap_after.len()
    }

    pub fn iter_range<'a>(&'a self, start: usize, stop: usize) -> GapBufferIntoIter<'a, T> {
        GapBufferIntoIter { gap_buffer: self, counter: start, stop: stop }
    }
}

// Iterators

pub struct GapBufferIntoIter<'a, T> {
    gap_buffer: &'a GapBuffer<T>,
    counter: usize,
    stop: usize
}

impl<'a, T> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;
    type IntoIter = GapBufferIntoIter<'a, T>;

    fn into_iter(self) -> GapBufferIntoIter<'a, T> {
        GapBufferIntoIter { gap_buffer: self, counter: 0, stop: self.len() }
    }
}

impl<'a, T> Iterator for GapBufferIntoIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.counter;
        self.counter += 1;
        if c == self.stop {
            None
        } else if c < self.gap_buffer.gap_before.len() {
            self.gap_buffer.gap_before.get(c)
        } else {
            self.gap_buffer.gap_after.get(c - self.gap_buffer.gap_before.len())
        }
    }
}

// Indexing

/*
impl<'a, T> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        
    }
}
*/
