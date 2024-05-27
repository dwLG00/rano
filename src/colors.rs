extern crate ncurses;
use ncurses::*;

// Color constants
pub static CP_HIGHLIGHT: i16 = 1;
pub static CP_SEARCH: i16 = 2;
pub static CP_SYNTAX_DEBUG: i16 = 3;

pub fn init_colors() {
    // Initialize Colors
    use_default_colors();
    start_color();
    init_pair(CP_HIGHLIGHT, COLOR_BLACK, COLOR_WHITE);
    init_pair(CP_SEARCH, COLOR_BLACK, COLOR_YELLOW);
    init_pair(CP_SYNTAX_DEBUG, COLOR_RED, COLOR_BLACK);
}
