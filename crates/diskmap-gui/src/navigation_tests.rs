#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::NavigationModel;
use diskmap_core::tree::{build_file_tree, build_tree};
use std::path::PathBuf;

/// Build a test tree: /root/{a/{b/{deep.txt}, mid.txt}, c/{other.txt}}
fn make_test_tree() -> diskmap_core::FileTree {
    let files = vec![
        ("/a/b/deep.txt", 100u64),
        ("/a/mid.txt", 200),
        ("/c/other.txt", 300),
    ];
    let dir = build_tree(&files);
    build_file_tree(&dir, PathBuf::from("/test"))
}

/// Find a node by name in the tree.
fn find_node(
    tree: &diskmap_core::FileTree,
    id: diskmap_core::NodeId,
    name: &str,
) -> Option<diskmap_core::NodeId> {
    if tree.node(id).name == name {
        return Some(id);
    }
    for &child_id in tree.children(id) {
        if let Some(found) = find_node(tree, child_id, name) {
            return Some(found);
        }
    }
    None
}

#[test]
fn new____initial_state____visible_root_is_scan_root() {
    let tree = make_test_tree();
    let nav = NavigationModel::new(tree.root());
    assert_eq!(nav.visible_root(), tree.root());
    assert!(!nav.can_zoom_out());
    assert_eq!(nav.breadcrumb().len(), 1);
}

#[test]
fn zoom_into____child_directory____updates_visible_root() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    nav.zoom_into(a_id, &tree);

    assert_eq!(nav.visible_root(), a_id);
    assert!(nav.can_zoom_out());
}

#[test]
fn zoom_into____builds_full_breadcrumb_path() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    // Zoom into deeply nested "b" (child of "a")
    let b_id = find_node(&tree, tree.root(), "b").unwrap();
    nav.zoom_into(b_id, &tree);

    // Breadcrumb should show: root > a > b
    let crumbs = nav.breadcrumb();
    assert_eq!(crumbs.len(), 3);
    assert_eq!(crumbs[0], tree.root());
    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    assert_eq!(crumbs[1], a_id);
    assert_eq!(crumbs[2], b_id);
}

#[test]
fn zoom_into____same_node____no_duplicate() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    // Zoom into root repeatedly — should not accumulate
    nav.zoom_into(tree.root(), &tree);
    nav.zoom_into(tree.root(), &tree);
    nav.zoom_into(tree.root(), &tree);

    assert_eq!(nav.breadcrumb().len(), 1);
    assert_eq!(nav.visible_root(), tree.root());
}

#[test]
fn zoom_out____from_nested____pops_one_level() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    let b_id = find_node(&tree, tree.root(), "b").unwrap();
    nav.zoom_into(b_id, &tree);
    assert_eq!(nav.visible_root(), b_id);

    nav.zoom_out();
    assert_eq!(nav.visible_root(), a_id);

    nav.zoom_out();
    assert_eq!(nav.visible_root(), tree.root());
}

#[test]
fn zoom_out____at_root____stays_at_root() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    nav.zoom_out();
    assert_eq!(nav.visible_root(), tree.root());
    assert_eq!(nav.breadcrumb().len(), 1);
}

#[test]
fn zoom_to_level____truncates_breadcrumb() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    let b_id = find_node(&tree, tree.root(), "b").unwrap();
    nav.zoom_into(b_id, &tree);
    assert_eq!(nav.breadcrumb().len(), 3); // root > a > b

    // Zoom to level 1 (should truncate to root > a)
    nav.zoom_to_level(1);
    assert_eq!(nav.breadcrumb().len(), 2);
    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    assert_eq!(nav.visible_root(), a_id);
}

#[test]
fn zoom_to_level____level_zero____goes_to_root() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    let b_id = find_node(&tree, tree.root(), "b").unwrap();
    nav.zoom_into(b_id, &tree);

    nav.zoom_to_level(0);
    assert_eq!(nav.visible_root(), tree.root());
    assert_eq!(nav.breadcrumb().len(), 1);
}

#[test]
fn can_zoom_out____at_root____false() {
    let tree = make_test_tree();
    let nav = NavigationModel::new(tree.root());
    assert!(!nav.can_zoom_out());
}

#[test]
fn can_zoom_out____after_zoom_in____true() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    nav.zoom_into(a_id, &tree);
    assert!(nav.can_zoom_out());
}

#[test]
fn zoom_into____sibling_directory____rebuilds_breadcrumb() {
    let tree = make_test_tree();
    let mut nav = NavigationModel::new(tree.root());

    // Zoom into "a" then directly into "c" (sibling, not child)
    let a_id = find_node(&tree, tree.root(), "a").unwrap();
    nav.zoom_into(a_id, &tree);
    assert_eq!(nav.visible_root(), a_id);

    let c_id = find_node(&tree, tree.root(), "c").unwrap();
    nav.zoom_into(c_id, &tree);

    // Breadcrumb should be rebuilt as root > c (not root > a > c)
    assert_eq!(nav.visible_root(), c_id);
    let crumbs = nav.breadcrumb();
    assert_eq!(crumbs.len(), 2);
    assert_eq!(crumbs[0], tree.root());
    assert_eq!(crumbs[1], c_id);
}

#[test]
fn max_display_depth____default____is_four() {
    let tree = make_test_tree();
    let nav = NavigationModel::new(tree.root());
    assert_eq!(nav.max_display_depth, 4);
}
