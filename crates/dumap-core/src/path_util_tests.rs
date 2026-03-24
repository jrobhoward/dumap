#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::clean_path;
use std::path::PathBuf;

#[test]
fn clean_path____unix_absolute____unchanged() {
    let path = PathBuf::from("/usr/local/bin");
    assert_eq!(clean_path(path.clone()), path);
}

#[test]
fn clean_path____relative____unchanged() {
    let path = PathBuf::from("src/main.rs");
    assert_eq!(clean_path(path.clone()), path);
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn clean_path____extended_prefix____stripped() {
        let path = PathBuf::from(r"\\?\C:\Users\test");
        assert_eq!(clean_path(path), PathBuf::from(r"C:\Users\test"));
    }

    #[test]
    fn clean_path____extended_prefix_root____stripped() {
        let path = PathBuf::from(r"\\?\C:\");
        assert_eq!(clean_path(path), PathBuf::from(r"C:\"));
    }

    #[test]
    fn clean_path____normal_windows_path____unchanged() {
        let path = PathBuf::from(r"C:\Users\test");
        assert_eq!(clean_path(path.clone()), path);
    }

    #[test]
    fn clean_path____unc_path____unchanged() {
        let path = PathBuf::from(r"\\?\UNC\server\share");
        assert_eq!(clean_path(path.clone()), path);
    }
}
