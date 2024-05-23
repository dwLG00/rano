

pub struct Config {
    tabsize: usize
    
}

pub fn default() -> Config {
    Config {
        tabsize: 4
    }
}

pub fn parse_config(config_buffer: Vec<char>) {
    // Parses the config file and spits out a vector
}
