extern crate ncurses;
use std::char;
use ncurses::*;
use std::env;
use std::io::{Read, Write};
use std::fs;
use std::io;
use std::cmp::min;
use std::path::Path;
use std::process;
mod gap_buffer;
//mod lines;
//mod nc;
mod gapnc;
mod colors;

// Missing keycodes
const KEY_SDOWN: i32 = 336;
const KEY_SUP: i32 = 337;


// File IO

fn open_file(window: WINDOW, path: &str) -> gapnc::GapEditor {
    // Open file given in argument, and return editor created from file contents

    let reader = fs::File::open(Path::new(path));
    let mut file = reader.ok().expect("Unable to open file");
    //nc::Editor::from_file(file, window)
    gapnc::GapEditor::from_file(file, window)
}

//fn save_to_file(filename: String, editor: &nc::Editor) -> Result<(), io::Error>{
fn save_to_file(filename: String, editor: &gapnc::GapEditor) -> Result<(), io::Error>{
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
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &" ".repeat(max_x as usize)).unwrap();
    wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    let ctrl_string = "HELP =>\t[Ctrl-X]  Quit\t[Ctrl-O]  Save\t[Ctrl-K]  Cut\t[Ctrl-J]  Copy\t[Ctrl-U]  Paste\t[Ctrl-I]  Clipboard\t[Ctrl-/]  Go To Line\t[Ctrl-L]  Set Mark".to_string();
    let ctrl_string_len = ctrl_string.len();
    mvwaddstr(window, 1, 0, &(ctrl_string + &" ".repeat(max_x as usize - ctrl_string_len))).unwrap();
}

//fn save_loop(window: WINDOW, editor: &nc::Editor, path: &String) -> bool{
fn save_loop(window: WINDOW, editor: &gapnc::GapEditor, path: &String) -> bool{
    // Runs the UI process of saving
    // Returns true if actually saved

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    let mut cur_x = 0;
    let mut cur_y = 0;

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);

    let ctrl_string = "Optn =>\t[Enter]  Save\t[Ctrl-C]  Quit".to_string();
    let ctrl_string_len = ctrl_string.len();
    let file_input_string = "File Name to Write: ".to_string();
    let file_input_string_len = file_input_string.len();
    mvwaddstr(window, 1, 0, &(ctrl_string + &" ".repeat(max_x as usize - ctrl_string_len))).unwrap();
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &(file_input_string + &" ".repeat(max_x as usize - file_input_string_len))).unwrap();
    wmove(window, 0, file_input_string_len as i32);
    waddstr(window, &path).unwrap();
    wrefresh(window);

    let left_limit = file_input_string_len as i32; // If cur_x == left_limit, prevent deletion
    let right_limit = max_x - 1; // If cur_x == max_x, prevent character addition

    let mut filename_buffer = path.clone();

    let mut ch;
    let mut ret: bool = false;
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
                    '\r' => {
                        // Enter
                        save_to_file(filename_buffer, editor);
                        ret = true;
                        break;
                    },
                    '\u{001C}' => {
                        // Ctrl-\
                    },
                    '\u{0001}'..='\u{001F}' => {
                        beep();
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
    wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    ret
}

//fn exit_loop(window: WINDOW, editor: &nc::Editor, path: &String) -> bool {
fn exit_loop(window: WINDOW, editor: &gapnc::GapEditor, path: &String) -> bool {
    // Handle UI sequence for exiting when you haven't saved

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    let mut cur_x = 0;
    let mut cur_y = 0;

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);

    let ctrl_string = "Optn =>\t[Y]  Yes\t[N]  No\t[Ctrl-C]  Cancel".to_string();
    let ctrl_string_len = ctrl_string.len();
    let buffer_query_string = "Save modified buffer?".to_string();
    let buffer_query_string_len = buffer_query_string.len();
    mvwaddstr(window, 1, 0, &(ctrl_string + &" ".repeat(max_x as usize - ctrl_string_len))).unwrap();
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &(buffer_query_string + &" ".repeat(max_x as usize - buffer_query_string_len))).unwrap();
    wmove(window, 0, buffer_query_string_len as i32 + 1);
    wrefresh(window);


    let mut ch = wget_wch(window);
    loop {
        match ch {
            Some(WchResult::Char(char_code)) => {
                let c = char::from_u32(char_code as u32).expect("Invalid character");
                match c {
                    'y' => {
                        wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
                        return save_loop(window, editor, path); // Cancel the exit if the save is also cancelled
                    },
                    'n' => {
                        return true;
                    },
                    '\u{0003}' => {
                        return false;
                    },
                    _ => {
                        beep();
                    },
                }
            },
            _ => {
                beep();
            }
        }
        ch = wget_wch(window);
    }
}

