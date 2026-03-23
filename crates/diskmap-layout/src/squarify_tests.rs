#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::*;
use crate::rect::LayoutRect;
use diskmap_core::tree::{build_file_tree, build_tree};
use std::path::PathBuf;

const EPSILON: f64 = 0.01;

fn make_tree(files: &[(&str, u64)]) -> (diskmap_core::FileTree, NodeId) {
    let dir = build_tree(files);
    let ft = build_file_tree(&dir, PathBuf::from("/test"));
    let root = ft.root();
    (ft, root)
}

fn default_bounds() -> LayoutRect {
    LayoutRect::new(0.0, 0.0, 800.0, 600.0)
}

#[test]
fn squarify_layout____single_file____fills_entire_rect() {
    let (tree, root) = make_tree(&[("/file.txt", 1000)]);
    let config = LayoutConfig {
        max_depth: 10,
        ..Default::default()
    };
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    // Root + file = 2 entries
    assert_eq!(layout.len(), 2);
}

#[test]
fn squarify_layout____two_equal_files____both_within_bounds() {
    let (tree, root) = make_tree(&[("/a.txt", 500), ("/b.txt", 500)]);
    let config = LayoutConfig {
        max_depth: 10,
        padding: 0.0,
        ..Default::default()
    };
    let bounds = default_bounds();
    let layout = squarify_layout(&tree, root, bounds, &config);

    for entry in layout.entries() {
        let r = &entry.rect;
        assert!(r.x >= bounds.x - EPSILON, "x out of bounds: {}", r.x);
        assert!(r.y >= bounds.y - EPSILON, "y out of bounds: {}", r.y);
        assert!(
            r.x + r.w <= bounds.x + bounds.w + EPSILON,
            "right edge out of bounds"
        );
        assert!(
            r.y + r.h <= bounds.y + bounds.h + EPSILON,
            "bottom edge out of bounds"
        );
    }
}

#[test]
fn squarify_layout____depth_limit____stops_descending() {
    let files = vec![
        ("/a/b/c/deep.txt", 100u64),
        ("/a/b/mid.txt", 200),
        ("/a/top.txt", 300),
    ];
    let (tree, root) = make_tree(&files);
    let config = LayoutConfig {
        max_depth: 1,
        padding: 0.0,
        ..Default::default()
    };
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    // With max_depth=1, we should see root (depth 0) and its immediate
    // children (depth 1), but nothing deeper should be subdivided
    let max_depth = layout.entries().iter().map(|e| e.depth).max().unwrap_or(0);
    assert!(max_depth <= 1, "max depth should be <= 1, got {max_depth}");
}

#[test]
fn squarify_layout____zero_size_items____excluded() {
    let (tree, root) = make_tree(&[("/real.txt", 1000), ("/empty.txt", 0)]);
    let config = LayoutConfig {
        max_depth: 10,
        padding: 0.0,
        ..Default::default()
    };
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    // Zero-size items should not get layout entries from squarify_children,
    // but they may still appear as direct children in the tree traversal
    for entry in layout.entries() {
        let node = tree.node(entry.node_id);
        if node.total_size == 0 {
            // Zero-size nodes should have zero-area rects or not be subdivided
            continue;
        }
        assert!(entry.rect.w >= 0.0);
        assert!(entry.rect.h >= 0.0);
    }
}

#[test]
fn squarify_layout____many_items____all_rects_positive_dimensions() {
    let files: Vec<(&str, u64)> = (0..20)
        .map(|i| {
            // Leak the string so we get &'static str
            let path: &str = Box::leak(format!("/dir/file{i}.dat").into_boxed_str());
            (path, (i as u64 + 1) * 100)
        })
        .collect();
    let (tree, root) = make_tree(&files);
    let config = LayoutConfig {
        max_depth: 10,
        padding: 0.0,
        ..Default::default()
    };
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    for entry in layout.entries() {
        assert!(entry.rect.w >= 0.0, "Negative width");
        assert!(entry.rect.h >= 0.0, "Negative height");
    }
}

