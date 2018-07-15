use std::fs::File;
use std::io::Read;
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct HistoryEntry {
    pub cmd: String,
}

#[derive(Debug)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    pub fn load() -> History {
        let history_file_path = env::home_dir().unwrap().join(PathBuf::from(".bash_history"));
        return History::from_path(history_file_path.to_str().unwrap());
    }

    pub fn from_path(file_name: &str) -> History {
        let mut f: File = File::open(file_name).unwrap();
        let mut bash_history_contents = String::new();
        f.read_to_string(&mut bash_history_contents).unwrap();

        let entries = bash_history_contents
            .split("\n")
            .filter(|&line| !line.starts_with('#'))
            .map(|line| HistoryEntry { cmd: String::from(line) })
            .collect();

        return History { entries }
    }
}

impl HistoryEntry {
    pub fn matches(&self, str: &String) -> bool {
        return self.cmd.contains(str);
    }
}
