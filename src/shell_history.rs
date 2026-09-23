use crate::settings::HistoryFormat;
use regex::Regex;
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
    /// Whether this entry had an explicit leading timestamp in bash history.
    pub has_timestamp: bool,
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
            has_timestamp: false,
        }
    }

    pub fn with_timestamp<S>(
        command: S,
        when: i64,
        format: HistoryFormat,
        has_timestamp: bool,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            command: command.into(),
            when,
            format,
            has_timestamp,
        }
    }
}

impl fmt::Display for HistoryCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            HistoryFormat::Bash => {
                if self.has_timestamp {
                    write!(f, "#{}\n{}", self.when, self.command)
                } else {
                    write!(f, "{}", self.command)
                }
            }
            HistoryFormat::Zsh { extended_history } => {
                let escaped_cmd = self.command.replace('\n', "\\\n");
                if extended_history {
                    write!(f, ": {}:0;{}", self.when, escaped_cmd)
                } else {
                    write!(f, "{}", escaped_cmd)
                }
            }
            HistoryFormat::Fish => writeln!(f, "- cmd: {}\n  when: {}", self.command, self.when),
        }
    }
}

#[must_use]
pub fn full_history(path: &Path, history_format: HistoryFormat) -> Vec<HistoryCommand> {
    match history_format {
        HistoryFormat::Bash => {
            let history_contents = read_ignoring_utf_errors(path);
            let zsh_timestamp_and_duration_regex = Regex::new(r"^: [0-9]+:[0-9]+;").unwrap();
            let default_when = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
                .as_secs() as i64;

            let mut commands = Vec::new();
            let mut current_command = String::new();
            let mut current_when = default_when;
            let mut current_has_timestamp = false;
            let mut in_timestamped_entry = false;

            for line in history_contents.split('\n') {
                if has_leading_timestamp(line) {
                    if !current_command.is_empty() {
                        let cleaned = zsh_timestamp_and_duration_regex
                            .replace(&std::mem::take(&mut current_command), "")
                            .into_owned();
                        commands.push(HistoryCommand::with_timestamp(
                            cleaned,
                            current_when,
                            history_format,
                            current_has_timestamp,
                        ));
                    }
                    in_timestamped_entry = true;
                    current_has_timestamp = true;
                    current_when = line[1..].trim().parse::<i64>().unwrap_or(default_when);
                } else if line.starts_with("#mcfly:") {
                    if !current_command.is_empty() {
                        let cleaned = zsh_timestamp_and_duration_regex
                            .replace(&std::mem::take(&mut current_command), "")
                            .into_owned();
                        commands.push(HistoryCommand::with_timestamp(
                            cleaned,
                            current_when,
                            history_format,
                            current_has_timestamp,
                        ));
                    }
                    commands.push(HistoryCommand::with_timestamp(
                        line,
                        default_when,
                        history_format,
                        false,
                    ));
                    in_timestamped_entry = false;
                    current_has_timestamp = false;
                    current_when = default_when;
                } else if in_timestamped_entry {
                    if !line.is_empty() {
                        if !current_command.is_empty() {
                            current_command.push('\n');
                        }
                        current_command.push_str(line);
                    }
                } else if !line.is_empty() {
                    let cleaned = zsh_timestamp_and_duration_regex
                        .replace(line, "")
                        .into_owned();
                    commands.push(HistoryCommand::with_timestamp(
                        cleaned,
                        default_when,
                        history_format,
                        false,
                    ));
                }
            }

            if !current_command.is_empty() {
                let cleaned = zsh_timestamp_and_duration_regex
                    .replace(&std::mem::take(&mut current_command), "")
                    .into_owned();
                commands.push(HistoryCommand::with_timestamp(
                    cleaned,
                    current_when,
                    history_format,
                    current_has_timestamp,
                ));
            }

            commands
        }
        HistoryFormat::Zsh { extended_history } => {
            let history_contents = read_and_unmetafy(path);
            let zsh_timestamp_and_duration_regex = Regex::new(r"^: [0-9]+:[0-9]+;").unwrap();
            let zsh_when_regex = Regex::new(r"^: ([0-9]+):[0-9]+;").unwrap();
            let default_when = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({err})"))
                .as_secs() as i64;

            let mut commands = Vec::new();
            let mut current_command = String::new();
            let mut current_when = default_when;

            for line in history_contents.split('\n') {
                if extended_history && zsh_timestamp_and_duration_regex.is_match(line) {
                    if !current_command.is_empty() {
                        let cleaned = zsh_timestamp_and_duration_regex
                            .replace(&std::mem::take(&mut current_command), "")
                            .into_owned();
                        commands.push(HistoryCommand::new(cleaned, current_when, history_format));
                    }
                    if let Some(matched) =
                        zsh_when_regex.captures(line).and_then(|caps| caps.get(1))
                    {
                        current_when = matched.as_str().parse::<i64>().unwrap_or(default_when);
                    }
                    current_command.push_str(line);
                } else if !current_command.is_empty() && current_command.ends_with('\\') {
                    current_command.pop();
                    current_command.push('\n');
                    current_command.push_str(line);
                } else if !line.is_empty() {
                    if !current_command.is_empty() {
                        let cleaned = zsh_timestamp_and_duration_regex
                            .replace(&std::mem::take(&mut current_command), "")
                            .into_owned();
                        commands.push(HistoryCommand::new(cleaned, current_when, history_format));
                        current_when = default_when;
                    }
                    current_command.push_str(line);
                }
            }
            if !current_command.is_empty() {
                let cleaned = zsh_timestamp_and_duration_regex
                    .replace(&std::mem::take(&mut current_command), "")
                    .into_owned();
                commands.push(HistoryCommand::new(cleaned, current_when, history_format));
            }
            commands
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
    }
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
    use super::{delete_lines, full_history, has_leading_timestamp};
    use crate::settings::HistoryFormat;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempTestFile {
        path: PathBuf,
    }

    impl TempTestFile {
        fn new(name: &str, content: &str) -> Self {
            let counter = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let filename = format!("mcfly_test_{}_{}_{}.tmp", std::process::id(), name, counter);
            let path = std::env::temp_dir().join(filename);
            fs::write(&path, content).expect("Failed to write temporary test history file");
            Self { path }
        }

        fn from_bytes(name: &str, bytes: &[u8]) -> Self {
            let counter = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let filename = format!("mcfly_test_{}_{}_{}.tmp", std::process::id(), name, counter);
            let path = std::env::temp_dir().join(filename);
            fs::write(&path, bytes).expect("Failed to write temporary test history file bytes");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
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
    fn test_bash_multiline_with_timestamps() {
        let content = "\
#1593624164
echo \"single line\"
#1593624165
for i in {1..3}; do
  echo \"number: $i\"
done
#1593624166
git commit -m \"feat: multiline commit
This is the body of the commit message.
Signed-off-by: Developer <dev@example.com>\"
#1593624167
cargo test
";
        let temp = TempTestFile::new("bash_multiline", content);
        let commands = full_history(temp.path(), HistoryFormat::Bash);

        assert_eq!(commands.len(), 4);
        assert_eq!(commands[0].command, "echo \"single line\"");
        assert_eq!(
            commands[1].command,
            "for i in {1..3}; do\n  echo \"number: $i\"\ndone"
        );
        assert_eq!(
            commands[2].command,
            "git commit -m \"feat: multiline commit\nThis is the body of the commit message.\nSigned-off-by: Developer <dev@example.com>\""
        );
        assert_eq!(commands[3].command, "cargo test");
    }

    #[test]
    fn test_bash_history_without_timestamps() {
        let content = "\
echo foo
cargo build
cargo test
";
        let temp = TempTestFile::new("bash_no_ts", content);
        let commands = full_history(temp.path(), HistoryFormat::Bash);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].command, "echo foo");
        assert_eq!(commands[1].command, "cargo build");
        assert_eq!(commands[2].command, "cargo test");
    }

    #[test]
    fn test_zsh_extended_multiline_commands() {
        let content = "\
: 1593624160:0;echo \"first command\"
: 1593624161:0;echo \"hello\\
world\"
: 1593624162:0;echo foo \\
 bar
: 1593624163:0;curl -X POST \\
  -H \"Content-Type: application/json\" \\
  -d '{\"key\": \"value\"}'
: 1593624164:0;git status
";
        let temp = TempTestFile::new("zsh_ext_multiline", content);
        let commands = full_history(
            temp.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
        );

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0].command, "echo \"first command\"");
        assert_eq!(commands[1].command, "echo \"hello\nworld\"");
        assert_eq!(commands[2].command, "echo foo \n bar");
        assert_eq!(
            commands[3].command,
            "curl -X POST \n  -H \"Content-Type: application/json\" \n  -d '{\"key\": \"value\"}'"
        );
        assert_eq!(commands[4].command, "git status");
    }

    #[test]
    fn test_zsh_standard_multiline_commands() {
        let content = "\
echo \"first command\"
echo \"hello\\
world\"
echo foo \\
 bar
git status
";
        let temp = TempTestFile::new("zsh_std_multiline", content);
        let commands = full_history(
            temp.path(),
            HistoryFormat::Zsh {
                extended_history: false,
            },
        );

        assert_eq!(commands.len(), 4);
        assert_eq!(commands[0].command, "echo \"first command\"");
        assert_eq!(commands[1].command, "echo \"hello\nworld\"");
        assert_eq!(commands[2].command, "echo foo \n bar");
        assert_eq!(commands[3].command, "git status");
    }

    #[test]
    fn test_empty_history_file() {
        let temp = TempTestFile::new("empty_hist", "\n\n");
        let bash_commands = full_history(temp.path(), HistoryFormat::Bash);
        assert!(bash_commands.is_empty());

        let zsh_commands = full_history(
            temp.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
        );
        assert!(zsh_commands.is_empty());
    }

    #[test]
    fn test_zsh_unmetafying_in_history() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b": 100:0;echo a");
        bytes.push(0x83);
        bytes.push(b'b' ^ 32);
        bytes.push(b'\n');

        let temp = TempTestFile::from_bytes("zsh_meta", &bytes);
        let commands = full_history(
            temp.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
        );

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command, "echo ab");
    }

    #[test]
    fn test_delete_lines_multiline_command() {
        let content = "\
#1593624164
echo \"single line\"
#1593624165
for i in {1..3}; do
  echo \"number: $i\"
done
#1593624166
cargo test
";
        let temp = TempTestFile::new("bash_delete_multiline", content);
        let multiline_cmd = "for i in {1..3}; do\n  echo \"number: $i\"\ndone";

        delete_lines(temp.path(), HistoryFormat::Bash, multiline_cmd);

        let commands = full_history(temp.path(), HistoryFormat::Bash);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "echo \"single line\"");
        assert_eq!(commands[1].command, "cargo test");
    }

    #[test]
    fn test_bash_history_with_mcfly_marker() {
        use super::{delete_last_history_entry_if_search, last_history_line};

        let content = "\
#1593624164
cargo build
#mcfly: carg
";
        let temp = TempTestFile::new("bash_mcfly_marker", content);

        // Verify last_history_line returns the marker, not glued onto cargo build
        let last = last_history_line(temp.path(), HistoryFormat::Bash);
        assert_eq!(last, Some("#mcfly: carg".to_string()));

        let commands = full_history(temp.path(), HistoryFormat::Bash);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "cargo build");
        assert!(commands[0].has_timestamp);
        assert_eq!(commands[0].when, 1593624164);
        assert_eq!(commands[1].command, "#mcfly: carg");

        // Verify delete_last_history_entry_if_search cleans up the marker without touching cargo build
        delete_last_history_entry_if_search(temp.path(), HistoryFormat::Bash, false);

        let remaining = full_history(temp.path(), HistoryFormat::Bash);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].command, "cargo build");
        assert!(remaining[0].has_timestamp);
        assert_eq!(remaining[0].when, 1593624164);

        // Verify file content has timestamp and cargo build
        let written = fs::read_to_string(temp.path()).unwrap();
        assert_eq!(written, "#1593624164\ncargo build\n");
    }

    #[test]
    fn test_bash_mixed_timestamp_history() {
        let mut content = String::new();
        for i in 1..=10 {
            content.push_str(&format!("old_cmd_{}\n", i));
        }
        content.push_str("#1593624164\necho hi\n");

        let temp = TempTestFile::new("bash_mixed", &content);
        let commands = full_history(temp.path(), HistoryFormat::Bash);

        assert_eq!(commands.len(), 11);
        for i in 1..=10 {
            assert_eq!(commands[i - 1].command, format!("old_cmd_{}", i));
            assert!(!commands[i - 1].has_timestamp);
        }
        assert_eq!(commands[10].command, "echo hi");
        assert!(commands[10].has_timestamp);
        assert_eq!(commands[10].when, 1593624164);
    }

    #[test]
    fn test_writer_symmetry_bash_and_zsh() {
        // Bash symmetry test
        let bash_content = "\
#1593624164
echo \"first\"
#1593624165
for i in {1..3}; do
  echo \"number: $i\"
done
#1593624166
cargo test
";
        let temp_bash = TempTestFile::new("bash_symmetry", bash_content);
        delete_lines(temp_bash.path(), HistoryFormat::Bash, "echo \"first\"");

        let raw_bash = fs::read_to_string(temp_bash.path()).unwrap();
        assert!(raw_bash.contains("#1593624165\nfor i in {1..3}; do\n  echo \"number: $i\"\ndone"));
        assert!(raw_bash.contains("#1593624166\ncargo test"));

        let bash_reloaded = full_history(temp_bash.path(), HistoryFormat::Bash);
        assert_eq!(bash_reloaded.len(), 2);
        assert_eq!(
            bash_reloaded[0].command,
            "for i in {1..3}; do\n  echo \"number: $i\"\ndone"
        );
        assert_eq!(bash_reloaded[1].command, "cargo test");

        // Zsh extended symmetry test
        let zsh_content = "\
: 1593624160:0;echo \"first\"
: 1593624161:0;echo \"hello\\
world\"
: 1593624162:0;cargo test
";
        let temp_zsh = TempTestFile::new("zsh_symmetry", zsh_content);
        delete_lines(
            temp_zsh.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
            "echo \"first\"",
        );

        let raw_zsh = fs::read_to_string(temp_zsh.path()).unwrap();
        assert!(raw_zsh.contains(": 1593624161:0;echo \"hello\\\nworld\""));

        let zsh_reloaded = full_history(
            temp_zsh.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
        );
        assert_eq!(zsh_reloaded.len(), 2);
        assert_eq!(zsh_reloaded[0].command, "echo \"hello\nworld\"");
        assert_eq!(zsh_reloaded[1].command, "cargo test");
    }

    #[test]
    fn test_zsh_multiline_empty_lines_and_heredocs() {
        let content = "\
: 1593624160:0;cat <<'EOF'\\
#1593624161\\
\\
final line\\
EOF
: 1593624162:0;next command
";
        let temp = TempTestFile::new("zsh_heredoc", content);
        let commands = full_history(
            temp.path(),
            HistoryFormat::Zsh {
                extended_history: true,
            },
        );

        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0].command,
            "cat <<'EOF'\n#1593624161\n\nfinal line\nEOF"
        );
        assert_eq!(commands[1].command, "next command");
    }
}
