#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::*;
use dumap_core::scan::ScanConfig;
use dumap_core::tree::{build_file_tree, build_tree};
use egui::{Pos2, Vec2};
use std::path::PathBuf;
use std::sync::Arc;

fn test_config() -> ScanConfig {
    ScanConfig {
        root: PathBuf::from("/tmp/test"),
        follow_links: false,
        include_hidden: false,
        max_depth: None,
        apparent_size: false,
    }
}

fn make_test_tree() -> FileTree {
    let files = vec![
        ("/a/b/deep.txt", 100u64),
        ("/a/mid.txt", 200),
        ("/c/other.txt", 300),
    ];
    let dir = build_tree(&files);
    build_file_tree(&dir, PathBuf::from("/test"))
}

// --- DumapApp::new ---

#[test]
fn new____default_state____tree_is_none() {
    let app = DumapApp::new(test_config());
    assert!(app.tree.is_none());
}

#[test]
fn new____default_state____layout_is_none() {
    let app = DumapApp::new(test_config());
    assert!(app.layout.is_none());
}

#[test]
fn new____default_state____navigation_is_none() {
    let app = DumapApp::new(test_config());
    assert!(app.navigation.is_none());
}

#[test]
fn new____default_state____no_scan_in_progress() {
    let app = DumapApp::new(test_config());
    assert!(app.scan_progress.is_none());
    assert!(app.scan_receiver.is_none());
}

#[test]
fn new____stores_config____root_path_preserved() {
    let config = ScanConfig {
        root: PathBuf::from("/custom/path"),
        follow_links: false,
        include_hidden: true,
        max_depth: Some(5),
        apparent_size: true,
    };
    let app = DumapApp::new(config);
    assert_eq!(app.scan_config.root, PathBuf::from("/custom/path"));
    assert!(app.scan_config.include_hidden);
    assert_eq!(app.scan_config.max_depth, Some(5));
    assert!(app.scan_config.apparent_size);
}

// --- invalidate_layout ---

#[test]
fn invalidate_layout____clears_cached_layout() {
    let mut app = DumapApp::new(test_config());
    app.last_panel_size = Some(Vec2::new(800.0, 600.0));

    app.invalidate_layout();

    assert!(app.layout.is_none());
    assert!(app.last_panel_size.is_none());
}

// --- poll_scan_result ---

#[test]
fn poll_scan_result____no_receiver____does_nothing() {
    let mut app = DumapApp::new(test_config());
    // Should not panic
    app.poll_scan_result();
    assert!(app.tree.is_none());
}

#[test]
fn poll_scan_result____successful_scan____sets_tree_and_navigation() {
    let mut app = DumapApp::new(test_config());
    app.scan_start = Some(Instant::now());

    let (tx, rx) = std::sync::mpsc::channel();
    app.scan_receiver = Some(rx);
    app.scan_progress = Some(Arc::new(ScanProgress::new()));

    let tree = make_test_tree();
    tx.send(Ok(tree)).unwrap();

    app.poll_scan_result();

    assert!(app.tree.is_some());
    assert!(app.navigation.is_some());
    assert!(app.scan_receiver.is_none());
    assert!(app.scan_progress.is_none());
    assert!(app.scan_duration.is_some());
}

#[test]
fn poll_scan_result____failed_scan____tree_stays_none() {
    let mut app = DumapApp::new(test_config());
    app.scan_start = Some(Instant::now());

    let (tx, rx) = std::sync::mpsc::channel();
    app.scan_receiver = Some(rx);
    app.scan_progress = Some(Arc::new(ScanProgress::new()));

    tx.send(Err("permission denied".to_string())).unwrap();

    app.poll_scan_result();

    assert!(app.tree.is_none());
    assert!(app.navigation.is_none());
    assert!(app.scan_receiver.is_none());
    assert!(app.scan_progress.is_none());
}

#[test]
fn poll_scan_result____nothing_ready_yet____state_unchanged() {
    let mut app = DumapApp::new(test_config());

    let (_tx, rx) = std::sync::mpsc::channel::<Result<FileTree, String>>();
    app.scan_receiver = Some(rx);
    app.scan_progress = Some(Arc::new(ScanProgress::new()));

    app.poll_scan_result();

    // Receiver still active, no result yet
    assert!(app.scan_receiver.is_some());
    assert!(app.scan_progress.is_some());
    assert!(app.tree.is_none());
}

