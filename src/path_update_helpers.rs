use relative_path::RelativePath;
use shellexpand;
use std::env;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use std::env;
    use super::{normalize_path, update_path};

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
        assert_eq!(normalize_path("~/foo/bar/../baz"), String::from(env::var("HOME").unwrap()) + "/foo/baz");
        assert_eq!(normalize_path("~/foo/bar/../.."), String::from(env::var("HOME").unwrap()));
    }

    #[test]
    fn update_path_works() {
        assert_eq!(update_path("/foo/bar", "/foo/bar", "/bar"), String::from("/bar"));
        assert_eq!(update_path("/foo/bar", "/foo/bar", "/blah"), String::from("/blah"));
        assert_eq!(update_path("/foo/bar", "/foo/bar", "/"), String::from("/"));
        assert_eq!(update_path("/foo/bar/baz/bing", "/foo/bar", "/bar"), String::from("/bar/baz/bing"));
        assert_eq!(update_path("/foo/bar/baz/bing", "/foo/bar", "/foo/blah"), String::from("/foo/blah/baz/bing"));
        assert_eq!(update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play", "/Users/joe/projects/oss"), String::from("/Users/joe/projects/oss/rust/mcfly"));
        assert_eq!(update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play", "/Users/joe/play"), String::from("/Users/joe/play/rust/mcfly"));
        assert_eq!(update_path("/Users/joe/projects/play/rust/mcfly", "/Users/joe/projects/play/rust", "/Users/joe/rust"), String::from("/Users/joe/rust/mcfly"));
    }
}
