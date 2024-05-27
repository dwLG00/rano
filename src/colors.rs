extern crate ncurses;
use ncurses::*;

// Color constants
// System colors
pub static CP_HIGHLIGHT: i16 = 1;
pub static CP_SEARCH: i16 = 2;

// Syntax-highlighting colors
pub static CP_BLACK: i16 = 0x0010;
pub static CP_RED: i16 = 0x0011;
pub static CP_GREEN: i16 = 0x0012;
pub static CP_YELLOW: i16 = 0x0013;
pub static CP_BLUE: i16 = 0x0014;
pub static CP_MAGENTA: i16 = 0x0015;
pub static CP_CYAN: i16 = 0x0016;
pub static CP_WHITE: i16 = 0x0017;

// Initialize
pub fn init_colors() {
    // Initialize Colors
    use_default_colors();
    start_color();

    init_pair(CP_HIGHLIGHT, COLOR_BLACK, COLOR_WHITE);
    init_pair(CP_SEARCH, COLOR_BLACK, COLOR_YELLOW);

    init_pair(CP_BLACK, COLOR_BLACK, COLOR_BLACK);
    init_pair(CP_RED, COLOR_RED, COLOR_BLACK);
    init_pair(CP_GREEN, COLOR_GREEN, COLOR_BLACK);
    init_pair(CP_YELLOW, COLOR_YELLOW, COLOR_BLACK);
    init_pair(CP_BLUE, COLOR_BLUE, COLOR_BLACK);
    init_pair(CP_MAGENTA, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(CP_CYAN, COLOR_CYAN, COLOR_BLACK);
    init_pair(CP_WHITE, COLOR_WHITE, COLOR_BLACK);
}
