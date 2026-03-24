#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::*;
use dumap_core::category::FileCategory;
use dumap_core::tree::{NodeKind, TreeNode};
use egui::Color32;

// --- category_color ---

#[test]
fn category_color____code_category____matches_rgb() {
    let color = category_color(FileCategory::Code);
    let (r, g, b) = FileCategory::Code.rgb();
    assert_eq!(color, Color32::from_rgb(r, g, b));
}

#[test]
fn category_color____all_categories____each_returns_valid_color() {
    for &cat in FileCategory::ALL {
        let color = category_color(cat);
        let (r, g, b) = cat.rgb();
        assert_eq!(color, Color32::from_rgb(r, g, b));
    }
}

// --- node_color ---

#[test]
fn node_color____file_node____returns_category_color() {
    let node = TreeNode {
        name: "test.rs".to_string(),
        kind: NodeKind::File {
            size: 100,
            category: FileCategory::Code,
            modified: None,
        },
        total_size: 100,
        file_count: 1,
        parent: None,
        depth: 0,
    };
    let color = node_color(&node);
    assert_eq!(color, category_color(FileCategory::Code));
}

#[test]
fn node_color____directory_node____returns_dark_gray() {
    let node = TreeNode {
        name: "src".to_string(),
        kind: NodeKind::Directory { children: vec![] },
        total_size: 1000,
        file_count: 5,
        parent: None,
        depth: 0,
    };
    let color = node_color(&node);
    assert_eq!(color, Color32::from_rgb(50, 50, 60));
}

#[test]
fn node_color____different_file_categories____different_colors() {
    let make_file_node = |cat: FileCategory| TreeNode {
        name: "file".to_string(),
        kind: NodeKind::File {
            size: 100,
            category: cat,
            modified: None,
        },
        total_size: 100,
        file_count: 1,
        parent: None,
        depth: 0,
    };

    let code_color = node_color(&make_file_node(FileCategory::Code));
    let image_color = node_color(&make_file_node(FileCategory::Image));
    assert_ne!(code_color, image_color);
}

// --- lighten ---

#[test]
fn lighten____factor_zero____unchanged() {
    let color = Color32::from_rgb(100, 150, 200);
    let result = lighten(color, 0.0);
    assert_eq!(result, color);
}

#[test]
fn lighten____factor_one____white() {
    let color = Color32::from_rgb(100, 150, 200);
    let result = lighten(color, 1.0);
    assert_eq!(result, Color32::from_rgb(255, 255, 255));
}

#[test]
fn lighten____factor_half____midpoint_toward_white() {
    let color = Color32::from_rgb(100, 0, 200);
    let result = lighten(color, 0.5);
    // 100 + (255 - 100) * 0.5 = 177.5 → 177
    // 0 + (255 - 0) * 0.5 = 127.5 → 127
    // 200 + (255 - 200) * 0.5 = 227.5 → 227
    assert_eq!(result.r(), 177);
    assert_eq!(result.g(), 127);
    assert_eq!(result.b(), 227);
}

#[test]
fn lighten____factor_negative____clamped_to_zero() {
    let color = Color32::from_rgb(100, 150, 200);
    let result = lighten(color, -0.5);
    assert_eq!(result, color);
}

#[test]
fn lighten____factor_above_one____clamped_to_white() {
    let color = Color32::from_rgb(100, 150, 200);
    let result = lighten(color, 2.0);
    assert_eq!(result, Color32::from_rgb(255, 255, 255));
}

#[test]
fn lighten____black____scales_correctly() {
    let black = Color32::from_rgb(0, 0, 0);
    let result = lighten(black, 0.5);
    assert_eq!(result.r(), 127);
    assert_eq!(result.g(), 127);
    assert_eq!(result.b(), 127);
}

#[test]
fn lighten____white____stays_white() {
    let white = Color32::from_rgb(255, 255, 255);
    let result = lighten(white, 0.5);
    assert_eq!(result, white);
}
