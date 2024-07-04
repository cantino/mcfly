use unicode_segmentation::UnicodeSegmentation;

const TRUNCATE_TO_N_TOKENS: u16 = 2;

#[derive(Debug)]
pub struct SimplifiedCommand {
    pub original: String,
    pub result: String,
    pub truncate: bool,
}

#[allow(clippy::collapsible_if)]
/// The goal of `SimplifiedCommand` is to produce a reduced approximation of the given command for template matching. It may
/// not produce an exact simplification. (For example, it does not handle deeply nested escaping, and it drops escape characters.)
/// Possible enhancements:
/// - Sort and expand command line options.
/// - Check to see if unknown strings represent valid local paths in the directory where the command was run.
impl SimplifiedCommand {
    pub fn new<S: Into<String>>(command: S, truncate: bool) -> SimplifiedCommand {
        let mut simplified_command = SimplifiedCommand {
            original: command.into(),
            result: String::new(),
            truncate,
        };
        simplified_command.simplify();
        simplified_command
    }

    fn simplify(&mut self) {
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        let mut escaped = false;
        let mut buffer = String::new();
        let mut tokens = 0;

        for grapheme in self.original.graphemes(true) {
            match grapheme {
                "\\" => {
                    escaped = true;
                }
                "\"" => {
                    if escaped {
                        escaped = false;
                    } else if in_double_quote {
                        in_double_quote = false;
                        self.result.push_str("QUOTED");
                    } else if !in_single_quote {
                        in_double_quote = true;
                    }
                }
                "\'" => {
                    if in_single_quote {
                        in_single_quote = false;
                        self.result.push_str("QUOTED");
                    } else if !in_double_quote {
                        in_single_quote = true;
                    }
                    escaped = false;
                }
                " " | ":" | "," => {
                    if !in_double_quote && !in_single_quote {
                        if self.truncate && grapheme.eq(" ") {
                            tokens += 1;
                            if tokens >= TRUNCATE_TO_N_TOKENS {
                                break;
                            }
                        }

                        if !self.result.is_empty() && buffer.contains('/') {
                            self.result.push_str("PATH");
                        } else {
                            self.result.push_str(&buffer);
                        }
                        self.result.push_str(grapheme);
                        buffer.clear();
                    }
                }
                _ => {
                    if !in_double_quote && !in_single_quote {
                        buffer.push_str(grapheme);
                    }
                    escaped = false;
                }
            }
        }
        if !self.result.is_empty() && buffer.contains('/') {
            self.result.push_str("PATH");
        } else {
            self.result.push_str(&buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SimplifiedCommand;

    #[test]
    fn it_works_for_simple_commands() {
        let simplified_command = SimplifiedCommand::new("git push", false);
        assert_eq!(simplified_command.result, "git push");

        let simplified_command = SimplifiedCommand::new("git pull", false);
        assert_eq!(simplified_command.result, "git pull");

        let simplified_command = SimplifiedCommand::new("rake db:test:prepare", false);
        assert_eq!(simplified_command.result, "rake db:test:prepare");
    }

    #[test]
    fn it_simplifies_simple_quoted_strings() {
        let simplified_command = SimplifiedCommand::new("git ci -m 'my commit message'", false);
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m 'my \"commit\" message'", false);
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m \"my commit message\"", false);
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m \"my 'commit' message\"", false);
        assert_eq!(simplified_command.result, "git ci -m QUOTED");
    }

    #[test]
    fn it_handles_one_level_of_quote_escaping() {
        let simplified_command =
            SimplifiedCommand::new("git ci -m \"my \\\"commit\\\" mes\\\\sage\"", false);
        assert_eq!(simplified_command.result, "git ci -m QUOTED");
    }

    #[test]
    fn it_ignores_escaping_otherwise() {
        let simplified_command = SimplifiedCommand::new("git ci -m \\foo\\", false);
        assert_eq!(simplified_command.result, "git ci -m foo");
    }

    #[test]
    fn it_simplifies_obvious_paths() {
        let simplified_command = SimplifiedCommand::new("ls /", false);
        assert_eq!(simplified_command.result, "ls PATH");

        let simplified_command = SimplifiedCommand::new("cd ../foo", false);
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("cd foo/", false);
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("cd ..", false);
        assert_eq!(simplified_command.result, "cd ..");

        let simplified_command = SimplifiedCommand::new("cd foo/bar/baz", false);
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("command path/1/2/3:/foo/bar", false);
        assert_eq!(simplified_command.result, "command PATH:PATH");

        let simplified_command =
            SimplifiedCommand::new("blah --input foo/bar/baz --output blarg", false);
        assert_eq!(
            simplified_command.result,
            "blah --input PATH --output blarg"
        );

        let simplified_command = SimplifiedCommand::new("cd ambiguous", false);
        assert_eq!(simplified_command.result, "cd ambiguous");
    }

    #[test]
    fn it_ignores_leading_paths() {
        let simplified_command = SimplifiedCommand::new("../ls /", false);
        assert_eq!(simplified_command.result, "../ls PATH");

        let simplified_command = SimplifiedCommand::new("./cd ../foo", false);
        assert_eq!(simplified_command.result, "./cd PATH");

        let simplified_command = SimplifiedCommand::new("/bin/cd foo/", false);
        assert_eq!(simplified_command.result, "/bin/cd PATH");
    }

    #[test]
    fn it_truncates_after_simplification() {
        let simplified_command = SimplifiedCommand::new("../ls /", true);
        assert_eq!(simplified_command.result, "../ls PATH");

        let simplified_command =
            SimplifiedCommand::new("blah --input foo/bar/baz --output blarg", true);
        assert_eq!(simplified_command.result, "blah --input");

        let simplified_command =
            SimplifiedCommand::new("git ci -m \"my \\\"commit\\\" mes\\\\sage\"", true);
        assert_eq!(simplified_command.result, "git ci");
    }

    //    #[test]
    //    fn it_sorts_and_expands_command_line_arguments() {
    //        let simplified_command = SimplifiedCommand::new("ls -t 2 -lah --foo bar --baz=bing");
    //        assert_eq!(simplified_command.result, "ls -a --baz=bing --foo bar -h -l -t 2");
    //
    //        let simplified_command = SimplifiedCommand::new("ls -l --foo bar -a -h --bazz=bing -t 2");
    //        assert_eq!(simplified_command.result, "ls -a --baz=bing --foo bar -h -l -t 2");
    //    }
}
