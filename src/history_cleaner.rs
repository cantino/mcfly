use crate::history::History;
use crate::settings::{HistoryFormat, Settings};
use crate::shell_history;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn clean(settings: &Settings, history: &History, command: &str) {
    // Clean up the database.
    history.delete_command(command);

    match settings.history_format {
        HistoryFormat::Bash | HistoryFormat::Zsh { .. } => {
            // Clean up the contents of MCFLY_HISTORY and all other temporary history files in the same
            // directory.
            clean_temporary_files(&settings.mcfly_history, settings.history_format, command);

            // Clean up HISTFILE/MCFLY_HISTFILE.
            let histfile = PathBuf::from(
                env::var("HISTFILE")
                    .or_else(|_| env::var("MCFLY_HISTFILE"))
                    .unwrap_or_else(|err| {
                        panic!(
                            "McFly error: Please ensure that HISTFILE/MCFLY_HISTFILE is set ({})",
                            err
                        )
                    }),
            );
            shell_history::delete_lines(&histfile, settings.history_format, command)
        }
        // Fish integration does not use a MCFLY_HISTORY file because we can get the last command
        // during a fish_postexec function.
        // Also, deletion from the fish history file is done by fish itself, via commands sent out
        // in the results file and interpreted by mcfly.fish.
        HistoryFormat::Fish => {}
    }
}

fn clean_temporary_files(mcfly_history: &Path, history_format: HistoryFormat, command: &str) {
    let path = mcfly_history;
    if let Some(directory) = path.parent() {
        let expanded_path = fs::canonicalize(directory).unwrap_or_else(|err| {
            panic!(
                "McFly error: The contents of $MCFLY_HISTORY appear invalid ({})",
                err
            )
        });
        let paths = fs::read_dir(expanded_path).unwrap();

        for entry in paths.flatten() {
            if let Some(file_name) = entry.path().file_name() {
                if let Some(valid_unicode_str) = file_name.to_str() {
                    if valid_unicode_str.starts_with("mcfly.") {
                        shell_history::delete_lines(&entry.path(), history_format, command);
                    }
                }
            }
        }
    }
}