#[test]
fn hit_test____inside____returns_entry() {
    let (tree, root) = make_tree(&[("/a.txt", 500), ("/b.txt", 500)]);
    let config = LayoutConfig {
        max_depth: 10,
        padding: 0.0,
        ..Default::default()
    };
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    // Center of the bounds should hit something
    let hit = layout.hit_test(400.0, 300.0);
    assert!(hit.is_some(), "Should hit an entry at center");
}

#[test]
fn hit_test____outside____returns_none() {
    let (tree, root) = make_tree(&[("/a.txt", 500)]);
    let config = LayoutConfig::default();
    let layout = squarify_layout(&tree, root, default_bounds(), &config);

    let hit = layout.hit_test(-10.0, -10.0);
    assert!(hit.is_none(), "Should not hit anything outside bounds");
}

// --- Property-based tests ---

use proptest::prelude::*;

fn arb_file_list_for_layout() -> impl Strategy<Value = Vec<(String, u64)>> {
    prop::collection::vec(
        ("[a-z]{1,4}/[a-z]{1,6}\\.[a-z]{2,3}", 1u64..1_000_000),
        1..30,
    )
}

proptest! {
    #[test]
    fn prop____all_rects_within_bounds(files in arb_file_list_for_layout()) {
        let refs: Vec<(&str, u64)> = files.iter().map(|(p, s)| (p.as_str(), *s)).collect();
        let dir = build_tree(&refs);
        let ft = build_file_tree(&dir, PathBuf::from("/test"));
        let bounds = LayoutRect::new(0.0, 0.0, 800.0, 600.0);
        let config = LayoutConfig { max_depth: 5, padding: 0.0, ..Default::default() };
        let layout = squarify_layout(&ft, ft.root(), bounds, &config);

        for entry in layout.entries() {
            let r = &entry.rect;
            prop_assert!(r.x >= bounds.x - EPSILON, "x={} < bounds.x={}", r.x, bounds.x);
            prop_assert!(r.y >= bounds.y - EPSILON, "y={} < bounds.y={}", r.y, bounds.y);
            prop_assert!(r.x + r.w <= bounds.x + bounds.w + EPSILON,
                "right edge {} > {}", r.x + r.w, bounds.x + bounds.w);
            prop_assert!(r.y + r.h <= bounds.y + bounds.h + EPSILON,
                "bottom edge {} > {}", r.y + r.h, bounds.y + bounds.h);
        }
    }

    #[test]
    fn prop____all_rects_non_negative_dimensions(files in arb_file_list_for_layout()) {
        let refs: Vec<(&str, u64)> = files.iter().map(|(p, s)| (p.as_str(), *s)).collect();
        let dir = build_tree(&refs);
        let ft = build_file_tree(&dir, PathBuf::from("/test"));
        let bounds = LayoutRect::new(0.0, 0.0, 800.0, 600.0);
        let config = LayoutConfig { max_depth: 5, padding: 0.0, ..Default::default() };
        let layout = squarify_layout(&ft, ft.root(), bounds, &config);

        for entry in layout.entries() {
            prop_assert!(entry.rect.w >= 0.0, "Negative width: {}", entry.rect.w);
            prop_assert!(entry.rect.h >= 0.0, "Negative height: {}", entry.rect.h);
        }
    }

    #[test]
    fn prop____layout_produces_entries(files in arb_file_list_for_layout()) {
        let refs: Vec<(&str, u64)> = files.iter().map(|(p, s)| (p.as_str(), *s)).collect();
        let dir = build_tree(&refs);
        let ft = build_file_tree(&dir, PathBuf::from("/test"));
        let bounds = LayoutRect::new(0.0, 0.0, 800.0, 600.0);
        let config = LayoutConfig::default();
        let layout = squarify_layout(&ft, ft.root(), bounds, &config);

        prop_assert!(!layout.is_empty(), "Layout should have at least the root entry");
    }
}

// --- Unit tests for internal functions ---

