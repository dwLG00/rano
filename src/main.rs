extern crate ncurses;

use std::char;

mod lines;

fn main() {
    ncurses::initscr();
    ncurses::raw();
    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::noecho();

    ncurses::addstr("Enter a character: ").unwrap();

    let ch = ncurses::getch();
    if ch == ncurses::KEY_F(1) {
        ncurses::addstr("\nF1 pressed").unwrap();
    }

    ncurses::refresh();
    ncurses::getch();
    ncurses::endwin();

}
