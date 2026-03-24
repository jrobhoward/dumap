#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::walker::*;
use std::fs;
use tempfile::TempDir;

fn create_test_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("subdir/nested")).unwrap();
    fs::write(dir.path().join("file1.txt"), "hello world").unwrap();
    fs::write(dir.path().join("subdir/file2.txt"), "more content here!!").unwrap();
    fs::write(dir.path().join("subdir/nested/file3.txt"), "deep").unwrap();
    dir
}

#[test]
fn scan_directory____valid_dir____finds_all_files() {
    let dir = create_test_dir();
    let config = ScanConfig {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };
    let progress = ScanProgress::new();

    let tree = scan_directory(&config, &progress).unwrap();
    assert_eq!(tree.total_file_count(), 3);
    assert_eq!(
        progress
            .files_found
            .load(std::sync::atomic::Ordering::Relaxed),
        3
    );
}

#[test]
fn scan_directory____nonexistent_path____returns_path_not_found() {
    let config = ScanConfig {
        root: "/nonexistent/path/that/does/not/exist".into(),
        ..Default::default()
    };
    let progress = ScanProgress::new();

    let result = scan_directory(&config, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, crate::error::ScanError::PathNotFound(_)));
}

#[test]
fn scan_directory____file_not_dir____returns_not_a_directory() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("just_a_file.txt");
    fs::write(&file_path, "content").unwrap();

    let config = ScanConfig {
        root: file_path,
        ..Default::default()
    };
    let progress = ScanProgress::new();

    let result = scan_directory(&config, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, crate::error::ScanError::NotADirectory(_)));
}

#[test]
fn scan_directory____cancellation____returns_cancelled() {
    let dir = create_test_dir();
    let config = ScanConfig {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };
    let progress = ScanProgress::new();
    // Pre-cancel
    progress
        .cancelled
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let result = scan_directory(&config, &progress);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, crate::error::ScanError::Cancelled));
}

#[test]
fn scan_directory____computes_correct_sizes() {
    let dir = create_test_dir();
    let config = ScanConfig {
        root: dir.path().to_path_buf(),
        apparent_size: true,
        ..Default::default()
    };
    let progress = ScanProgress::new();

    let tree = scan_directory(&config, &progress).unwrap();
    // "hello world" = 11 bytes, "more content here!!" = 19 bytes, "deep" = 4 bytes
    assert_eq!(tree.total_size(), 11 + 19 + 4);
}

#[test]
fn scan_directory____empty_dir____returns_empty_tree() {
    let dir = TempDir::new().unwrap();
    let config = ScanConfig {
        root: dir.path().to_path_buf(),
        ..Default::default()
    };
    let progress = ScanProgress::new();

    let tree = scan_directory(&config, &progress).unwrap();
    assert_eq!(tree.total_size(), 0);
    assert_eq!(tree.total_file_count(), 0);
}

#[test]
fn format_size____various_sizes____formats_correctly() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(500), "500 B");
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(1536), "1.5 KB");
    assert_eq!(format_size(1_048_576), "1.0 MB");
    assert_eq!(format_size(1_073_741_824), "1.0 GB");
    assert_eq!(format_size(1_099_511_627_776), "1.0 TB");
}

#[cfg(unix)]
#[test]
fn scan_directory____symlink____does_not_follow() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("real.txt"), "data").unwrap();
    std::os::unix::fs::symlink(dir.path().join("real.txt"), dir.path().join("link.txt")).unwrap();

    let config = ScanConfig {
        root: dir.path().to_path_buf(),
        follow_links: false,
        apparent_size: true,
        ..Default::default()
    };
    let progress = ScanProgress::new();
    let tree = scan_directory(&config, &progress).unwrap();

    // Should only find the real file, not the symlink
    assert_eq!(tree.total_file_count(), 1);
    assert_eq!(tree.total_size(), 4);
}
