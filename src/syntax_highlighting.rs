extern crate intervaltree;
use regex::Regex;
use std::cmp::min;
use std::cmp::max;
use intervaltree::{IntervalTree, Element};

// Paint Interval Tree
pub type PaintTree = IntervalTree<usize, (u64, usize)>;

impl From<Paint> for Element<usize, (u64, usize)> {
    fn from(paint: Paint) -> Self {
        Element { range: std::ops::Range { start: paint.region_left, end: paint.region_right }, value: (paint.color, paint.priority) }
    }
}

pub fn paint_tree_from_vecs(vec: Vec<Paint>) -> PaintTree {
    PaintTree::from_iter(vec)
}

pub fn paint_tree_get_color(paint_tree: &PaintTree, point: usize) -> Option<u64> {
    // Gets the color of the lowest-priority element in paint tree
    let hits = paint_tree.query_point(point);
    let mut matched_color: u64 = 0;
    let mut lowest_priority: usize = usize::MAX;
    for hit in hits {
        let (color, priority) = hit.value;
        if priority < lowest_priority {
            matched_color = color;
            lowest_priority = priority;
        }
    }
    if matched_color != 0 {
        Some(matched_color)
    } else {
        None
    }
}

// ----------------------

#[derive(Clone, Debug)]
pub struct SyntaxHighlight {
    regex: Regex,
    color: u64
}

#[derive(Clone, Debug)]
pub struct HighlightRules {
    rules: Vec<SyntaxHighlight>
}

#[derive(Clone, Copy)]
pub struct Paint {
    region_left: usize,
    region_right: usize,
    color: u64,
    priority: usize
}

impl SyntaxHighlight {
    pub fn new(regex: Regex, color: u64) -> SyntaxHighlight {
        SyntaxHighlight { regex: regex, color: color }
    }
}

// Basic impl
impl HighlightRules {
    pub fn new(rules: Vec<SyntaxHighlight>) -> HighlightRules {
        HighlightRules { rules: rules }
    }

    pub fn highlight_region(&self, buffer: String, region_left: usize, region_right: usize) -> Vec<Paint> {
        // Returns a vector of paint regions, to be applied in order

        let mut paints = Vec::<Paint>::new();
        let mut priority: usize = 0;
        let offsets = generate_offsets(&buffer);

        for highlight in &self.rules {
            let mut index = 0;
            let mut prev_index = 0;
            //let mut offset: usize = 0;
            let mut itcount = 0;
            while index < region_right {
                // Set the previous index
                prev_index = index;
                // Skip over the middle of unicode
                while !buffer.is_char_boundary(index) {
                    index += 1;
                }
                //if skip_over > 0 { panic!() };
                match highlight.regex.find(&buffer[index..]) {
                    Some(m) => {
                        if m.start() + index >= region_right { // Region is entirely after the window
                            break;
                        }
                        if m.end() + index <= region_left { // Region is entirely before the window
                            index += m.end() + 1;
                            //offset += get_offset_in_region(&buffer, prev_index, index);
                            continue;
                        }

                        let start_index = m.start() + index;
                        let end_index = m.end() + index;

                        // Calculate the offset before
                        //offset += get_offset_in_region(&buffer, prev_index, start_index);
                        let start = start_index - offsets[start_index];

                        // Calculate offset during
                        //offset += get_offset_in_region(&buffer, start_index, end_index);
                        let end = end_index - offsets[end_index];
                        /*
                        for i in m.start()..m.end() {
                            if !buffer.is_char_boundary(i + index) {
                                unicode_adjust += 1;
                            }
                        }
                        */

                        //if unicode_adjust > 0 { panic!(); }

                        //let start = m.start() + index - offset;
                        //let end = m.end() + index - offset - unicode_adjust;

                        //offset += unicode_adjust;

                        index = end_index + 1;

                        // Calculate offset after
                        //offset += get_offset_in_region(&buffer, end_index, index);

                        paints.push(Paint { region_left: max(region_left, start), region_right: min(region_right, end), color: highlight.color, priority: priority });
                    },
                    None => { break; }
                }
            }
            priority += 1;
        }
        paints
    }
}

impl Paint {
    pub fn find_match(paints: &Vec<Paint>, idx: usize) -> Option<u64> {
        // Finds the first paint in the vector s.t. index is within the
        // bounds of the paint region, and returns the color (or None if
        // there are no matches)

        for paint in paints {
            if paint.region_left <= idx && idx < paint.region_right {
                return Some(paint.color);
            }
        }
        None
    }
}

fn get_offset_in_region(buffer: &String, start_index: usize, end_index: usize) -> usize {
    // Gets the byte offset of region in buffer
    let mut offset = 0;
    for i in start_index..end_index {
        if !buffer.is_char_boundary(i) {
            offset += 1;
        }
    }
    offset
}

fn generate_offsets(buffer: &String) -> Vec<usize> {
    // Generates a vector of offset values

    let mut offsets = Vec::<usize>::new();
    let mut offset_count: usize = 0;
 
   for i in 0..buffer.len() {
        if !buffer.is_char_boundary(i) {
            offset_count += 1;
        }
        offsets.push(offset_count);
    }
    offsets
}
