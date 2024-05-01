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
    nc::Editor::from_file(file, stdscr())
}

fn main() {
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();

    let mut editor = open_file();

    editor.display_at_frame_cursor();
    editor.move_cursor_to();
    refresh();

    let mut ch = getch();
    while ch != KEY_F(1) {
        clear();
        match ch {
            KEY_DOWN => {
                editor.scroll_down(false);
            },
            KEY_UP => {
                editor.scroll_up(false);
            },
            KEY_RIGHT => {
                editor.scroll_right(false);
            },
            KEY_LEFT => {
                editor.scroll_left(false);
            },
            _ => {}
        }
        editor.display_at_frame_cursor();
        editor.move_cursor_to();
        refresh();
        ch = getch();
    }

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
