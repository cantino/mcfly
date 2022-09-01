use std::path::Path;
use relative_path::RelativePath;
use std::env;
use std::path::{Path, PathBuf};
use unicode_segmentation::UnicodeSegmentation;
use path_absolutize::*;

pub fn normalize_path(incoming_path: &str) -> String {
    return Path::new(incoming_path).absolutize_from(std::env::current_dir().unwrap().as_path()).unwrap().to_str().unwrap().to_string();
}

pub fn parse_mv_command(command: &str) -> Vec<String> {
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut escaped = false;
    let mut buffer = String::new();
    let mut result: Vec<String> = Vec::new();

    for grapheme in command.graphemes(true) {
        match grapheme {
            "\\" => {
                escaped = true;
            }
            "\"" => {
                if escaped {
                    escaped = false;
                    buffer.push_str(grapheme);
                } else if in_double_quote {
                    in_double_quote = false;
                    if !buffer.is_empty() {
                        result.push(buffer);
                    }
                    buffer = String::new();
                } else if !in_single_quote {
                    in_double_quote = true;
                } else {
                    buffer.push_str(grapheme);
                }
            }
            "\'" => {
                if in_single_quote {
                    in_single_quote = false;
                    if !buffer.is_empty() {
                        result.push(buffer);
                    }
                    buffer = String::new();
                } else if !in_double_quote {
                    in_single_quote = true;
                } else {
                    buffer.push_str(grapheme);
                }
                escaped = false;
            }
            " " => {
                if in_double_quote || in_single_quote || escaped {
                    buffer.push_str(grapheme);
                } else {
                    if !buffer.is_empty() {
                        result.push(buffer);
                    }
                    buffer = String::new();
                }
                escaped = false;
            }
            _ => {
                buffer.push_str(grapheme);
                escaped = false;
            }
        }
    }

    if !buffer.is_empty() {
        result.push(buffer);
    }

    result
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with('-'))
        .map(|s| s.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{normalize_path, parse_mv_command};
    use std::env;
    use std::path::PathBuf;
    use std::path::Path;

    #[test]
    fn parse_mv_command_works_in_the_basic_case() {
        assert_eq!(
            parse_mv_command("mv foo bar"),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn parse_mv_command_works_with_options() {
        assert_eq!(
            parse_mv_command("mv -v foo bar"),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn parse_mv_command_works_with_escaped_strings() {
        assert_eq!(
            parse_mv_command("mv \"foo baz\" 'bar bing'"),
            vec!["foo baz".to_string(), "bar bing".to_string()]
        );
        assert_eq!(
            parse_mv_command("mv -v \"foo\" 'bar'"),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn parse_mv_command_works_with_escaping() {
        assert_eq!(
            parse_mv_command("mv \\foo bar"),
            vec!["foo".to_string(), "bar".to_string()]
        );
        assert_eq!(
            parse_mv_command("mv foo\\ bar bing"),
            vec!["foo bar".to_string(), "bing".to_string()]
        );
        assert_eq!(
            parse_mv_command("mv \"foo\\ bar\" bing"),
            vec!["foo bar".to_string(), "bing".to_string()]
        );
        assert_eq!(
            parse_mv_command("mv \"'foo\\' bar\" bing"),
            vec!["'foo' bar".to_string(), "bing".to_string()]
        );
        assert_eq!(
            parse_mv_command("mv \"\\\"foo\" bar"),
            vec!["\"foo".to_string(), "bar".to_string()]
        );
    }
}
