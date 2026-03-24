use crate::category::FileCategory;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Index into the arena-allocated node vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// A node in the arena-allocated file tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// File or directory name (just the component, not full path)
    pub name: String,
    /// Kind-specific data
    pub kind: NodeKind,
    /// Total size of this node and all descendants (bytes)
    pub total_size: u64,
    /// Number of files in this subtree (1 for files, sum for dirs)
    pub file_count: u64,
    /// Parent node (None for root)
    pub parent: Option<NodeId>,
    /// Depth from root (root = 0)
    pub depth: u16,
}

/// Whether a node is a file or directory.
#[derive(Debug, Clone)]
pub enum NodeKind {
    File {
        /// Size in bytes
        size: u64,
        /// File category (derived from extension)
        category: FileCategory,
        /// Last modified time
        modified: Option<SystemTime>,
    },
    Directory {
        /// Children NodeIds, sorted by total_size descending
        children: Vec<NodeId>,
    },
}

/// Arena-based file tree. All nodes stored in a single Vec for cache locality.
#[derive(Debug)]
pub struct FileTree {
    /// All nodes, indexed by NodeId
    nodes: Vec<TreeNode>,
    /// Root node
    root: NodeId,
    /// The scanned path
    scan_root: PathBuf,
}

impl FileTree {
    /// Get a node by ID.
    pub fn node(&self, id: NodeId) -> &TreeNode {
        &self.nodes[id.0 as usize]
    }

    /// Get the root NodeId.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// The scanned root path.
    pub fn scan_root(&self) -> &Path {
        &self.scan_root
    }

    /// Total number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get direct children of a node (empty for files).
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        match &self.node(id).kind {
            NodeKind::Directory { children } => children,
            NodeKind::File { .. } => &[],
        }
    }

    /// Build the relative path for a node by walking up to (but excluding) the root.
    ///
    /// The root node represents `scan_root` itself, so it is excluded to avoid
    /// duplication when callers do `scan_root.join(path(...))`.
    pub fn path(&self, id: NodeId) -> PathBuf {
        let mut components = Vec::new();
        let mut current = id;
        loop {
            let node = self.node(current);
            match node.parent {
                Some(parent) => {
                    components.push(node.name.as_str());
                    current = parent;
                }
                None => break, // skip root node
            }
        }
        components.reverse();
        let mut path = PathBuf::new();
        for c in components {
            path.push(c);
        }
        path
    }
}

/// Build a FileTree from a DirNode tree.
///
/// This converts the HashMap-based DirNode tree into an arena-allocated
/// FileTree for efficient layout traversal. Children are sorted by
/// total_size descending.
pub fn build_file_tree(dir_node: &super::DirNode, scan_root: PathBuf) -> FileTree {
    let mut nodes = Vec::new();
    let root_name = scan_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| scan_root.to_string_lossy().into_owned());

    build_recursive(dir_node, &root_name, None, 0, &mut nodes);

    let root = NodeId(0);
    FileTree {
        nodes,
        root,
        scan_root,
    }
}

fn build_recursive(
    dir_node: &super::DirNode,
    name: &str,
    parent: Option<NodeId>,
    depth: u16,
    nodes: &mut Vec<TreeNode>,
) -> NodeId {
    let my_id = NodeId(nodes.len() as u32);

    if dir_node.children.is_empty() {
        // Leaf file
        let category = FileCategory::from_path(Path::new(name));
        nodes.push(TreeNode {
            name: name.to_string(),
            kind: NodeKind::File {
                size: dir_node.file_size,
                category,
                modified: None,
            },
            total_size: dir_node.file_size,
            file_count: 1,
            parent,
            depth,
        });
        return my_id;
    }

    // Directory — push a placeholder, then recurse into children
    nodes.push(TreeNode {
        name: name.to_string(),
        kind: NodeKind::Directory {
            children: Vec::new(),
        },
        total_size: 0,
        file_count: 0,
        parent,
        depth,
    });

    let mut child_ids: Vec<NodeId> = Vec::with_capacity(dir_node.children.len());
    let mut total_size = dir_node.file_size; // files directly in this dir
    let mut file_count: u64 = dir_node.file_count as u64;

    for (child_name, child_node) in &dir_node.children {
        let child_id = build_recursive(child_node, child_name, Some(my_id), depth + 1, nodes);
        let child = &nodes[child_id.0 as usize];
        total_size += child.total_size;
        file_count += child.file_count;
        child_ids.push(child_id);
    }

    // Sort children by total_size descending (for squarified layout)
    child_ids.sort_by(|a, b| {
        let sa = nodes[b.0 as usize].total_size;
        let sb = nodes[a.0 as usize].total_size;
        sa.cmp(&sb)
    });

    // Update the placeholder
    let node = &mut nodes[my_id.0 as usize];
    node.kind = NodeKind::Directory {
        children: child_ids,
    };
    node.total_size = total_size;
    node.file_count = file_count;

    my_id
}

#[cfg(test)]
#[path = "arena_tests.rs"]
mod arena_tests;
