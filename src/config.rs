extern crate ncurses;
extern crate custom_error;
use crate::syntax_highlighting;
use crate::colors;
use ncurses::*;
use custom_error::custom_error;
use regex::Regex;
use std::{error::Error, fmt};

// Data Structures
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    StringLiteral(String),
    Number(usize),
    Semicolon,
}

#[derive(Debug)]
pub struct Config {
    pub highlight_rules: syntax_highlighting::HighlightRules
}

// Errors
custom_error!{pub TokenizationError
    UnclosedQuotes{location: usize} = "Quotes unclosed at position {location}.",
    CantParseNumber{location: usize} = "Number unable to be parsed at position {location}."
}

// Parsing functions
pub fn tokenize(chars: Vec<char>) -> Result<Vec<Token>, TokenizationError> {
    // Ignores whitespace, unless quoted
    let mut stack = Vec::<char>::new();
    let mut tokens = Vec::<Token>::new();

    let mut quote_flag = false; // Currently parsing a quote
    let mut comment_flag = false; // Currently parsing a comment
    let mut number_flag = false; // Currently parsing a number (integer)

    for (i, c) in chars.iter().enumerate() {
        match c {
            ';' => {
                if quote_flag {
                    // Ignore, add to the stack
                    stack.push(*c);
                } else if comment_flag {
                    // Ignore entirely
                } else if number_flag {
                    let s: String = stack.clone().into_iter().collect();
                    match s.parse::<usize>() {
                        Ok(n) => { tokens.push(Token::Number(n)); },
                        Err(e) => { return Err(TokenizationError::CantParseNumber {location: i}); }
                    }
                    tokens.push(Token::Semicolon);
                    stack.clear();
                    number_flag = false;
                } else {
                    // Merge stack contents into a single string and push a keyword
                    let s: String = stack.clone().into_iter().collect();
                    tokens.push(Token::Keyword(s));
                    tokens.push(Token::Semicolon);
                    stack.clear();
                }
            },
            '"' => {
                if quote_flag {
                    let s: String = stack.clone().into_iter().collect();
                    tokens.push(Token::StringLiteral(s));
                    stack.clear();
                    quote_flag = false;
                } else if comment_flag {
                    // Ignore entirely
                } else if stack.len() > 0 {
                    // Parse error
                    return Err(TokenizationError::UnclosedQuotes{ location: i });
                } else {
                    quote_flag = true;
                }
            },
            '/' => {
                if quote_flag {
                    // Just push the character onto the stack
                    stack.push(*c);
                } else if stack.len() > 0 {
                    // Parse error
                } else {
                    // Start the comments
                    comment_flag = true;
                }
            },
            '\n' => {
                if stack.len() == 0 {
                    continue;
                }
                if comment_flag {
                    comment_flag = false;
                } else if quote_flag {
                    // Parse error - quotes not closed by end of line
                    return Err(TokenizationError::UnclosedQuotes{ location: i });
                } else if number_flag {
                    let s: String = stack.clone().into_iter().collect();
                    match s.parse::<usize>() {
                        Ok(n) => { tokens.push(Token::Number(n)); },
                        Err(e) => { return Err(TokenizationError::CantParseNumber {location: i}); }
                    }
                    stack.clear();
                    number_flag = false;
                } else {
                    let s: String = stack.clone().into_iter().collect();
                    tokens.push(Token::Keyword(s));
                    stack.clear();
                }
            },
            ' ' => {
                if stack.len() == 0 {
                    continue;
                }
                if quote_flag {
                    stack.push(*c);
                } else if comment_flag {
                    // Ignore
                } else if number_flag {
                    let s: String = stack.clone().into_iter().collect();
                    match s.parse::<usize>() {
                        Ok(n) => { tokens.push(Token::Number(n)); },
                        Err(e) => { return Err(TokenizationError::CantParseNumber {location: i}); }
                    }
                    stack.clear();
                    number_flag = false;
                } else {
                    let s: String = stack.clone().into_iter().collect();
                    tokens.push(Token::Keyword(s));
                    stack.clear();
                }
            },
            c if c.is_whitespace() => { // Non-newline, non-space whitespace
                if stack.len() == 0 {
                    continue;
                }
                if quote_flag {
                    // Parse error - 'weird' whitespace found while parsing string
                    return Err(TokenizationError::UnclosedQuotes{location: i});
                } else if comment_flag {
                    // Ignore
                } else if number_flag {
                    let s: String = stack.clone().into_iter().collect();
                    match s.parse::<usize>() {
                        Ok(n) => { tokens.push(Token::Number(n)); },
                        Err(e) => { return Err(TokenizationError::CantParseNumber {location: i}); }
                    }
                    stack.clear();
                    number_flag = false;
                } else {
                    let s: String = stack.clone().into_iter().collect();
                    tokens.push(Token::Keyword(s));
                    stack.clear();
                }
            },
            '0'..='9' => { // Number characters
                if quote_flag {
                    stack.push(*c);
                } else if comment_flag {
                    // Ignore
                } else if stack.len() == 0 {
                    // Stack empty -> Set number flag
                    number_flag = true;
                    stack.push(*c);
                } else {
                    stack.push(*c);
                }
            },
            _ => {
                if quote_flag {
                    stack.push(*c);
                } else if comment_flag {
                    // Ignore
                } else {
                    stack.push(*c);
                }
            }
        }
    }

    // Check for remaining chars in stack after main parsing loop
    if stack.len() > 0 {
        let s: String = stack.clone().into_iter().collect();
        if number_flag {
            match s.parse::<usize>() {
                Ok(n) => { tokens.push(Token::Number(n)); },
                Err(e) => { return Err(TokenizationError::CantParseNumber {location: chars.len() - 1}); }
            }
        } else {
            tokens.push(Token::Keyword(s));
        }
    }
    Ok(tokens)
}

