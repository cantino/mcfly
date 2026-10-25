use crate::settings::HistoryFormat;
use regex::Regex;
use rusqlite::{Connection, OpenFlags};
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

// Nushell's plaintext history stores multi-line commands on one line, with newlines written as `<\n>`.
const NU_NEWLINE_ESCAPE: &str = "<\\n>";

fn read_ignoring_utf_errors(path: &Path) -> String {
    let mut f =
        File::open(path).unwrap_or_else(|_| panic!("McFly error: {:?} file not found", path));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .unwrap_or_else(|_| panic!("McFly error: Unable to read from {:?}", path));
    String::from_utf8_lossy(&buffer).to_string()
}

// Zsh uses a meta char (0x83) to signify that the previous character should be ^ 32.
fn read_and_unmetafy(path: &Path) -> String {
    let mut f =
        File::open(path).unwrap_or_else(|_| panic!("McFly error: {:?} file not found", path));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .unwrap_or_else(|_| panic!("McFly error: Unable to read from {:?}", path));
    for index in (0..buffer.len()).rev() {
        if buffer[index] == 0x83 {
            buffer.remove(index);
            buffer[index] ^= 32;
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}

#[allow(clippy::if_same_then_else)]
fn has_leading_timestamp(line: &str) -> bool {
    let mut matched_chars = 0;

    for (index, c) in line.chars().enumerate() {
        if index == 0 && c == '#' {
            matched_chars += 1;
        } else if index > 0 && index < 11 && (c.is_ascii_digit()) {
            matched_chars += 1;
        } else if index > 11 {
            break;
        }
    }

    matched_chars == 11
}

#[must_use]
pub fn history_file_path() -> PathBuf {
    let path = PathBuf::from(
        env::var("HISTFILE")
            .or_else(|_| env::var("MCFLY_HISTFILE"))
            .unwrap_or_else(|err| {
                panic!(
            "McFly error: Please ensure HISTFILE or MCFLY_HISTFILE is set for your shell ({err})"
        )
            }),
    );
    fs::canonicalize(path).unwrap_or_else(|err| {
        panic!("McFly error: The contents of $HISTFILE/$MCFLY_HISTFILE appears invalid ({err})")
    })
}

/// Represents each entry in a history file.
#[derive(Debug)]
pub struct HistoryCommand {
    /// The user's command.
    pub command: String,
    /// When the command was run, in seconds since Unix epoch.
    pub when: i64,
    /// The format of the file, so we can write the record back out.
    pub format: HistoryFormat,
}

impl HistoryCommand {
    pub fn new<S>(command: S, when: i64, format: HistoryFormat) -> Self
    where
        S: Into<String>,
    {
        Self {
            command: command.into(),
            when,
            format,
        }
    }
}

impl fmt::Display for HistoryCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            HistoryFormat::Bash => write!(f, "{}", self.command),
            HistoryFormat::Zsh { extended_history } => {
                if extended_history {
                    write!(f, ": {}:0;{}", self.when, self.command)
                } else {
                    write!(f, "{}", self.command)
                }
            }
            HistoryFormat::Fish => writeln!(f, "- cmd: {}\n  when: {}", self.command, self.when),
            HistoryFormat::Nu => write!(f, "{}", self.command.replace('\n', NU_NEWLINE_ESCAPE)),
        }
    }
}

