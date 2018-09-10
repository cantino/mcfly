use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SimplifiedCommand {
    pub original: String,
    pub result: String
}

/// The goal of SimplifiedCommand is to produce a reduced approximation of the given command for template matching. It may
/// not produce an exact simplification. (For example, it does not handle deeply nested escaping, and it drops escape characters.)
/// Possible enhancements:
/// - Sort and expand command line options.
/// - Check to see if unknown strings represent valid local paths in the directory where the command was run.
impl SimplifiedCommand {
    pub fn new<S: Into<String>>(command: S) -> SimplifiedCommand {
        let mut simplified_command = SimplifiedCommand { original: command.into().clone(), result: String::new() };
        simplified_command.simplify();
        simplified_command
    }

    fn simplify(&mut self) {
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        let mut in_backticks = false;
        let mut escaped = false;
        let mut buffer = String::new();

        for grapheme in self.original.graphemes(true) {
            match grapheme {
                "\\" => {
                    escaped = true;
                },
                "\"" => {
                    if escaped {
                        escaped = false;
                    } else {
                        if in_double_quote {
                            in_double_quote = false;
                            self.result.push_str("QUOTED");
                        } else if !in_single_quote {
                            in_double_quote = true;
                        }
                    }
                },
                "\'" => {
                    if in_single_quote {
                        in_single_quote = false;
                        self.result.push_str("QUOTED");
                    } else if !in_double_quote {
                        in_single_quote = true;
                    }
                    escaped = false;
                },
                "`" => {
                    if escaped {
                        escaped = false;
                    } else {
                        if in_backticks {
                            in_backticks = false;
                            self.result.push_str("SUBSHELL");
                        } else if !in_double_quote && !in_single_quote {
                            in_backticks = true;
                        }
                    }
                },
                " " => {
                    if !in_double_quote && !in_single_quote && !in_backticks {
                        buffer.push_str(grapheme);
                        if self.result.len() > 0 && buffer.contains("/") {
                            self.result.push_str("PATH ");
                        } else {
                            self.result.push_str(&buffer);
                        }
                        buffer.clear();
                    }
                },
                _ => {
                    if !in_double_quote && !in_single_quote && !in_backticks {
                        buffer.push_str(grapheme);
                    }
                    escaped = false;
                }
            }
        }
        if self.result.len() > 0 && buffer.contains("/") {
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
        let simplified_command = SimplifiedCommand::new("git push");
        assert_eq!(simplified_command.result, "git push");

        let simplified_command = SimplifiedCommand::new("git pull");
        assert_eq!(simplified_command.result, "git pull");

        let simplified_command = SimplifiedCommand::new("rake db:test:prepare");
        assert_eq!(simplified_command.result, "rake db:test:prepare");
    }

    #[test]
    fn it_simplifies_simple_quoted_strings() {
        let simplified_command = SimplifiedCommand::new("git ci -m 'my commit message'");
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m 'my \"commit\" message'");
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m \"my commit message\"");
        assert_eq!(simplified_command.result, "git ci -m QUOTED");

        let simplified_command = SimplifiedCommand::new("git ci -m \"my 'commit' message\"");
        assert_eq!(simplified_command.result, "git ci -m QUOTED");
    }

    #[test]
    fn it_handles_one_level_of_quote_escaping() {
        let simplified_command = SimplifiedCommand::new("git ci -m \"my \\\"commit\\\" mes\\\\sage\"");
        assert_eq!(simplified_command.result, "git ci -m QUOTED");
    }

    #[test]
    fn it_ignores_escaping_otherwise() {
        let simplified_command = SimplifiedCommand::new("git ci -m \\foo\\");
        assert_eq!(simplified_command.result, "git ci -m foo");
    }

    #[test]
    fn it_simplifies_backticks() {
        let simplified_command = SimplifiedCommand::new("ls `which something`");
        assert_eq!(simplified_command.result, "ls SUBSHELL");

        let simplified_command = SimplifiedCommand::new("ls `echo \\`foo\\``");
        assert_eq!(simplified_command.result, "ls SUBSHELL");

        let simplified_command = SimplifiedCommand::new("ls \"\\`which something\\`\"");
        assert_eq!(simplified_command.result, "ls QUOTED");

        let simplified_command = SimplifiedCommand::new("ls \"`which something`\"");
        assert_eq!(simplified_command.result, "ls QUOTED"); // Not technically correct, Bash would run the subshell.
    }

    #[test]
    fn it_simplifies_obvious_paths() {
        let simplified_command = SimplifiedCommand::new("ls /");
        assert_eq!(simplified_command.result, "ls PATH");

        let simplified_command = SimplifiedCommand::new("cd ../foo");
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("cd foo/");
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("cd ..");
        assert_eq!(simplified_command.result, "cd ..");

        let simplified_command = SimplifiedCommand::new("cd foo/bar/baz");
        assert_eq!(simplified_command.result, "cd PATH");

        let simplified_command = SimplifiedCommand::new("blah --input foo/bar/baz --output blarg");
        assert_eq!(simplified_command.result, "blah --input PATH --output blarg");

        let simplified_command = SimplifiedCommand::new("cd ambiguous");
        assert_eq!(simplified_command.result, "cd ambiguous");
    }

    #[test]
    fn it_ignores_leading_paths() {
        let simplified_command = SimplifiedCommand::new("../ls /");
        assert_eq!(simplified_command.result, "../ls PATH");

        let simplified_command = SimplifiedCommand::new("./cd ../foo");
        assert_eq!(simplified_command.result, "./cd PATH");

        let simplified_command = SimplifiedCommand::new("/bin/cd foo/");
        assert_eq!(simplified_command.result, "/bin/cd PATH");
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
