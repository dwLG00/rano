extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::Path;
mod lines;
mod nc;

fn open_file(window: WINDOW) -> nc::Editor {
    // Open file given in argument, and return editor created from file contents
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("Requires filepath argument");
    }

    let reader = fs::File::open(Path::new(&args[1]));
    let mut file = reader.ok().expect("Unable to open file");
    nc::Editor::from_file(file, window)
}


// Window creators

fn create_editor_window() -> WINDOW {
    // Create a window with height = max_height - 2 to allow
    // room for functions

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let window = newwin(max_y - 1, max_x, 0, 0);
    wrefresh(window);
    window
}

fn create_control_bar_window() -> WINDOW {
    // Create a window with height = 1 to allow room
    // for control bar

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let window = newwin(1, max_x, max_y, 0);
    wrefresh(window);
    window
}

fn draw_control_bar(window: WINDOW) {
    // Draws control sequences
}

fn refresh_all_windows(windows: &Vec<WINDOW>) {
    // Refreshes all windows
    for window in windows.iter() {
        wrefresh(window.clone());
    }
}

// Main loop

fn main() {
    initscr();
    raw();
    noecho();

    // Create windows
    let mut windows = Vec::new();
    windows.push(stdscr());

    let editor_window = create_editor_window();
    windows.push(editor_window);
    //let ctrl_window = create_control_bar_window();

    for window in windows.iter() {
        keypad(window.clone(), true);
    }

    let mut editor = open_file(editor_window);

    editor.display_at_frame_cursor();
    editor.move_cursor_to(editor_window);
    wrefresh(editor_window);

    let mut ch = wget_wch(editor_window);
    while true {
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
            Some(WchResult::Char(char_code)) => {
                // Typed some character
                let c = char::from_u32(char_code as u32).expect("Invalid char");
                match c {
                    '\n' => {
                        // Handle newlines separately
                        editor.newline(false);
                    },
                    '\u{007F}' => {
                        // Handle backspaces separately
                        editor.backspace(false);
                    },
                    '\u{0018}' => {
                        // Ctrl-X -> break
                        break;
                    },
                    '\u{000F}' => {
                        // Ctrl-O -> save loop
                        //editor.save_loop();
                    },
                    _ => {
                        editor.type_character(c, false);
                    }
                }
            },
            _ => {
                break;
            }
        }
        werase(editor_window);
        editor.display_at_frame_cursor();
        editor.move_cursor_to(editor_window);
        wrefresh(editor_window);
        ch = wget_wch(editor_window);
    }
    endwin();
}
