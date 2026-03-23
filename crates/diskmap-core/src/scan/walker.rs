use crate::error::ScanError;
use crate::tree::{DirNode, split_path};
use ignore::WalkBuilder;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::debug;

/// Configuration for a filesystem scan.
pub struct ScanConfig {
    /// Root path to scan
    pub root: PathBuf,
    /// Whether to follow symbolic links (default: false)
    pub follow_links: bool,
    /// Whether to include hidden files (default: true)
    pub include_hidden: bool,
    /// Maximum directory depth (None = unlimited)
    pub max_depth: Option<usize>,
    /// Whether to use apparent size instead of disk usage (default: false)
    pub apparent_size: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            follow_links: false,
            include_hidden: true,
            max_depth: None,
            apparent_size: false,
        }
    }
}

/// Progress counters shared between the scan thread and the UI.
pub struct ScanProgress {
    pub files_found: Arc<AtomicU64>,
    pub dirs_found: Arc<AtomicU64>,
    pub bytes_found: Arc<AtomicU64>,
    pub current_path: Arc<RwLock<String>>,
    pub cancelled: Arc<AtomicBool>,
}

impl ScanProgress {
    pub fn new() -> Self {
        Self {
            files_found: Arc::new(AtomicU64::new(0)),
            dirs_found: Arc::new(AtomicU64::new(0)),
            bytes_found: Arc::new(AtomicU64::new(0)),
            current_path: Arc::new(RwLock::new(String::new())),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the size of a file from metadata.
///
/// When `apparent` is true, returns the logical file size.
/// When false, returns the actual disk usage (block-aligned on Unix).
fn file_size(metadata: &std::fs::Metadata, apparent: bool) -> u64 {
    if apparent {
        metadata.len()
    } else {
        disk_usage(metadata)
    }
}

#[cfg(unix)]
fn disk_usage(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    // blocks are always 512-byte units on Unix
    metadata.blocks() * 512
}

#[cfg(not(unix))]
fn disk_usage(metadata: &std::fs::Metadata) -> u64 {
    // On Windows, apparent size is the best we can do without Win32 API calls
    metadata.len()
}

/// Scan a directory tree and produce a DirNode tree.
///
/// Uses `ignore::WalkBuilder` for efficient traversal with symlink safety.
/// Progress is reported via the `ScanProgress` atomic counters.
pub fn scan_directory(config: &ScanConfig, progress: &ScanProgress) -> Result<DirNode, ScanError> {
    let root = &config.root;

    if !root.exists() {
        return Err(ScanError::PathNotFound(root.clone()));
    }
    if !root.is_dir() {
        return Err(ScanError::NotADirectory(root.clone()));
    }

    let root_canonical = root.canonicalize().map_err(|e| ScanError::Io {
        path: root.clone(),
        source: e,
    })?;

    let mut builder = WalkBuilder::new(&root_canonical);
    builder
        .hidden(!config.include_hidden)
        .follow_links(config.follow_links)
        .parents(false);

    if let Some(depth) = config.max_depth {
        builder.max_depth(Some(depth));
    }

    let mut tree = DirNode::new();
    let root_prefix = root_canonical.as_path();

    for entry_result in builder.build() {
        if progress.cancelled.load(Ordering::Relaxed) {
            return Err(ScanError::Cancelled);
        }

        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                debug!("Walk error: {err:?}");
                continue;
            }
        };

        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };

        if file_type.is_dir() {
            progress.dirs_found.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();

        // Get metadata (don't follow symlinks)
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(err) => {
                debug!("Unable to read metadata for {path:?}: {err:?}");
                continue;
            }
        };

        let size = file_size(&metadata, config.apparent_size);

        // Get relative path from scan root
        let rel_path = match path.strip_prefix(root_prefix) {
            Ok(rel) => rel,
            Err(_) => path,
        };

        let path_str = rel_path.to_string_lossy();
        let components = split_path(&path_str);
        if !components.is_empty() {
            tree.insert(&components, size);
        }

        progress.files_found.fetch_add(1, Ordering::Relaxed);
        progress.bytes_found.fetch_add(size, Ordering::Relaxed);

        // Update current path periodically (every file for now; could throttle)
        *progress.current_path.write() = path.to_string_lossy().into_owned();
    }

    Ok(tree)
}

/// Format a byte count as a human-readable string.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Find the N largest entries (files or directories) in a tree.
///
/// Returns a list of (path, size) pairs sorted by size descending.
pub fn find_largest(
    root: &DirNode,
    root_path: &Path,
    count: usize,
    files_only: bool,
) -> Vec<(PathBuf, u64)> {
    let mut results: Vec<(PathBuf, u64)> = Vec::new();
    collect_entries(root, root_path, files_only, &mut results);
    results.sort_by(|a, b| b.1.cmp(&a.1));
    results.truncate(count);
    results
}

fn collect_entries(
    node: &DirNode,
    current_path: &Path,
    files_only: bool,
    results: &mut Vec<(PathBuf, u64)>,
) {
    for (name, child) in &node.children {
        let child_path = current_path.join(name);
        if child.children.is_empty() {
            // Leaf (file)
            results.push((child_path, child.file_size));
        } else {
            // Directory
            if !files_only {
                results.push((child_path.clone(), child.total_size()));
            }
            collect_entries(child, &child_path, files_only, results);
        }
    }
}
