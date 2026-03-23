use diskmap_core::category::FileCategory;
use diskmap_core::tree::{NodeKind, TreeNode};
use egui::Color32;

/// Convert a FileCategory to an egui Color32, using the shared RGB values.
pub fn category_color(category: FileCategory) -> Color32 {
    let (r, g, b) = category.rgb();
    Color32::from_rgb(r, g, b)
}

/// Get the fill color for a tree node.
pub fn node_color(node: &TreeNode) -> Color32 {
    match &node.kind {
        NodeKind::File { category, .. } => category_color(*category),
        NodeKind::Directory { .. } => Color32::from_rgb(50, 50, 60),
    }
}

/// Lighten a color by a factor (0.0 = unchanged, 1.0 = white).
pub fn lighten(color: Color32, factor: f32) -> Color32 {
    let f = factor.clamp(0.0, 1.0);
    Color32::from_rgb(
        (color.r() as f32 + (255.0 - color.r() as f32) * f) as u8,
        (color.g() as f32 + (255.0 - color.g() as f32) * f) as u8,
        (color.b() as f32 + (255.0 - color.b() as f32) * f) as u8,
    )
}
