extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::Path;
mod lines;

fn open_file() -> fs::File {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!();
    }

    let reader = fs::File::open(Path::new(&args[1]));
    reader.ok().expect("Unable to open file")
}

fn main() {
    initscr();
    raw();
    keypad(ncurses::stdscr(), true);
    noecho();

    addstr("Enter a character: ").unwrap();

    let ch = getch();
    if ch == KEY_F(1) {
        addstr("\nF1 pressed").unwrap();
    }

    refresh();
    getch();
    endwin();

}
