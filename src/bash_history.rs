use std::path::PathBuf;
use std::env;
use regex::Regex;
use std::io::Write;
use std::fs;
use std::fs::OpenOptions;

pub fn bash_history_file_path() -> PathBuf {
    // TODO: This should read the HISTFILE environment variable and source from that file instead.
    env::home_dir()
        .expect("Unable to access home directory")
        .join(PathBuf::from(".bash_history"))
}

pub fn full_history(path: &PathBuf) -> Vec<String> {
    let bash_history_contents = fs::read_to_string(&path)
        .expect(format!("{:?} file not found", &path).as_str());

    let timestamp_regex = Regex::new(r"\A#\d{10}").unwrap();

    bash_history_contents
        .split("\n")
        .filter(|line| !timestamp_regex.is_match(line) && !line.is_empty())
        .map(String::from)
        .collect::<Vec<String>>()
}

pub fn last_history_line(path: &PathBuf) -> Option<String> {
    // Could switch to https://github.com/mikeycgto/rev_lines
    full_history(path).last().map(|s| s.trim().to_string())
}

pub fn delete_last_history_entry_if_search(path: &PathBuf) {
    let bash_history_contents = fs::read_to_string(&path)
        .expect(format!("{:?} file not found", &path).as_str());

    let mut lines = bash_history_contents
        .split("\n")
        .map(String::from)
        .collect::<Vec<String>>();

    let timestamp_regex = Regex::new(r"\A#\d{10}").unwrap();

    if lines.len() > 0 && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }

    if lines.len() == 0 || !lines[lines.len() - 1].starts_with("#mcfly:") {
        return; // Abort if empty or the last line isn't a comment.
    }

    lines.pop();

    if lines.len() > 0 && timestamp_regex.is_match(&lines[lines.len() - 1]) {
        lines.pop();
    }

    lines.push(String::from("")); // New line at end of file expected by bash.

    fs::write(&path, lines.join("\n"))
        .expect(format!("Unable to update {:?}", &path).as_str());
}

pub fn append_history_entry(command: &String, path: &PathBuf) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", command) {
        eprintln!("Couldn't append to file {:?}: {}", &path, e);
    }
}
