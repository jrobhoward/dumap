use std::path::PathBuf;

/// Strip the Windows extended-length path prefix (`\\?\`) if present.
///
/// `Path::canonicalize()` on Windows produces paths like `\\?\C:\Users\...`.
/// This prefix is valid but looks ugly in UI and HTML output. This function
/// strips it when the remaining path is a simple absolute path (drive letter).
pub fn clean_path(path: PathBuf) -> PathBuf {
    _clean_path(path)
}

#[cfg(windows)]
fn _clean_path(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        // Only strip if what remains looks like a normal absolute path (e.g. C:\...)
        // to avoid breaking UNC paths like \\?\UNC\server\share
        if stripped.len() >= 3 && stripped.as_bytes()[1] == b':' {
            return PathBuf::from(stripped);
        }
    }
    path
}

#[cfg(not(windows))]
fn _clean_path(path: PathBuf) -> PathBuf {
    path
}

#[cfg(test)]
#[path = "path_util_tests.rs"]
mod path_util_tests;
