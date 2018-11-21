use relative_path::RelativePath;
use shellexpand;
use std::env;
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

pub fn normalize_path(incoming_path: &str) -> String {
    let expanded_path = shellexpand::full(incoming_path).expect("Unable to expand path");

    let current_dir = env::var("PWD").expect("Unable to determine current directory");
    let current_dir_path = Path::new(&current_dir);

    let path_buf = if expanded_path.starts_with("/") {
        RelativePath::new(&expanded_path.into_owned()).normalize().to_path("/")
    } else {
        let to_current_dir = RelativePath::new(&expanded_path).to_path(current_dir_path);
        RelativePath::new(to_current_dir.to_str().unwrap()).normalize().to_path("/")
    };

    path_buf.to_str().expect("Path to be valid UTF8").to_string()
}

pub fn update_path(path: &str, old_path: &str, new_path: &str) -> String {
    path.replacen(old_path, new_path, 1)
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
                } else {
                    if in_double_quote {
                        in_double_quote = false;
                        if buffer.len() > 0 {
                            result.push(buffer);
                        }
                        buffer = String::new();
                    } else if !in_single_quote {
                        in_double_quote = true;
                    } else {
                        buffer.push_str(grapheme);
                    }
                }
            }
            "\'" => {
                if in_single_quote {
                    in_single_quote = false;
                    if buffer.len() > 0 {
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
                    if buffer.len() > 0 {
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

    if buffer.len() > 0 {
        result.push(buffer);
    }

    result
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with("-"))
        .map(|s| s.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::{normalize_path, update_path, parse_mv_command};

    #[test]
    fn normalize_path_works_absolute_paths() {
        assert_eq!(normalize_path("/foo/bar/baz"), String::from("/foo/bar/baz"));
        assert_eq!(normalize_path("/"), String::from("/"));
        assert_eq!(normalize_path("////"), String::from("/"));
    }

    #[test]
    fn normalize_path_works_with_tilda() {
        assert_eq!(normalize_path("~/"), String::from(env::var("HOME").unwrap()));
        assert_eq!(normalize_path("~/foo"), String::from(env::var("HOME").unwrap()) + "/foo");
    }

    #[test]
    fn normalize_path_works_with_double_dots() {
        assert_eq!(normalize_path("/foo/bar/../baz"), String::from("/foo/baz"));
        assert_eq!(normalize_path("/foo/bar/../../baz"), String::from("/baz"));
        assert_eq!(normalize_path("/foo/bar/../../"), String::from("/"));
        assert_eq!(normalize_path("/foo/bar/../.."), String::from("/"));
        assert_eq!(
            normalize_path("~/foo/bar/../baz"),
            String::from(env::var("HOME").unwrap()) + "/foo/baz"
        );
        assert_eq!(
            normalize_path("~/foo/bar/../.."),
            String::from(env::var("HOME").unwrap())
        );
    }

    #[test]
    fn update_path_works() {
        assert_eq!(
            update_path("/foo/bar", "/foo/bar", "/bar"),
            String::from("/bar")
        );
        assert_eq!(
            update_path("/foo/bar", "/foo/bar", "/blah"),
            String::from("/blah")
        );
        assert_eq!(
            update_path("/foo/bar", "/foo/bar", "/"),
            String::from("/")
        );
        assert_eq!(
            update_path("/foo/bar/baz/bing", "/foo/bar", "/bar"),
            String::from("/bar/baz/bing")
        );
        assert_eq!(
            update_path("/foo/bar/baz/bing", "/foo/bar", "/foo/blah"),
            String::from("/foo/blah/baz/bing")
        );
        assert_eq!(
            update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play", "/Users/joe/projects/oss"),
            String::from("/Users/joe/projects/oss/rust/mcfly")
        );
        assert_eq!(
            update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play", "/Users/joe/play"),
            String::from("/Users/joe/play/rust/mcfly")
        );
        assert_eq!(
            update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play/rust", "/Users/joe/rust"),
            String::from("/Users/joe/rust/mcfly")
        );
    }

    #[test]
    fn parse_mv_command_works_in_the_basic_case() {
        assert_eq!(parse_mv_command("mv foo bar"), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn parse_mv_command_works_with_options() {
        assert_eq!(parse_mv_command("mv -v foo bar"), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn parse_mv_command_works_with_escaped_strings() {
        assert_eq!(parse_mv_command("mv \"foo baz\" 'bar bing'"), vec!["foo baz".to_string(), "bar bing".to_string()]);
        assert_eq!(parse_mv_command("mv -v \"foo\" 'bar'"), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn parse_mv_command_works_with_escaping() {
        assert_eq!(parse_mv_command("mv \\foo bar"), vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(parse_mv_command("mv foo\\ bar bing"), vec!["foo bar".to_string(), "bing".to_string()]);
        assert_eq!(parse_mv_command("mv \"foo\\ bar\" bing"), vec!["foo bar".to_string(), "bing".to_string()]);
        assert_eq!(parse_mv_command("mv \"'foo\\' bar\" bing"), vec!["'foo' bar".to_string(), "bing".to_string()]);
        assert_eq!(parse_mv_command("mv \"\\\"foo\" bar"), vec!["\"foo".to_string(), "bar".to_string()]);
    }
}
