pub mod category;
pub mod error;
pub mod html;
pub mod scan;
pub mod tree;

pub use category::FileCategory;
pub use error::ScanError;
pub use html::generate_html;
pub use scan::{ScanConfig, ScanProgress, scan_directory};
pub use tree::{DirNode, EChartsNode, FileTree, NodeId, NodeKind, TreeNode, build_file_tree};
