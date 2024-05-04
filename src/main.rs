extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::{Read, Write};
use std::fs;
use std::io;
use std::path::Path;
mod lines;
mod nc;

static CP_HIGHLIGHT: i16 = 1;

// File IO

fn open_file(window: WINDOW, path: &str) -> nc::Editor {
    // Open file given in argument, and return editor created from file contents

    let reader = fs::File::open(Path::new(path));
    let mut file = reader.ok().expect("Unable to open file");
    nc::Editor::from_file(file, window)
}

fn save_to_file(filename: String, editor: &nc::Editor) -> Result<(), io::Error>{
    // Saves the file to the given path
    let mut maybe_file = fs::OpenOptions::new().write(true).create(true).open(&filename);
    match maybe_file {
        Ok(mut file) => {
            file.write(editor.export().as_bytes());
            Ok(())
        },
        Err(T) => Err(T)
    }
}

fn file_exists(filename: String) -> bool {
    // Check if a file exists
    let mut file = fs::OpenOptions::new().read(true).open(&filename);
    match file {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => false,
            _ => true
        },
        _ => true
    }
}

// Window creators

fn create_editor_window() -> WINDOW {
    // Create a window with height = max_height - 2 to allow
    // room for functions

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let window = newwin(max_y - 2, max_x, 0, 0);
    wrefresh(window);
    window
}

fn create_control_bar_window() -> WINDOW {
    // Create a window with height = 1 to allow room
    // for control bar

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let window = newwin(2, max_x, max_y - 2, 0);
    wrefresh(window);
    window
}

// Window drawers and helpers

fn draw_control_bar(window: WINDOW) {
    // Draws control sequences

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    //mvwaddstr(window, 0, 0, &"\u{2593}".repeat(max_x as usize)).unwrap();
    wattron(window, COLOR_PAIR(CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &" ".repeat(max_x as usize)).unwrap();
    wattroff(window, COLOR_PAIR(CP_HIGHLIGHT));
    mvwaddstr(window, 1, 0, "HELP =>\t\t\t[Ctrl-X]  Quit\t\t[Ctrl-O]  Save\t\t[Ctrl-K]  Cut").unwrap();
}

fn save_loop(window: WINDOW, editor: &nc::Editor, path: &String) {
    // Runs the UI process of saving

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    let mut cur_x = 0;
    let mut cur_y = 0;

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);

    mvwaddstr(window, 1, 0, "HELP =>\t\t\t[Enter]  Save\t\t[Ctrl-C]  Quit").unwrap();
    wattron(window, COLOR_PAIR(CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, "File Name to Write: ").unwrap();
    waddstr(window, &path).unwrap();
    wrefresh(window);

    let left_limit = 20; // If cur_x == left_limit, prevent deletion
    let right_limit = max_x - 1; // If cur_x == max_x, prevent character addition

    let mut filename_buffer = path.clone();

    let mut ch;
    loop {
        ch = wget_wch(window);
        getyx(window, &mut cur_y, &mut cur_x); // Get current cursor location
        match ch {
            Some(WchResult::Char(char_code)) => {
                let c = char::from_u32(char_code as u32).expect("Invalid char");
                match c {
                    '\u{0003}' => {
                        // Ctrl-C
                        break;
                    },
                    '\u{007F}' => {
                        // Backspace

                        // Check if can't delete further
                        if cur_x == left_limit {
                            beep();
                            continue;
                        }

                        // We are essentially replacing the characters with spaces
                        wmove(window, cur_y, cur_x - 1);
                        wdelch(window);
                        winsch(window, ' ' as chtype);
                        filename_buffer.pop();
                    },
                    '\n' => {
                        // Enter
                        save_to_file(filename_buffer, editor);
                        break;
                    },
                    _ => {
                        if cur_x == right_limit {
                            beep();
                            continue;
                        }

                        waddch(window, c as chtype);
                        filename_buffer.push(c);
                    }
                }
            },
            _ => {break;}
        }
        wrefresh(window);
    }
    wattroff(window, COLOR_PAIR(CP_HIGHLIGHT));
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
}

fn refresh_all_windows(windows: &Vec<WINDOW>) {
    // Refreshes all windows
    for window in windows.iter() {
        wrefresh(window.clone());
    }
}



// Main loop

fn main() {
    setlocale(constants::LcCategory::all, ""); // We need this to display weird unicode characters
    initscr();
    raw();
    noecho();

    // Initialize Colors
    use_default_colors();
    start_color();
    init_pair(CP_HIGHLIGHT, COLOR_BLACK, COLOR_WHITE);

    // Create windows
    let mut windows = Vec::new();

    let editor_window = create_editor_window();
    let ctrl_window = create_control_bar_window();

    windows.push(ctrl_window); // Draw control bar before editor
    windows.push(editor_window);

    keypad(stdscr(), true);
    for window in windows.iter() {
        keypad(window.clone(), true);
    }

    // Initialize editor
    let args: Vec<_> = env::args().collect();
    let mut path;
    let mut editor;
    /*
    if args.len() != 2 {
        panic!("Requires filepath argument");
    }
    */
    if args.len() == 1 {
        path = String::new();
        editor = nc::Editor::blank(editor_window);
    } else if args.len() == 2 {
        path = args[1].to_string();
        editor = open_file(editor_window, &path);
    } else {
        panic!("More than 1 argument provided!");
    }



    draw_control_bar(ctrl_window);
    editor.display_at_frame_cursor();
    editor.move_cursor_to(editor_window);
    //wrefresh(editor_window);
    refresh_all_windows(&windows);

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
                        save_loop(ctrl_window, &editor, &path);
                        draw_control_bar(ctrl_window);
                        wrefresh(ctrl_window);
                    },
                    '\u{000B}' => {
                        // Ctrl-K -> cut
                        editor.cut_line();
                    },
                    '\u{0015}' => {
                        // Ctrl-U -> paste
                        editor.paste();
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
        //refresh_all_windows(&windows);
        ch = wget_wch(editor_window);
    }
    endwin();
}