pub fn token_to_color(token: Token) -> Option<u64> {
    match token {
        Token::Keyword(s) => {
            match s.as_str() {
                "red" => Some(COLOR_PAIR(colors::CP_RED)),
                "brightred" => Some(COLOR_PAIR(colors::CP_RED) | A_BOLD),
                "blue" => Some(COLOR_PAIR(colors::CP_BLUE)),
                "brightblue" => Some(COLOR_PAIR(colors::CP_BLUE) | A_BOLD),
                "green" => Some(COLOR_PAIR(colors::CP_GREEN)),
                "brightgreen" => Some(COLOR_PAIR(colors::CP_GREEN) | A_BOLD),
                "yellow" => Some(COLOR_PAIR(colors::CP_YELLOW)),
                "brightyellow" => Some(COLOR_PAIR(colors::CP_YELLOW) | A_BOLD),
                "cyan" => Some(COLOR_PAIR(colors::CP_CYAN)),
                "brightcyan" => Some(COLOR_PAIR(colors::CP_CYAN) | A_BOLD),
                "magenta" => Some(COLOR_PAIR(colors::CP_MAGENTA)),
                "brightmagenta" => Some(COLOR_PAIR(colors::CP_MAGENTA) | A_BOLD),
                "black" => Some(COLOR_PAIR(colors::CP_BLACK)),
                "brightblack" => Some(COLOR_PAIR(colors::CP_BLACK) | A_BOLD),
                "white" => Some(COLOR_PAIR(colors::CP_WHITE)),
                "brightwhite" => Some(COLOR_PAIR(colors::CP_WHITE) | A_BOLD),
                _ => None
            }
        },
        _ => None
    }
}

pub fn token_is_color(token: Token) -> bool {
    // Matches only if token is a color code
    match token {
        Token::Keyword(s) => {
            (s == "red") | (s == "brightred")
          | (s == "blue") | (s == "brightblue")
          | (s == "green") | (s == "brightgreen")
          | (s == "yellow") | (s == "brightyellow")
          | (s == "cyan") | (s == "brightcyan")
          | (s == "magenta") | (s == "brightmagenta")
          | (s == "black") | (s == "brightblack")
          | (s == "white") | (s == "brightwhite")
        },
        _ => false
    }
}

pub fn parse(tokens: Vec<Token>) -> Option<Config> {
    // Parses a vector of tokens into a config file
    let mut syntax_highlighting_vector = Vec::<syntax_highlighting::SyntaxHighlight>::new();
    let mut stack = Vec::<Token>::new();

    for token in tokens.iter() {
        match token {
            Token::Semicolon => {
                // As of right now, only support `color/2`
                if stack.len() != 3 { return None; }
                match &stack[0] {
                    Token::Keyword(s) => { if s != "color" { return None; }},
                    _ => { return None; }
                }
                let token_color = match token_to_color(stack[1].clone()) {
                    Some(color) => color,
                    None => { return None; }
                };
                match &stack[2] {
                    Token::StringLiteral(s) => {
                        let sh = syntax_highlighting::SyntaxHighlight::new(
                            Regex::new(&regex::escape(&s)).unwrap(),
                            token_color
                        );
                        syntax_highlighting_vector.push(sh);
                    },
                    _ => { return None; }
                }
                stack.clear();
            },
            _ => {
                stack.push(token.clone());
            }
        }
    }

    if stack.len() > 0 {
        // Unfinished clause in file, parse error...
        return None;
    }

    Some(Config { highlight_rules: syntax_highlighting::HighlightRules::new(syntax_highlighting_vector) })
}
