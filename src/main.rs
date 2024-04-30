extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::Path;
mod lines;
mod nc;

fn open_file() -> nc::Editor {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("Requires filepath argument");
    }

    let reader = fs::File::open(Path::new(&args[1]));
    let mut file = reader.ok().expect("Unable to open file");
    nc::Editor::from_file(file)
}

fn main() {
    let mut editor = open_file();
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();

    editor.display_at_cursor();

    /*
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let buffer = line_arena.display_frame(max_x as usize, max_y as usize);
    for line in buffer.iter() {
        for ch in line.iter() {
            addch(*ch as chtype);
        }
        let mut cur_x = 0;
        let mut cur_y = 0;
        getyx(stdscr(), &mut cur_y, &mut cur_x);
        mv(cur_y + 1, 0);
    }
    */

    refresh();
    getch();

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