// --- to_egui_rect ---

#[test]
fn to_egui_rect____zero_origin____maps_directly() {
    let layout_rect = dumap_layout::LayoutRect::new(10.0, 20.0, 100.0, 50.0);
    let origin = Pos2::new(0.0, 0.0);
    let rect = to_egui_rect(&layout_rect, origin);

    assert_eq!(rect.min.x, 10.0);
    assert_eq!(rect.min.y, 20.0);
    assert_eq!(rect.width(), 100.0);
    assert_eq!(rect.height(), 50.0);
}

#[test]
fn to_egui_rect____nonzero_origin____offsets_position() {
    let layout_rect = dumap_layout::LayoutRect::new(10.0, 20.0, 100.0, 50.0);
    let origin = Pos2::new(30.0, 40.0);
    let rect = to_egui_rect(&layout_rect, origin);

    assert_eq!(rect.min.x, 40.0);
    assert_eq!(rect.min.y, 60.0);
    assert_eq!(rect.width(), 100.0);
    assert_eq!(rect.height(), 50.0);
}

#[test]
fn to_egui_rect____zero_size____creates_zero_rect() {
    let layout_rect = dumap_layout::LayoutRect::new(5.0, 5.0, 0.0, 0.0);
    let origin = Pos2::new(0.0, 0.0);
    let rect = to_egui_rect(&layout_rect, origin);

    assert_eq!(rect.width(), 0.0);
    assert_eq!(rect.height(), 0.0);
}

// --- ensure_layout ---

#[test]
fn ensure_layout____no_tree____does_nothing() {
    let mut app = DumapApp::new(test_config());
    app.ensure_layout(Vec2::new(800.0, 600.0));
    assert!(app.layout.is_none());
}

#[test]
fn ensure_layout____with_tree____computes_layout() {
    let mut app = DumapApp::new(test_config());
    let tree = make_test_tree();
    let root = tree.root();
    app.tree = Some(Arc::new(tree));
    app.navigation = Some(NavigationModel::new(root));

    app.ensure_layout(Vec2::new(800.0, 600.0));

    assert!(app.layout.is_some());
    assert!(app.last_panel_size.is_some());
}

#[test]
fn ensure_layout____same_size____reuses_cached_layout() {
    let mut app = DumapApp::new(test_config());
    let tree = make_test_tree();
    let root = tree.root();
    app.tree = Some(Arc::new(tree));
    app.navigation = Some(NavigationModel::new(root));

    app.ensure_layout(Vec2::new(800.0, 600.0));
    let first_layout = app.layout.as_ref().unwrap() as *const TreemapLayout;

    app.ensure_layout(Vec2::new(800.0, 600.0));
    let second_layout = app.layout.as_ref().unwrap() as *const TreemapLayout;

    // Should be the same allocation (not recomputed)
    assert_eq!(first_layout, second_layout);
}

#[test]
fn ensure_layout____different_size____recomputes() {
    let mut app = DumapApp::new(test_config());
    let tree = make_test_tree();
    let root = tree.root();
    app.tree = Some(Arc::new(tree));
    app.navigation = Some(NavigationModel::new(root));

    app.ensure_layout(Vec2::new(800.0, 600.0));
    assert!(app.layout.is_some());

    // Significantly different size triggers recompute
    app.ensure_layout(Vec2::new(1200.0, 900.0));
    let size = app.last_panel_size.unwrap();
    assert_eq!(size.x, 1200.0);
    assert_eq!(size.y, 900.0);
}

// --- start_scan ---

#[test]
fn start_scan____sets_progress_and_receiver() {
    let mut app = DumapApp::new(ScanConfig {
        root: PathBuf::from("/tmp"),
        follow_links: false,
        include_hidden: false,
        max_depth: Some(1),
        apparent_size: false,
    });

    app.start_scan();

    assert!(app.scan_progress.is_some());
    assert!(app.scan_receiver.is_some());
    assert!(app.scan_start.is_some());
}