#[must_use]
pub fn full_history(path: &Path, history_format: HistoryFormat) -> Vec<HistoryCommand> {
    match history_format {
        HistoryFormat::Bash => {
            let history_contents = read_ignoring_utf_errors(path);
            let zsh_timestamp_and_duration_regex = Regex::new(r"^: [0-9]+:[0-9]+;").unwrap();
            let when = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
                .as_secs() as i64;
            history_contents
                .split('\n')
                .filter(|line| !has_leading_timestamp(line) && !line.is_empty())
                .map(|line| zsh_timestamp_and_duration_regex.replace(line, ""))
                .map(|line| HistoryCommand::new(line, when, history_format))
                .collect()
        }
        HistoryFormat::Zsh { .. } => {
            let history_contents = read_and_unmetafy(path);
            let zsh_timestamp_and_duration_regex = Regex::new(r"^: [0-9]+:[0-9]+;").unwrap();
            let when = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
                .as_secs() as i64;
            history_contents
                .split('\n')
                .filter(|line| !has_leading_timestamp(line) && !line.is_empty())
                .map(|line| zsh_timestamp_and_duration_regex.replace(line, ""))
                .map(|line| HistoryCommand::new(line, when, history_format))
                .collect()
        }
        HistoryFormat::Fish => {
            // Fish history format is not technically YAML.  This is a naive parser of the format,
            // only caring about command strings (which are always on one line, with embedded
            // newlines) and timestamps, ignoring the 'paths' field.
            let mut commands = Vec::new();

            let history_contents = read_ignoring_utf_errors(path);

            // Store command strings, and add them as HistoryCommand when we see the timestamp.
            let mut command = None;
            for line in history_contents.split('\n') {
                if line.starts_with("- cmd: ") {
                    command = Some(line.split_at(7).1);
                } else if line.starts_with("  when: ") {
                    let when_str = line.split_at(8).1;
                    let when =
                        i64::from_str(when_str).unwrap_or_else(|e| panic!("McFly error: fish history '{}' has 'when' that's not a valid i64 ({}) - {}", path.display(), when_str, e));
                    // Remove (take) the command string, restarting our state machine.
                    commands.push(HistoryCommand::new(
                        command.take().unwrap_or_else(|| panic!("McFly error: invalid fish history file '{}', found 'when' without 'cmd' ({})", path.display(), when)),
                        when,
                        history_format,
                    ));
                } // ignore other lines, like 'paths' lists
            }

            commands
        }
        HistoryFormat::Nu => {
            if is_sqlite_file(path) {
                return nu_sqlite_history(path);
            }

            let history_contents = read_ignoring_utf_errors(path);
            let when = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
                .as_secs() as i64;
            history_contents
                .split('\n')
                .filter(|line| !line.is_empty())
                .map(|line| line.replace(NU_NEWLINE_ESCAPE, "\n"))
                .map(|line| HistoryCommand::new(line, when, history_format))
                .collect()
        }
    }
}

fn is_sqlite_file(path: &Path) -> bool {
    let mut header = [0; 16];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok()
        && &header == b"SQLite format 3\0"
}

// Reads Nushell's history when `$env.config.history.file_format` is "sqlite".
fn nu_sqlite_history(path: &Path) -> Vec<HistoryCommand> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .unwrap_or_else(|err| panic!("McFly error: Unable to open {path:?} ({err})"));
    let mut statement = connection
        .prepare("SELECT command_line, start_timestamp FROM history ORDER BY id")
        .unwrap_or_else(|err| panic!("McFly error: Unable to read Nushell history ({err})"));
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
        .as_secs() as i64;
    statement
        .query_map([], |row| {
            let command: String = row.get(0)?;
            // Nushell stores timestamps in milliseconds.
            let when: Option<i64> = row.get(1)?;
            Ok(HistoryCommand::new(
                command,
                when.map_or(now, |ms| ms / 1000),
                HistoryFormat::Nu,
            ))
        })
        .unwrap_or_else(|err| panic!("McFly error: Unable to read Nushell history ({err})"))
        .filter_map(Result::ok)
        .collect()
}

#[must_use]
pub fn last_history_line(path: &Path, history_format: HistoryFormat) -> Option<String> {
    // Could switch to https://github.com/mikeycgto/rev_lines
    full_history(path, history_format)
        .last()
        .map(|s| s.command.trim().to_string())
}

pub fn delete_last_history_entry_if_search(
    path: &Path,
    history_format: HistoryFormat,
    debug: bool,
) {
    let mut commands = full_history(path, history_format);

    if !commands.is_empty() && commands[commands.len() - 1].command.is_empty() {
        commands.pop();
    }

    let starts_with_mcfly = Regex::new(r"^(: [0-9]+:[0-9]+;)?#mcfly:").unwrap();

    if commands.is_empty() || !starts_with_mcfly.is_match(&commands[commands.len() - 1].command) {
        return; // Abort if empty or the last line isn't a comment.
    }

    if debug {
        println!(
            "McFly: Removed from file '{}': {:?}",
            path.display(),
            commands.pop()
        );
    } else {
        commands.pop();
    }

    if !commands.is_empty() && has_leading_timestamp(&commands[commands.len() - 1].command) {
        commands.pop();
    }

    let lines = commands
        .into_iter()
        .map(|cmd| cmd.to_string())
        // Newline at end of file.
        .chain(Some(String::new()))
        .collect::<Vec<String>>();

    fs::write(path, lines.join("\n"))
        .unwrap_or_else(|_| panic!("McFly error: Unable to update {:?}", path));
}

