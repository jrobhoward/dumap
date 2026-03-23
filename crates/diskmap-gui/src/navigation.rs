use diskmap_core::tree::{FileTree, NodeId};

/// Tracks navigation state: which subtree is visible and the breadcrumb path.
pub struct NavigationModel {
    /// The scan root (never changes after scan).
    scan_root: NodeId,
    /// Path from scan_root to current visible root.
    /// First element is scan_root, last is visible_root.
    breadcrumb: Vec<NodeId>,
    /// Maximum display depth (relative to visible root).
    pub max_display_depth: u16,
}

impl NavigationModel {
    pub fn new(scan_root: NodeId) -> Self {
        Self {
            scan_root,
            breadcrumb: vec![scan_root],
            max_display_depth: 4,
        }
    }

    /// The currently visible root.
    pub fn visible_root(&self) -> NodeId {
        *self.breadcrumb.last().unwrap_or(&self.scan_root)
    }

    /// Zoom into a directory (make it the new visible root).
    ///
    /// Builds the full path from scan_root to node_id using parent links,
    /// so the breadcrumb always shows the complete hierarchy.
    pub fn zoom_into(&mut self, node_id: NodeId, tree: &FileTree) {
        // Don't zoom if already at this node
        if self.visible_root() == node_id {
            return;
        }

        // Build path from scan_root to node_id by walking parent links
        let mut path = Vec::new();
        let mut current = node_id;
        loop {
            path.push(current);
            if current == self.scan_root {
                break;
            }
            match tree.node(current).parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
        path.reverse();
        self.breadcrumb = path;
    }

    /// Zoom out one level.
    pub fn zoom_out(&mut self) {
        if self.breadcrumb.len() > 1 {
            self.breadcrumb.pop();
        }
    }

    /// Zoom to a specific breadcrumb level.
    pub fn zoom_to_level(&mut self, level: usize) {
        if level < self.breadcrumb.len() {
            self.breadcrumb.truncate(level + 1);
        }
    }

    /// Get the breadcrumb path.
    pub fn breadcrumb(&self) -> &[NodeId] {
        &self.breadcrumb
    }

    /// Whether we can zoom out further.
    pub fn can_zoom_out(&self) -> bool {
        self.breadcrumb.len() > 1
    }
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod navigation_tests;
