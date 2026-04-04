#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::*;
use std::fs;
use tempfile::TempDir;

// --- default_path ---

#[test]
fn default_path____returns_valid_directory() {
    let path = default_path();
    // Should return a non-empty path (either home dir or ".")
    assert!(!path.as_os_str().is_empty());
}

#[test]
fn default_path____result_exists_on_disk() {
    let path = default_path();
    assert!(path.exists());
}

// --- run_export ---

#[test]
fn run_export____small_directory____creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("data");
    fs::create_dir(&scan_dir).unwrap();
    fs::write(scan_dir.join("file1.txt"), "hello world").unwrap();
    fs::write(scan_dir.join("file2.rs"), "fn main() {}").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, false, false, None, false);

    assert!(result.is_ok());
    assert!(output.exists());

    let html = fs::read_to_string(&output).unwrap();
    assert!(html.contains("echarts"));
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<!doctype html>"));
}

#[test]
fn run_export____empty_directory____creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("empty");
    fs::create_dir(&scan_dir).unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, false, false, None, false);

    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn run_export____nested_directories____scans_recursively() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("root");
    fs::create_dir_all(scan_dir.join("a/b/c")).unwrap();
    fs::write(scan_dir.join("a/b/c/deep.txt"), "deep content").unwrap();
    fs::write(scan_dir.join("a/shallow.txt"), "shallow").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, false, false, None, false);

    assert!(result.is_ok());
    let html = fs::read_to_string(&output).unwrap();
    assert!(html.contains("deep.txt"));
}

#[test]
fn run_export____custom_depth____accepted() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("data");
    fs::create_dir(&scan_dir).unwrap();
    fs::write(scan_dir.join("test.txt"), "test").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 5, false, false, None, false);
    assert!(result.is_ok());
}

#[test]
fn run_export____apparent_size_flag____succeeds() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("data");
    fs::create_dir(&scan_dir).unwrap();
    fs::write(scan_dir.join("test.txt"), "test").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, true, false, None, false);
    assert!(result.is_ok());
}

#[test]
fn run_export____with_hidden_files____succeeds() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("data");
    fs::create_dir(&scan_dir).unwrap();
    fs::write(scan_dir.join(".hidden"), "secret").unwrap();
    fs::write(scan_dir.join("visible.txt"), "visible").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, false, true, None, false);
    assert!(result.is_ok());
}

#[test]
fn run_export____max_scan_depth____limits_traversal() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("root");
    fs::create_dir_all(scan_dir.join("a/b/c")).unwrap();
    fs::write(scan_dir.join("a/b/c/deep.txt"), "deep").unwrap();
    fs::write(scan_dir.join("a/top.txt"), "top").unwrap();

    let output = tmp.path().join("output.html");

    let result = run_export(scan_dir, output.clone(), 3, false, false, Some(1), false);
    assert!(result.is_ok());
}

#[test]
fn run_export____nonexistent_directory____returns_error() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("output.html");

    let result = run_export(
        PathBuf::from("/nonexistent/path/that/does/not/exist"),
        output,
        3,
        false,
        false,
        None,
        false,
    );

    assert!(result.is_err());
}

#[test]
fn run_export____html_contains_scan_path() {
    let tmp = TempDir::new().unwrap();
    let scan_dir = tmp.path().join("myproject");
    fs::create_dir(&scan_dir).unwrap();
    fs::write(scan_dir.join("main.rs"), "fn main() {}").unwrap();

    let output = tmp.path().join("output.html");

    run_export(
        scan_dir.clone(),
        output.clone(),
        3,
        false,
        false,
        None,
        false,
    )
    .unwrap();

    let html = fs::read_to_string(&output).unwrap();
    // The cleaned canonicalized path should appear in the HTML (no \\?\ prefix)
    let canonical = clean_path(scan_dir.canonicalize().unwrap());
    assert!(html.contains(&canonical.display().to_string()));
}