fn go_to_line_loop(window: WINDOW, editor: &gapnc::GapEditor) -> Option<usize> {
    // Handle UI sequence for going to a particular line

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    let mut cur_x = 0;
    let mut cur_y = 0;

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);

    let ctrl_string = "Optn =>\t[Enter]  Go to Line\t[Ctrl-C]  Quit".to_string();
    let ctrl_string_len = ctrl_string.len();
    let lineno_input_string = "Go to Line Number: ".to_string();
    let lineno_input_string_len = lineno_input_string.len();
    mvwaddstr(window, 1, 0, &(ctrl_string + &" ".repeat(max_x as usize - ctrl_string_len))).unwrap();
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &(lineno_input_string + &" ".repeat(max_x as usize - lineno_input_string_len))).unwrap();
    wmove(window, 0, lineno_input_string_len as i32);
    wrefresh(window);

    let left_limit = lineno_input_string_len as i32; // If cur_x == left_limit, prevent deletion
    let right_limit = max_x - 1; // If cur_x == max_x, prevent character addition

    let mut lineno_buffer = String::new();

    let mut ch;
    let mut ret: bool = false;
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
                        lineno_buffer.pop();
                    },
                    '\r' => {
                        // Enter
                        match lineno_buffer.parse::<usize>() {
                            Ok(lineno) => { return Some(lineno) },
                            Err(e) => {
                                // We can't parse the buffer, throw an error
                                beep();
                            }
                        }
                        //
                    },
                    '\u{0001}'..='\u{001F}' => {
                        beep();
                    },
                    _ => {
                        if cur_x == right_limit {
                            beep();
                            continue;
                        }

                        waddch(window, c as chtype);
                        lineno_buffer.push(c);
                    }
                }
            },
            _ => {break;}
        }
        wrefresh(window);
    }
    wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    None
}

fn clipboard_select_loop(window: WINDOW, editor: &gapnc::GapEditor) -> Option<usize>{
    // Handle UI sequence for going to a particular line
    // Returns an Option<usize> of the new clipboard position

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(window, &mut max_y, &mut max_x);

    let mut cur_x = 0;
    let mut cur_y = 0;

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);

    let ctrl_string = "Optn =>\t[Enter]  Go to Line\t[Up]  Select Previous\t[Down]  Select Next\t[Ctrl-C]  Quit".to_string();
    let ctrl_string_len = ctrl_string.len();
    let clipboard_select_string = "Clipboard: ".to_string();
    let clipboard_select_string_len = clipboard_select_string.len();
    mvwaddstr(window, 1, 0, &(ctrl_string + &" ".repeat(max_x as usize - ctrl_string_len))).unwrap();
    wattron(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    mvwaddstr(window, 0, 0, &(clipboard_select_string + &" ".repeat(max_x as usize - clipboard_select_string_len))).unwrap();
    wmove(window, 0, clipboard_select_string_len as i32); // This is the position right after the clipboard select string

    // Get the initial clipboard string
    let left_limit = clipboard_select_string_len as i32; // If cur_x == left_limit, prevent deletion
    let right_limit = max_x - 1; // If cur_x == max_x, prevent character addition

    let clipboard_maxlen = (right_limit - left_limit) as usize; // Limit the number of characters of the clipboard buffer displayed
    let mut clipboard_cursor = match editor.get_clipboard_cursor() {
        Some(i) => i,
        None => { return None; }
    };
    let mut clipboard_string = match editor.get_clipboard(clipboard_cursor) {
        Some(buffer) => pad(display_with_cutoff(buffer.to_vec(), clipboard_maxlen, 3), clipboard_maxlen),
        None => pad(String::new(), clipboard_maxlen)
    };

    waddstr(window, &clipboard_string);
    wrefresh(window);


    let mut ch;
    let mut ret: bool = false;
    loop {
        ch = wget_wch(window);
        getyx(window, &mut cur_y, &mut cur_x); // Get current cursor location
        match ch {
            Some(WchResult::KeyCode(KEY_UP)) => {
                if clipboard_cursor == 0 {
                    // We've reached the oldest clipboard entry
                    beep();
                    continue;
                }
                clipboard_cursor -= 1;
                clipboard_string = match editor.get_clipboard(clipboard_cursor) {
                    Some(buffer) => pad(display_with_cutoff(buffer.to_vec(), clipboard_maxlen, 3), clipboard_maxlen),
                    None => pad(String::new(), clipboard_maxlen)
                };
                wmove(window, 0, clipboard_select_string_len as i32);
                waddstr(window, &clipboard_string);
                wmove(window, 0, clipboard_select_string_len as i32);
                // Get the previous clipboard entry
            },
            Some(WchResult::KeyCode(KEY_DOWN)) => {
                if clipboard_cursor == editor.clipboard_len() - 1 {
                    // We've reached the newest clipboard entry
                    beep();
                    continue;
                }
                clipboard_cursor += 1;
                clipboard_string = match editor.get_clipboard(clipboard_cursor) {
                    Some(buffer) => pad(display_with_cutoff(buffer.to_vec(), clipboard_maxlen, 3), clipboard_maxlen),
                    None => pad(String::new(), clipboard_maxlen)
                };
                wmove(window, 0, clipboard_select_string_len as i32);
                waddstr(window, &clipboard_string);
                wmove(window, 0, clipboard_select_string_len as i32);
                // Get the next clipboard entry
            },
            Some(WchResult::Char(char_code)) => {
                let c = char::from_u32(char_code as u32).expect("Invalid char");
                match c {
                    '\u{0003}' => {
                        // Ctrl-C
                        break;
                    },
                    '\r' => {
                        return Some(clipboard_cursor);
                        // Enter
                        /*
                        match lineno_buffer.parse::<usize>() {
                            Ok(lineno) => { return Some(lineno) },
                            Err(e) => {
                                // We can't parse the buffer, throw an error
                                beep();
                            }
                        }
                        */
                        //
                    },
                    _ => {
                        beep();
                    }
                }
            },
            _ => {break;}
        }
        wrefresh(window);
    }
    wattroff(window, COLOR_PAIR(colors::CP_HIGHLIGHT));
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    None
}

