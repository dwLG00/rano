// This is intended to be a demo syntax highlighting spec for rust files, taken from the rust nanorc
// This is not meant to be complete or used in production (users will be able to specify their own
// syntax highlighting)

extern crate ncurses;
use crate::syntax_highlighting::{HighlightRules, SyntaxHighlight};
use crate::colors;
use regex::Regex;
use ncurses::*;

/*
pub fn build_highlighting_rules() -> HighlightRules {
    // Builds highlighting rules

    let syntax_highlightings: Vec<SyntaxHighlight> = vec![
        // Classes
        SyntaxHighlight::new(
            Regex::new(r"[A-Z][A-Za-z0-9]+").unwrap(),
            COLOR_PAIR(colors::CP_MAGENTA)
        )
    ];

    HighlightRules::new(syntax_highlightings)
}
*/

pub fn build_highlighting_rules() -> HighlightRules {
    // Builds highlighting rules

    let syntax_highlightings: Vec<SyntaxHighlight> = vec![
        // Comment markers
        SyntaxHighlight::new(
            Regex::new(r"(XXX|TODO|FIXME|\?\?\?)").unwrap(),
            COLOR_PAIR(colors::CP_CYAN) | A_BOLD
        ),
        // Attributes
        SyntaxHighlight::new(
            Regex::new(r"#!\[.*\]").unwrap(),
            COLOR_PAIR(colors::CP_MAGENTA)
        ),
        // Comments
        SyntaxHighlight::new(
            Regex::new(r"/\*(.|\n)*?\*/").unwrap(),
            COLOR_PAIR(colors::CP_GREEN)
        ),
        SyntaxHighlight::new(
            Regex::new(r"//.*").unwrap(),
            COLOR_PAIR(colors::CP_GREEN)
        ),
        // Strings
        SyntaxHighlight::new(
            Regex::new(r##"r#+\"(.|\n)*?\"#+"##).unwrap(),
            COLOR_PAIR(colors::CP_YELLOW)
        ),
        SyntaxHighlight::new(
            Regex::new(r##"\\".*\\$(.|\n)*?.*\\""##).unwrap(),
            COLOR_PAIR(colors::CP_YELLOW)
        ),
        SyntaxHighlight::new(
            Regex::new(r#"\"(.|\n)*?\""#).unwrap(),
            COLOR_PAIR(colors::CP_YELLOW)
        ),
        // Classes
        SyntaxHighlight::new(
            Regex::new(r"[A-Z][A-Za-z0-9]+").unwrap(),
            COLOR_PAIR(colors::CP_MAGENTA)
        ),
        // Constants
        SyntaxHighlight::new(
            Regex::new(r"[A-Z][A-Z_0-9]+").unwrap(),
            COLOR_PAIR(colors::CP_MAGENTA)
        ),
        // Macros
        SyntaxHighlight::new(
            Regex::new(r"[a-z_]+!").unwrap(),
            COLOR_PAIR(colors::CP_RED)
        ),
        // Keywords
        SyntaxHighlight::new(
            Regex::new(r"\<(abstract|as|async|await|become|box|break|const|continue|crate|do|dyn|else|enum|extern|false|final|fn|for|if|impl|in|let|loop|macro|match|mod|move|mut|override|priv|pub|ref|return\
            |self|static|struct|super|trait|true|try|type|typeof|unsafe|unsized|use|virtual|where|while|yield)\>").unwrap(),
            COLOR_PAIR(colors::CP_RED) | A_BOLD
        ),
        // Functions
        SyntaxHighlight::new(
            Regex::new(r"fn [a-z_0-9]+").unwrap(),
            COLOR_PAIR(colors::CP_MAGENTA)
        )
    ];

    HighlightRules::new(syntax_highlightings)
}
