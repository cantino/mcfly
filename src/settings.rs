use std::env;

#[derive(Debug)]
pub struct Settings {
    pub cmd: String
}

impl Settings {
    // Primitive and fast argument parsing.
    pub fn parse_args() -> Settings {
        let mut parts = Vec::new();

        for (i, argument) in env::args().enumerate() {
            if i == 0 { continue; } // Skip the command name argument
            if i == 1 {
                if argument == "-a" {
                    continue;
                }
            }
            parts.push(argument);
        }

        return Settings { cmd: parts.join(" ") }
    }
}
