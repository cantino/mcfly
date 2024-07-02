use crate::settings::pwd;
use path_absolutize::Absolutize;
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[must_use]
pub fn normalize_path(incoming_path: &str) -> String {
    let expanded_path = shellexpand::tilde(incoming_path).to_string();
    return Path::new(&expanded_path)
        .absolutize_from(pwd())
        .unwrap()
        .to_str()
        .unwrap_or_else(|| panic!("McFly error: Path must be a valid UTF8 string"))
        .to_string();
}

#[must_use]
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
        .map(std::borrow::ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{normalize_path, parse_mv_command};
    use std::env;
    use std::path::PathBuf;

    #[test]
    #[cfg(not(windows))]
    fn normalize_path_works_absolute_paths() {
        assert_eq!(normalize_path("/foo/bar/baz"), String::from("/foo/bar/baz"));
        assert_eq!(normalize_path("/"), String::from("/"));
        assert_eq!(normalize_path("////"), String::from("/"));
    }

    #[test]
    #[cfg(not(windows))]
    fn normalize_path_works_with_tilda() {
        assert_eq!(normalize_path("~/"), env::var("HOME").unwrap());
        assert_eq!(
            normalize_path("~/foo"),
            PathBuf::from(env::var("HOME").unwrap())
                .join("foo")
                .to_string_lossy()
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn normalize_path_works_with_double_dots() {
        assert_eq!(normalize_path("/foo/bar/../baz"), String::from("/foo/baz"));
        assert_eq!(normalize_path("/foo/bar/../../baz"), String::from("/baz"));
        assert_eq!(normalize_path("/foo/bar/../../"), String::from("/"));
        assert_eq!(normalize_path("/foo/bar/../.."), String::from("/"));
        assert_eq!(
            normalize_path("~/foo/bar/../baz"),
            PathBuf::from(env::var("HOME").unwrap())
                .join("foo/baz")
                .to_string_lossy()
        );
        assert_eq!(normalize_path("~/foo/bar/../.."), env::var("HOME").unwrap());
    }

    #[cfg(windows)]
    fn windows_home_path() -> String {
        PathBuf::from(env::var("HOMEDRIVE").unwrap())
            .join(env::var("HOMEPATH").unwrap())
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    #[cfg(windows)]
    fn normalize_path_works_absolute_paths() {
        assert_eq!(
            normalize_path("C:\\foo\\bar\\baz"),
            String::from("C:\\foo\\bar\\baz")
        );
        assert_eq!(normalize_path("C:\\"), String::from("C:\\"));
        assert_eq!(normalize_path("C:\\\\\\\\"), String::from("C:\\"));
    }

    #[test]
    #[cfg(windows)]
    fn normalize_path_works_with_tilda() {
        assert_eq!(normalize_path("~\\"), windows_home_path());
        assert_eq!(
            normalize_path("~\\foo"),
            PathBuf::from(windows_home_path())
                .join("foo")
                .to_string_lossy()
        );
    }

    #[test]
    #[cfg(windows)]
    fn normalize_path_works_with_double_dots() {
        assert_eq!(
            normalize_path("C:\\foo\\bar\\..\\baz"),
            String::from("C:\\foo\\baz")
        );
        assert_eq!(
            normalize_path("C:\\foo\\bar\\..\\..\\baz"),
            String::from("C:\\baz")
        );
        assert_eq!(
            normalize_path("C:\\foo\\bar\\..\\..\\"),
            String::from("C:\\")
        );
        assert_eq!(normalize_path("C:\\foo\\bar\\..\\.."), String::from("C:\\"));
        assert_eq!(
            normalize_path("~\\foo\\bar\\..\\baz"),
            PathBuf::from(windows_home_path())
                .join("foo\\baz")
                .to_string_lossy()
        );
        assert_eq!(normalize_path("~\\foo\\bar\\..\\.."), windows_home_path());
    }

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
