use serde::Serialize;
use std::collections::HashMap;

/// Intermediate tree node for building the hierarchy from filesystem paths.
///
/// Each node represents either a file (leaf with `file_size > 0` and no children)
/// or a directory (with children). Sizes aggregate upward via `total_size()`.
#[derive(Debug)]
pub struct DirNode {
    pub children: HashMap<String, DirNode>,
    /// Total size of files directly in this node (leaf files)
    pub file_size: u64,
    /// Number of files directly in this node
    pub file_count: usize,
}

/// JSON-serializable tree node for ECharts treemap visualization.
#[derive(Debug, Serialize)]
pub struct EChartsNode {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<EChartsNode>,
}

impl DirNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            file_size: 0,
            file_count: 0,
        }
    }

    /// Insert a file path into the tree, splitting on path separators.
    pub fn insert(&mut self, path_components: &[&str], size: u64) {
        if path_components.is_empty() {
            return;
        }

        if path_components.len() == 1 {
            // Leaf file — store as a child with no further children
            let leaf = self
                .children
                .entry(path_components[0].to_string())
                .or_default();
            leaf.file_size += size;
            leaf.file_count += 1;
        } else {
            // Intermediate directory
            let child = self
                .children
                .entry(path_components[0].to_string())
                .or_default();
            child.insert(&path_components[1..], size);
        }
    }

    /// Compute total size of this subtree.
    pub fn total_size(&self) -> u64 {
        let children_size: u64 = self.children.values().map(|c| c.total_size()).sum();
        self.file_size + children_size
    }

    /// Total number of files in this subtree.
    pub fn total_file_count(&self) -> usize {
        let children_count: usize = self.children.values().map(|c| c.total_file_count()).sum();
        self.file_count + children_count
    }

    /// Convert to ECharts JSON tree, collapsing single-child directories.
    ///
    /// Single-child directory chains are collapsed (e.g., `a/b/c` becomes
    /// a single node named `"a/b/c"`) to reduce visual clutter.
    pub fn to_echarts(&self, name: &str) -> EChartsNode {
        if self.children.is_empty() {
            // Leaf
            return EChartsNode {
                name: name.to_string(),
                value: Some(self.file_size),
                children: Vec::new(),
            };
        }

        let mut children: Vec<EChartsNode> = self
            .children
            .iter()
            .map(|(child_name, child_node)| child_node.to_echarts(child_name))
            .collect();

        // Sort children by value descending (largest first) for better treemap layout
        children.sort_by(|a, b| {
            let size_a = echarts_subtree_size(a);
            let size_b = echarts_subtree_size(b);
            size_b.cmp(&size_a)
        });

        // Collapse single-child directory chains: /a/b/c -> "a/b/c"
        if children.len() == 1 && !children[0].children.is_empty() {
            let child = children.remove(0);
            let collapsed_name = format!("{}/{}", name, child.name);
            return EChartsNode {
                name: collapsed_name,
                value: child.value,
                children: child.children,
            };
        }

        EChartsNode {
            name: name.to_string(),
            value: None,
            children,
        }
    }
}

impl Default for DirNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the total size of an EChartsNode subtree for sorting.
fn echarts_subtree_size(node: &EChartsNode) -> u64 {
    if let Some(val) = node.value {
        val
    } else {
        node.children.iter().map(echarts_subtree_size).sum()
    }
}

/// Split a file path into components, handling both Unix and Windows separators.
pub fn split_path(path: &str) -> Vec<&str> {
    path.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
}

/// Build a tree from a list of (path, size) pairs.
pub fn build_tree(files: &[(&str, u64)]) -> DirNode {
    let mut root = DirNode::new();
    for (path, size) in files {
        let components = split_path(path);
        if !components.is_empty() {
            root.insert(&components, *size);
        }
    }
    root
}