pub fn delete_lines(path: &Path, history_format: HistoryFormat, command: &str) {
    let commands = full_history(path, history_format);

    let zsh_timestamp_and_duration_regex = Regex::new(r"^: [0-9]+:[0-9]+;").unwrap();

    let lines = commands
        .into_iter()
        .filter(|cmd| !command.eq(&zsh_timestamp_and_duration_regex.replace(&cmd.command, "")))
        .map(|cmd| cmd.to_string())
        // Newline at end of file.
        .chain(Some(String::new()))
        .collect::<Vec<String>>();

    fs::write(path, lines.join("\n"))
        .unwrap_or_else(|_| panic!("McFly error: Unable to update {:?}", path));
}

pub fn append_history_entry(command: &HistoryCommand, path: &Path, debug: bool) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap_or_else(|err| {
            panic!(
                "McFly error: please make sure the specified --append-to-histfile file ({path:?}) exists ({err})"
            )
        });

    if debug {
        println!("McFly: Appended to file '{:?}': {}", path, command);
    }

    if let Err(e) = writeln!(file, "{command}") {
        eprintln!("Couldn't append to file '{}': {}", path.display(), e);
    }
}

#[cfg(test)]
mod tests {
    use super::{HistoryCommand, full_history, has_leading_timestamp};
    use crate::settings::HistoryFormat;
    use rusqlite::Connection;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("mcfly-test-{}-{}", std::process::id(), name))
    }

    #[test]
    fn has_leading_timestamp_works() {
        assert!(!has_leading_timestamp("abc"));
        assert!(!has_leading_timestamp("#abc"));
        assert!(!has_leading_timestamp("#123456"));
        assert!(has_leading_timestamp("#1234567890"));
        assert!(!has_leading_timestamp("#123456789"));
        assert!(!has_leading_timestamp("# 1234567890"));
        assert!(!has_leading_timestamp("1234567890"));
        assert!(!has_leading_timestamp("hello 1234567890"));
    }

    #[test]
    fn full_history_reads_nu_plaintext() {
        let path = temp_path("nu-history.txt");
        fs::write(
            &path,
            "ls\nfor x in [1 2] {<\\n>  print $x<\\n>}\n\ncd ~/src\n",
        )
        .unwrap();

        let commands: Vec<String> = full_history(&path, HistoryFormat::Nu)
            .into_iter()
            .map(|c| c.command)
            .collect();
        fs::remove_file(&path).unwrap();

        assert_eq!(
            commands,
            vec!["ls", "for x in [1 2] {\n  print $x\n}", "cd ~/src"]
        );
    }

    #[test]
    fn full_history_reads_nu_sqlite() {
        let path = temp_path("nu-history.sqlite3");
        let _ = fs::remove_file(&path);
        {
            // The table as created by Nushell's line editor (reedline).
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE history (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        command_line TEXT NOT NULL,
                        start_timestamp INTEGER,
                        session_id INTEGER,
                        hostname TEXT,
                        cwd TEXT,
                        duration_ms INTEGER,
                        exit_status INTEGER,
                        more_info TEXT
                    ) STRICT;
                    INSERT INTO history (command_line, start_timestamp) VALUES ('ls', 1700000000123);
                    INSERT INTO history (command_line, start_timestamp) VALUES ('cargo build', NULL);",
                )
                .unwrap();
        }

        let commands = full_history(&path, HistoryFormat::Nu);
        fs::remove_file(&path).unwrap();

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "ls");
        assert_eq!(commands[0].when, 1700000000);
        assert_eq!(commands[1].command, "cargo build");
        assert!(commands[1].when > 1700000000);
    }

    #[test]
    fn nu_history_command_escapes_newlines() {
        let command = HistoryCommand::new("if true {\n  ls\n}", 0, HistoryFormat::Nu);
        assert_eq!(command.to_string(), "if true {<\\n>  ls<\\n>}");
    }
}
