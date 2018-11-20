use history::History;
use relative_path::RelativePath;
use shellexpand;
use std::env;
use std::path::Path;
use std::path::PathBuf;

fn normalize_path(incoming_path: &str) -> PathBuf {
    let expanded_path = shellexpand::full(incoming_path).expect("Unable to expand path");

    let current_dir = env::var("PWD").expect("Unable to determine current directory");
    let current_dir_path = Path::new(&current_dir);

    if expanded_path.starts_with("/") {
        RelativePath::new(&expanded_path.into_owned()).normalize().to_path("/")
    } else {
        let to_current_dir = RelativePath::new(&expanded_path).to_path(current_dir_path);
        RelativePath::new(to_current_dir.to_str().unwrap()).normalize().to_path("/")
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::normalize_path;
    use std::path::PathBuf;

    #[test]
    fn normalize_path_works_absolute_paths() {
        assert_eq!(normalize_path("/foo/bar/baz"), PathBuf::from("/foo/bar/baz"));
        assert_eq!(normalize_path("/"), PathBuf::from("/"));
        assert_eq!(normalize_path("////"), PathBuf::from("/"));
    }

    #[test]
    fn normalize_path_works_with_tilda() {
        assert_eq!(normalize_path("~/"), PathBuf::from(env::var("HOME").unwrap()));
        assert_eq!(normalize_path("~/foo"), PathBuf::from(env::var("HOME").unwrap()).join("foo"));
    }

    #[test]
    fn normalize_path_works_with_double_dots() {
        assert_eq!(normalize_path("/foo/bar/../baz"), PathBuf::from("/foo/baz"));
        assert_eq!(normalize_path("/foo/bar/../../baz"), PathBuf::from("/baz"));
        assert_eq!(normalize_path("/foo/bar/../../"), PathBuf::from("/"));
        assert_eq!(normalize_path("/foo/bar/../.."), PathBuf::from("/"));
        assert_eq!(normalize_path("~/foo/bar/../baz"), PathBuf::from(env::var("HOME").unwrap()).join("foo/baz"));
        assert_eq!(normalize_path("~/foo/bar/../.."), PathBuf::from(env::var("HOME").unwrap()));
    }
}
