#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::*;
use crate::category::FileCategory;
use crate::tree::{build_file_tree, build_tree};
use std::path::PathBuf;

fn make_test_tree() -> FileTree {
    let files = vec![
        ("/a/b/file1.rs", 1000u64),
        ("/a/b/file2.txt", 500),
        ("/a/file3.png", 2000),
        ("/c/file4.zip", 3000),
    ];
    let dir = build_tree(&files);
    build_file_tree(&dir, PathBuf::from("/test"))
}

#[test]
fn build_file_tree____basic____correct_node_count() {
    let tree = make_test_tree();
    // root + a + b + file1 + file2 + file3 + c + file4 = 8
    assert_eq!(tree.len(), 8);
}

#[test]
fn build_file_tree____basic____correct_total_size() {
    let tree = make_test_tree();
    let root = tree.node(tree.root());
    assert_eq!(root.total_size, 1000 + 500 + 2000 + 3000);
}

#[test]
fn build_file_tree____basic____correct_file_count() {
    let tree = make_test_tree();
    let root = tree.node(tree.root());
    assert_eq!(root.file_count, 4);
}

#[test]
fn build_file_tree____children_sorted_by_size_descending() {
    let tree = make_test_tree();
    let root_children = tree.children(tree.root());

    let sizes: Vec<u64> = root_children
        .iter()
        .map(|id| tree.node(*id).total_size)
        .collect();

    // Should be descending
    for i in 0..sizes.len().saturating_sub(1) {
        assert!(
            sizes[i] >= sizes[i + 1],
            "Children not sorted descending: {:?}",
            sizes
        );
    }
}

#[test]
fn build_file_tree____leaf_has_correct_category() {
    let files = vec![("/src/main.rs", 100u64)];
    let dir = build_tree(&files);
    let tree = build_file_tree(&dir, PathBuf::from("/proj"));

    // Find the .rs leaf
    fn find_leaf<'a>(tree: &'a FileTree, id: NodeId, name: &str) -> Option<&'a TreeNode> {
        let node = tree.node(id);
        if node.name == name {
            return Some(node);
        }
        for child_id in tree.children(id) {
            if let Some(found) = find_leaf(tree, *child_id, name) {
                return Some(found);
            }
        }
        None
    }

    let rs_node = find_leaf(&tree, tree.root(), "main.rs").unwrap();
    match &rs_node.kind {
        NodeKind::File { category, .. } => assert_eq!(*category, FileCategory::Code),
        _ => panic!("Expected file node"),
    }
}

#[test]
fn build_file_tree____path____reconstructs_from_root() {
    let files = vec![("/a/b/deep.txt", 100u64)];
    let dir = build_tree(&files);
    let tree = build_file_tree(&dir, PathBuf::from("/scan"));

    // Find the leaf
    fn find_by_name(tree: &FileTree, id: NodeId, name: &str) -> Option<NodeId> {
        if tree.node(id).name == name {
            return Some(id);
        }
        for child_id in tree.children(id) {
            if let Some(found) = find_by_name(tree, *child_id, name) {
                return Some(found);
            }
        }
        None
    }

    let leaf_id = find_by_name(&tree, tree.root(), "deep.txt").unwrap();
    let path = tree.path(leaf_id);
    assert!(
        path.to_string_lossy().contains("deep.txt"),
        "Path should contain filename: {:?}",
        path
    );
}

#[test]
fn path____root_node____returns_empty_path() {
    let files = vec![("/a/file.txt", 100u64)];
    let dir = build_tree(&files);
    let tree = build_file_tree(&dir, PathBuf::from("/scan"));

    let root_path = tree.path(tree.root());
    assert_eq!(
        root_path,
        PathBuf::new(),
        "Root path should be empty since it represents scan_root itself"
    );
}

#[test]
fn path____joined_with_scan_root____produces_correct_absolute_path() {
    let files = vec![("/a/b/deep.txt", 100u64)];
    let dir = build_tree(&files);
    let tree = build_file_tree(&dir, PathBuf::from("/scan"));

    fn find_by_name(tree: &FileTree, id: NodeId, name: &str) -> Option<NodeId> {
        if tree.node(id).name == name {
            return Some(id);
        }
        for child_id in tree.children(id) {
            if let Some(found) = find_by_name(tree, *child_id, name) {
                return Some(found);
            }
        }
        None
    }

    let leaf_id = find_by_name(&tree, tree.root(), "deep.txt").unwrap();
    let rel_path = tree.path(leaf_id);
    let abs_path = tree.scan_root().join(&rel_path);

    // Must not duplicate the root component
    let abs_str = abs_path.to_string_lossy();
    assert!(
        abs_str.ends_with("deep.txt"),
        "Absolute path should end with filename: {abs_str}"
    );
    // Count occurrences of "scan" — should appear exactly once (from scan_root)
    assert_eq!(
        abs_str.matches("scan").count(),
        1,
        "Root component should not be duplicated: {abs_str}"
    );
}

#[test]
fn path____intermediate_node____joined_correctly() {
    let files = vec![("/subdir/file.txt", 100u64)];
    let dir = build_tree(&files);
    let scan_root = PathBuf::from("/myroot");
    let tree = build_file_tree(&dir, scan_root.clone());

    fn find_by_name(tree: &FileTree, id: NodeId, name: &str) -> Option<NodeId> {
        if tree.node(id).name == name {
            return Some(id);
        }
        for child_id in tree.children(id) {
            if let Some(found) = find_by_name(tree, *child_id, name) {
                return Some(found);
            }
        }
        None
    }

    let dir_id = find_by_name(&tree, tree.root(), "subdir").unwrap();
    let abs_path = tree.scan_root().join(tree.path(dir_id));
    assert_eq!(abs_path, PathBuf::from("/myroot/subdir"));
}

#[test]
fn build_file_tree____parent_links____correct() {
    let tree = make_test_tree();
    for child_id in tree.children(tree.root()) {
        let child = tree.node(*child_id);
        assert_eq!(child.parent, Some(tree.root()));
    }
}

#[test]
fn build_file_tree____depth____increments_correctly() {
    let tree = make_test_tree();
    let root = tree.node(tree.root());
    assert_eq!(root.depth, 0);

    for child_id in tree.children(tree.root()) {
        let child = tree.node(*child_id);
        assert_eq!(child.depth, 1);
    }
}

#[test]
fn build_file_tree____empty_dir____single_node() {
    let dir = crate::tree::DirNode::new();
    let tree = build_file_tree(&dir, PathBuf::from("/empty"));
    // Empty dir has no children, treated as a leaf
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.node(tree.root()).total_size, 0);
}
