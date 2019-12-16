use crate::bash_history;
use crate::history::History;
use crate::settings::Settings;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn clean(settings: &Settings, history: &History, command: &str) {
    // Clean up the database.
    history.delete_command(command);

    // Clean up the contents of MCFLY_HISTORY and all other temporary history files in the same
    // directory.
    clean_temporary_files(&settings.mcfly_history, command);

    // Clean up HISTFILE.
    let histfile =
        PathBuf::from(env::var("HISTFILE").unwrap_or_else(|err| panic!(format!("McFly error: Please ensure that HISTFILE is set ({})", err))));
    bash_history::delete_lines(&histfile, command);
}

fn clean_temporary_files(mcfly_history: &PathBuf, command: &str) {
    let path = mcfly_history.as_path();
    if let Some(directory) = path.parent() {
        let expanded_path =
            fs::canonicalize(directory).unwrap_or_else(|err| panic!(format!("McFly error: The contents of $MCFLY_HISTORY appear invalid ({})", err)));
        let paths = fs::read_dir(&expanded_path).unwrap();

        for path in paths {
            if let Ok(entry) = path {
                if let Some(file_name) = entry.path().file_name() {
                    if let Some(valid_unicode_str) = file_name.to_str() {
                        if valid_unicode_str.starts_with("mcfly.") {
                            bash_history::delete_lines(&entry.path(), command);
                        }
                    }
                }
            }
        }
    }
}
