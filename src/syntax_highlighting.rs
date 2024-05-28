use regex::Regex;
use std::cmp::min;
use std::cmp::max;

pub struct ITreeNode {
    val: Paint,
    lnode: Option<Box<ITreeNode>>,
    rnode: Option<Box<ITreeNode>>,
    max: usize
}

pub struct SyntaxHighlight {
    regex: Regex,
    color: u64
}

pub struct HighlightRules {
    rules: Vec<SyntaxHighlight>
}

#[derive(Clone, Copy)]
pub struct Paint {
    region_left: usize,
    region_right: usize,
    color: u64
}

impl ITreeNode {
    pub fn new(paint: Paint) -> ITreeNode {
        let max_value = paint.region_right;
        ITreeNode { val: paint, lnode: None, rnode: None, max: max_value }
    }

    pub fn find_paints(&self, idx: usize) -> Vec<Paint> {
        // Finds all paints that intersect with an index
        let mut vec = Vec::<Paint>::new();
        self.find_paints_recursive(idx, &mut vec);
        vec
    }

    fn find_paints_recursive(&self, idx: usize, vec: &mut Vec<Paint>) {
        // Tail call optimized recursive find_points
        if self.val.region_left <= idx && idx < self.val.region_right {
            vec.push(self.val);
        }
        if let Some(ref left_node) = self.lnode {
            if left_node.max > idx {
                left_node.find_points_recursive(idx, vec);
                return;
            }
        }
        if let Some(ref right_node) = self.rnode {
            right_node.find_points_recursive(idx, vec);
        }
    }
}

impl SyntaxHighlight {
    pub fn new(regex: Regex, color: u64) -> SyntaxHighlight {
        SyntaxHighlight { regex: regex, color: color }
    }
}

impl HighlightRules {
    pub fn new(rules: Vec<SyntaxHighlight>) -> HighlightRules {
        HighlightRules { rules: rules }
    }

    pub fn highlight_region(&self, buffer: String, region_left: usize, region_right: usize) -> Vec<Paint> {
        // Returns a vector of paint regions, to be applied in order

        let mut paints = Vec::<Paint>::new();

        for highlight in &self.rules {
            let mut index = 0;
            while index < region_right {
                // Skip over the middle of unicode
                while !buffer.is_char_boundary(index) {
                    index += 1;
                }
                match highlight.regex.find(&buffer[index..]) {
                    Some(m) => {
                        if m.start() + index >= region_right { // Region is entirely after the window
                            break;
                        }
                        if m.end() + index <= region_left { // Region is entirely before the window
                            index += m.end() + 1;
                            continue;
                        }
                        let start = m.start() + index;
                        let end = m.end() + index;

                        paints.push(Paint { region_left: max(region_left, start), region_right: min(region_right, end), color: highlight.color });
                        index = end + 1;
                    },
                    None => { break; }
                }
            }
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
