use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::env;

pub fn bash_history_file_path() -> PathBuf {
    env::home_dir()
        .expect("Unable to access home directory")
        .join(PathBuf::from(".bash_history"))
}

pub fn full_history(path: &PathBuf) -> Vec<String> {
    let mut f: File = File::open(&path)
        .expect(format!("{:?} file not found", &path).as_str());

    let mut bash_history_contents = String::new();
    f.read_to_string(&mut bash_history_contents)
        .expect(format!("Unable to read {:?}", &path).as_str());

    bash_history_contents
        .split("\n")
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(String::from)
        .collect::<Vec<String>>()
}

pub fn last_history_line(path: &PathBuf) -> Option<String> {
    // Could switch to https://github.com/mikeycgto/rev_lines
    full_history(path).last().map(|s| s.trim().to_string())
}