fn refresh_all_windows(windows: &Vec<WINDOW>) {
    // Refreshes all windows
    for window in windows.iter() {
        wrefresh(window.clone());
    }
}

// Various helpers
fn display_with_cutoff(buffer: Vec<char>, cutoff: usize, dots: usize) -> String {
    // Display the first `cutoff` characters.
    // If the buffer contains a newline before the cutoff, end at the newline.
    // If the buffer trails off, replace the last `dots` characters with ellipses
    assert!(cutoff > dots);

    let mut out_buffer = Vec::<char>::new();
    for (i, ch) in buffer.iter().enumerate() {
        if *ch != '\n' && i < cutoff {
            out_buffer.push(*ch);
        } else if i == cutoff {
            if buffer.len() > cutoff {
                for j in cutoff-dots..cutoff {
                    // set ellipses
                    out_buffer[j] = '.';
                }
            }
            break;
            //return out_buffer.into_iter().collect();
        } else {
            let out_buffer_len = out_buffer.len();
            if i != buffer.len() - 1 {
                // set ellipses
                for j in min(out_buffer_len, dots)-dots..out_buffer_len {
                    out_buffer[j] = '.';
                }
            }
            break;
            //return out_buffer.into_iter().collect();
        }
    }
    return out_buffer.into_iter().collect();
}

fn pad(string: String, length: usize) -> String {
    // Right pad a string with spaces
    assert!(string.len() <= length);
    if string.len() == length {
        string
    } else {
        let mut outstring = string.clone();
        let padby = length - string.len();
        for _ in 0..padby {
            outstring.push(' ');
        }
        outstring
    }
}

// Main loop

