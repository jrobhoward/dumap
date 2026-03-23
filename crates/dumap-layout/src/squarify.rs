use crate::rect::LayoutRect;
use dumap_core::tree::{FileTree, NodeId, NodeKind};

/// Configuration for the layout algorithm.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Maximum depth to lay out (relative to visible root).
    pub max_depth: u16,
    /// Minimum rectangle dimension in pixels. Rectangles smaller than this
    /// are not subdivided further.
    pub min_rect_size: f64,
    /// Padding between directory boundaries (pixels per side).
    pub padding: f64,
    /// Height reserved at the top of each directory for the name header (pixels).
    /// Set to 0.0 to disable directory headers.
    pub header_height: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            min_rect_size: 2.0,
            padding: 3.0,
            header_height: 20.0,
        }
    }
}

/// A laid-out treemap entry: pairs a file tree NodeId with its screen rectangle.
#[derive(Debug, Clone)]
pub struct LayoutEntry {
    pub node_id: NodeId,
    pub rect: LayoutRect,
    /// Depth relative to the visible root (0 = visible root).
    pub depth: u16,
}

/// The complete layout result.
pub struct TreemapLayout {
    /// Laid-out entries in parent-before-child order.
    entries: Vec<LayoutEntry>,
}

impl TreemapLayout {
    /// Get all entries.
    pub fn entries(&self) -> &[LayoutEntry] {
        &self.entries
    }

    /// Number of laid-out entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Find the deepest entry whose rectangle contains the given point.
    /// Returns None if no entry contains the point.
    pub fn hit_test(&self, x: f64, y: f64) -> Option<&LayoutEntry> {
        // Iterate in reverse (children before parents) to find the deepest match
        self.entries.iter().rev().find(|entry| {
            let r = &entry.rect;
            x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
        })
    }
}

/// Lay out a subtree of the FileTree starting at `root_id` into `bounds`.
pub fn squarify_layout(
    tree: &FileTree,
    root_id: NodeId,
    bounds: LayoutRect,
    config: &LayoutConfig,
) -> TreemapLayout {
    let estimated_capacity = tree.len().min(10_000);
    let mut entries = Vec::with_capacity(estimated_capacity);
    layout_node(tree, root_id, bounds, 0, config, &mut entries);
    TreemapLayout { entries }
}

fn layout_node(
    tree: &FileTree,
    node_id: NodeId,
    rect: LayoutRect,
    depth: u16,
    config: &LayoutConfig,
    out: &mut Vec<LayoutEntry>,
) {
    let node = tree.node(node_id);

    // Record this node's layout
    out.push(LayoutEntry {
        node_id,
        rect,
        depth,
    });

    // Stop conditions
    let children = match &node.kind {
        NodeKind::File { .. } => return,
        NodeKind::Directory { children } => children,
    };
    if depth >= config.max_depth || !rect.is_visible(config.min_rect_size) || children.is_empty() {
        return;
    }

    // Apply padding to create visual border for directories
    let padded = rect.inset(config.padding);
    if padded.w <= 0.0 || padded.h <= 0.0 {
        return;
    }

    // Reserve space at the top for the directory header label
    let inner =
        if config.header_height > 0.0 && padded.h > config.header_height + config.min_rect_size {
            LayoutRect::new(
                padded.x,
                padded.y + config.header_height,
                padded.w,
                padded.h - config.header_height,
            )
        } else {
            padded
        };
    if inner.w <= 0.0 || inner.h <= 0.0 {
        return;
    }

    // Children are pre-sorted by total_size descending in FileTree.
    // Filter out zero-size children.
    let sized_children: Vec<(NodeId, f64)> = children
        .iter()
        .map(|&cid| (cid, tree.node(cid).total_size as f64))
        .filter(|(_, size)| *size > 0.0)
        .collect();

    if sized_children.is_empty() {
        return;
    }

    // Run squarified algorithm
    let child_rects = squarify_children(&sized_children, inner);

    // Recurse into each child
    for (child_id, child_rect) in child_rects {
        layout_node(tree, child_id, child_rect, depth + 1, config, out);
    }
}

