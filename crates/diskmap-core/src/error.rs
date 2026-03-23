use std::path::PathBuf;
use thiserror::Error;

/// Errors that occur during filesystem scanning
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Scan cancelled by user")]
    Cancelled,

    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
