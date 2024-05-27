use regex::Regex;
use std::cmp::min;
use std::cmp::max;

pub struct SyntaxHighlight {
    regex: Regex,
    color: u64
}

pub struct HighlightRules {
    rules: Vec<SyntaxHighlight>
}

pub struct Paint {
    region_left: usize,
    region_right: usize,
    color: u64
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