/// Core squarified algorithm: given items sorted by size descending and a
/// bounding rectangle, compute the rectangle for each item.
///
/// Algorithm (Bruls, Huizing, van Wijk 2000):
/// 1. Start with the full rectangle as "remaining" area.
/// 2. Begin a new row (strip along the shorter side).
/// 3. Add items one at a time.
/// 4. After each addition, compute the worst aspect ratio in the row.
/// 5. If adding the next item would make the worst aspect ratio worse,
///    finalize the current row and start a new one.
/// 6. Repeat until all items are placed.
fn squarify_children(children: &[(NodeId, f64)], bounds: LayoutRect) -> Vec<(NodeId, LayoutRect)> {
    let total_size: f64 = children.iter().map(|(_, s)| *s).sum();
    if total_size <= 0.0 {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(children.len());
    let mut remaining = bounds;
    let mut i = 0;

    while i < children.len() {
        if remaining.w <= 0.0 || remaining.h <= 0.0 {
            break;
        }

        let remaining_size: f64 = children[i..].iter().map(|(_, s)| *s).sum();
        let shorter = remaining.shorter_side();

        // Build current row
        let mut row: Vec<(NodeId, f64)> = Vec::new();
        let mut row_size = 0.0;

        // Add first item (always)
        row.push(children[i]);
        row_size += children[i].1;
        i += 1;

        // Try adding more items
        while i < children.len() {
            let worst_before =
                worst_aspect_ratio(&row, row_size, shorter, remaining_size, remaining.area());

            row.push(children[i]);
            let candidate_size = row_size + children[i].1;
            let worst_after = worst_aspect_ratio(
                &row,
                candidate_size,
                shorter,
                remaining_size,
                remaining.area(),
            );

            if worst_after > worst_before {
                // Adding this item made things worse; remove and finalize row
                row.pop();
                break;
            }
            row_size = candidate_size;
            i += 1;
        }

        // Lay out the finalized row as a strip
        let row_fraction = row_size / remaining_size;
        let (strip, rest) = remaining.split_strip(row_fraction);
        layout_row(&row, row_size, strip, &mut result);
        remaining = rest;
    }

    result
}

/// Compute worst (maximum) aspect ratio among items in a row.
fn worst_aspect_ratio(
    row: &[(NodeId, f64)],
    row_size: f64,
    shorter_side: f64,
    total_remaining: f64,
    remaining_area: f64,
) -> f64 {
    if row_size <= 0.0 || shorter_side <= 0.0 || total_remaining <= 0.0 {
        return f64::MAX;
    }

    let strip_area = (row_size / total_remaining) * remaining_area;
    let strip_thickness = strip_area / shorter_side;

    if strip_thickness <= 0.0 {
        return f64::MAX;
    }

    let mut worst = 1.0_f64;
    for &(_, item_size) in row {
        let item_length = (item_size / row_size) * shorter_side;
        if item_length <= 0.0 {
            continue;
        }
        let ratio = if item_length > strip_thickness {
            item_length / strip_thickness
        } else {
            strip_thickness / item_length
        };
        worst = worst.max(ratio);
    }
    worst
}

/// Lay out items within a finalized row strip.
fn layout_row(
    row: &[(NodeId, f64)],
    row_size: f64,
    strip: LayoutRect,
    out: &mut Vec<(NodeId, LayoutRect)>,
) {
    if row_size <= 0.0 {
        return;
    }

    let mut offset = 0.0;
    for &(node_id, item_size) in row {
        let fraction = item_size / row_size;
        let item_rect = if strip.is_wide() {
            LayoutRect {
                x: strip.x + offset * strip.w,
                y: strip.y,
                w: fraction * strip.w,
                h: strip.h,
            }
        } else {
            LayoutRect {
                x: strip.x,
                y: strip.y + offset * strip.h,
                w: strip.w,
                h: fraction * strip.h,
            }
        };
        out.push((node_id, item_rect));
        offset += fraction;
    }
}

#[cfg(test)]
#[path = "squarify_tests.rs"]
mod squarify_tests;
