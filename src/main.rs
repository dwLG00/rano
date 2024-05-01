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

//#[cfg(feature = "wide")]
fn main() {
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();

    let mut editor = open_file();

    editor.display_at_frame_cursor();
    editor.move_cursor_to();
    refresh();

    let mut ch = wget_wch(stdscr());
    while true {
        clear();
        match ch {
            Some(WchResult::KeyCode(KEY_DOWN)) => {
                editor.scroll_down(false);
            },
            Some(WchResult::KeyCode(KEY_UP)) => {
                editor.scroll_up(false);
            },
            Some(WchResult::KeyCode(KEY_RIGHT)) => {
                editor.scroll_right(false);
            },
            Some(WchResult::KeyCode(KEY_LEFT)) => {
                editor.scroll_left(false);
            },
            Some(WchResult::Char(c)) => {
                // Typed some character
                editor.type_character(char::from_u32(c as u32).expect("Invalid char"), false);
            },
            _ => {
                break;
            }
        }
        editor.display_at_frame_cursor();
        editor.move_cursor_to();
        refresh();
        ch = wget_wch(stdscr());
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

/*#[cfg(not(feature = "wide"))]
fn main() {
    initscr();
    addstr("Need wide character support").unwrap();
    getch();
    endwin();
}
*/