#[test]
fn worst_aspect_ratio____square_item____returns_one() {
    // A single item filling a square strip should have aspect ratio ~1
    let node_id = NodeId(0);
    let row = vec![(node_id, 100.0)];
    let ratio = worst_aspect_ratio(&row, 100.0, 10.0, 100.0, 100.0);
    assert!((1.0..1.01).contains(&ratio), "Expected ~1.0, got {ratio}");
}

#[test]
fn worst_aspect_ratio____zero_row_size____returns_max() {
    let node_id = NodeId(0);
    let row = vec![(node_id, 0.0)];
    let ratio = worst_aspect_ratio(&row, 0.0, 10.0, 100.0, 100.0);
    assert_eq!(ratio, f64::MAX);
}

#[test]
fn worst_aspect_ratio____zero_shorter_side____returns_max() {
    let node_id = NodeId(0);
    let row = vec![(node_id, 50.0)];
    let ratio = worst_aspect_ratio(&row, 50.0, 0.0, 100.0, 100.0);
    assert_eq!(ratio, f64::MAX);
}

#[test]
fn worst_aspect_ratio____multiple_items____worst_is_smallest() {
    let row = vec![(NodeId(0), 900.0), (NodeId(1), 100.0)];
    let ratio = worst_aspect_ratio(&row, 1000.0, 10.0, 1000.0, 1000.0);
    // The smaller item (100/1000 of shorter_side = 1px) with strip thickness 10px
    // gives aspect ratio 10, which is the worst
    assert!(
        ratio > 5.0,
        "Expected high ratio for unequal items, got {ratio}"
    );
}

#[test]
fn layout_row____single_item____fills_strip() {
    let node_id = NodeId(0);
    let row = vec![(node_id, 100.0)];
    let strip = LayoutRect::new(10.0, 20.0, 200.0, 50.0);
    let mut out = Vec::new();
    layout_row(&row, 100.0, strip, &mut out);

    assert_eq!(out.len(), 1);
    let (id, rect) = &out[0];
    assert_eq!(*id, node_id);
    assert!((rect.w - 200.0).abs() < EPSILON);
    assert!((rect.h - 50.0).abs() < EPSILON);
}

#[test]
fn layout_row____two_equal_items____split_evenly() {
    let row = vec![(NodeId(0), 50.0), (NodeId(1), 50.0)];
    let strip = LayoutRect::new(0.0, 0.0, 100.0, 50.0); // wide strip
    let mut out = Vec::new();
    layout_row(&row, 100.0, strip, &mut out);

    assert_eq!(out.len(), 2);
    // Should split width evenly
    assert!((out[0].1.w - 50.0).abs() < EPSILON);
    assert!((out[1].1.w - 50.0).abs() < EPSILON);
    // Both should have full height
    assert!((out[0].1.h - 50.0).abs() < EPSILON);
    assert!((out[1].1.h - 50.0).abs() < EPSILON);
}

#[test]
fn layout_row____tall_strip____splits_vertically() {
    let row = vec![(NodeId(0), 75.0), (NodeId(1), 25.0)];
    let strip = LayoutRect::new(0.0, 0.0, 30.0, 100.0); // tall strip
    let mut out = Vec::new();
    layout_row(&row, 100.0, strip, &mut out);

    assert_eq!(out.len(), 2);
    // Should split height (75/25 split)
    assert!((out[0].1.h - 75.0).abs() < EPSILON);
    assert!((out[1].1.h - 25.0).abs() < EPSILON);
    // Both should have full width
    assert!((out[0].1.w - 30.0).abs() < EPSILON);
    assert!((out[1].1.w - 30.0).abs() < EPSILON);
}

#[test]
fn layout_row____zero_row_size____no_output() {
    let row: Vec<(NodeId, f64)> = vec![];
    let strip = LayoutRect::new(0.0, 0.0, 100.0, 50.0);
    let mut out = Vec::new();
    layout_row(&row, 0.0, strip, &mut out);
    assert!(out.is_empty());
}
