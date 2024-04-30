extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::Path;
mod lines;

fn open_file() -> lines::LineArena {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("Requires filepath argument");
    }

    let reader = fs::File::open(Path::new(&args[1]));
    let mut file = reader.ok().expect("Unable to open file");
    lines::LineArena::from_file(file)
}

fn main() {
    let line_arena = open_file();
    initscr();
    raw();
    keypad(ncurses::stdscr(), true);
    noecho();

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    /*
    addstr("Enter a character: ").unwrap();

    let ch = getch();
    if ch == KEY_F(1) {
        addstr("\nF1 pressed").unwrap();
    }

    refresh();
    getch();
    */
    endwin();
}
