

pub struct Config {
    tabsize: usize
}

enum Command {
    Set(Attribute, Value)
}

enum Attribute {
}

enum Value {
    Number(usize),
    String(String),
    Atom(String)
}

pub fn default() -> Config {
    Config {
        tabsize: 4
    }
}

pub fn parse_config(config_buffer: Vec<char>) {
    // Parses the config file and spits out a vector
}