fn main() {
    setlocale(constants::LcCategory::all, ""); // We need this to display weird unicode characters
    initscr();
    raw();
    noecho();
    nonl();

    // Initialize Colors
    use_default_colors();
    start_color();
    init_pair(colors::CP_HIGHLIGHT, COLOR_BLACK, COLOR_WHITE);

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
        /*
        path = String::new();
        editor = nc::Editor::blank(editor_window);
        */
        panic!("Empty Editor not implemented!");
    } else if args.len() == 2 {
        path = args[1].to_string();
        editor = open_file(editor_window, &path);
    } else {
        panic!("More than 1 argument provided!");
    }



    draw_control_bar(ctrl_window);
    editor.display_at_frame_cursor();
    //editor.move_cursor_to(editor_window);
    editor.move_cursor_to();
    //wrefresh(editor_window);
    refresh_all_windows(&windows);

    let mut ch = wget_wch(editor_window);
    while true {
        match ch {
            // Arrow keys
            Some(WchResult::KeyCode(KEY_DOWN)) => {
                if editor.is_shift_selected() {
                    editor.deselect_marks();
                }
                editor.scroll_down();
            },
            Some(WchResult::KeyCode(KEY_UP)) => {
                if editor.is_shift_selected() {
                    editor.deselect_marks();
                }
                editor.scroll_up();
            },
            Some(WchResult::KeyCode(KEY_RIGHT)) => {
                if editor.is_shift_selected() {
                    editor.deselect_marks();
                }
                editor.scroll_right();
            },
            Some(WchResult::KeyCode(KEY_LEFT)) => {
                if editor.is_shift_selected() {
                    editor.deselect_marks();
                }
                editor.scroll_left();
            },
            Some(WchResult::KeyCode(KEY_SDOWN)) => {
                if !editor.is_shift_selected() {
                    editor.set_mark();
                    editor.set_select_shift();
                }
                editor.scroll_down();
                //panic!("{} {} {} {}", KEY_SDOWN, KEY_SUP, KEY_SRIGHT, KEY_SLEFT);
            },
            Some(WchResult::KeyCode(KEY_SUP)) => {
                if !editor.is_shift_selected() {
                    editor.set_mark();
                    editor.set_select_shift();
                }
                editor.scroll_up();
            },
            Some(WchResult::KeyCode(KEY_SRIGHT)) => {
                if !editor.is_shift_selected() {
                    editor.set_mark();
                    editor.set_select_shift();
                }
                editor.scroll_right();
            },
            Some(WchResult::KeyCode(KEY_SLEFT)) => {
                if !editor.is_shift_selected() {
                    editor.set_mark();
                    editor.set_select_shift();
                }
                editor.scroll_left();
            },
            // Unrecognized keycode
            Some(WchResult::KeyCode(code)) => {
                panic!("Got keycode: {:?}", code);
            }
            // Actual characters + Ctrl keys
            Some(WchResult::Char(char_code)) => {
                // Typed some character
                let c = char::from_u32(char_code as u32).expect("Invalid char");
                match c {
                    '\r' => {
                        // Enter
                        editor.newline();
                    },
                    '\t' => {
                        editor.tab();
                    },
                    '\u{007F}' => {
                        // Handle backspaces separately
                        editor.backspace();
                        //editor.backspace(false);
                    },
                    '\u{0018}' => {
                        // Ctrl-X -> break
                        //TODO Implement
                        if editor.save_flag {
                            break;
                        } else if exit_loop(ctrl_window, &editor, &path) {
                            break;
                        } else {
                            draw_control_bar(ctrl_window);
                            wrefresh(ctrl_window);
                        }
                        return;
                    },
                    '\u{000F}' => {
                        // Ctrl-O -> save loop
                        if save_loop(ctrl_window, &editor, &path) {
                            editor.set_save();
                        }
                        draw_control_bar(ctrl_window);
                        wrefresh(ctrl_window);
                    },
                    '\u{0009}' => {
                        // Ctrl-I -> clipboard
                        match clipboard_select_loop(ctrl_window, &editor) {
                            Some(new_clipboard_cursor) => { editor.set_clipboard_cursor(new_clipboard_cursor) },
                            None => {}
                        }
                        draw_control_bar(ctrl_window);
                        wrefresh(ctrl_window);
                    },
                    '\u{000A}' => {
                        // Ctrl-J -> copy
                        editor.copy();
                    },
                    '\u{000B}' => {
                        // Ctrl-K -> cut
                        editor.cut();
                    },
                    '\u{000C}' => {
                        // Ctrl-L -> Set mark
                        editor.set_mark();
                    }
                    '\u{0015}' => {
                        // Ctrl-U -> paste
                        //editor.paste();
                        //TODO Implement
                        //break;
                        editor.paste();
                    },
                    '\u{001F}' => {
                        // Ctrl-/ -> Go to line
                        match go_to_line_loop(ctrl_window, &editor) {
                            Some(n) => { editor.go_to_line(n); },
                            None => { beep(); }
                        }
                        draw_control_bar(ctrl_window);
                        wrefresh(ctrl_window);
                    },
                    '\u{0001}'..='\u{001F}' => { // All other control keys
                        beep();
                    },
                    _ => {
                        editor.type_character(c);
                        //break;
                    }
                }
            },
            _ => {
                break;
            }
        }
        werase(editor_window);
        editor.display_at_frame_cursor();
        //editor.move_cursor_to(editor_window);
        editor.move_cursor_to();
        wrefresh(editor_window);
        //refresh_all_windows(&windows);
        ch = wget_wch(editor_window);
    }
    endwin();
}
