//! Hard errors from `gyeoldaero-core`. Content problems are diagnostics, not errors.

use std::path::PathBuf;

/// A failure that stops loading a notebook or its configuration.
///
/// Stub (PRD-001 tests): variants follow the plan (unreadable root, I/O, config parse)
/// and may be refined during implementation.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The notebook root doesn't exist, isn't a directory, or can't be read.
    #[error("cannot read notebook root {path}: {source}")]
    UnreadableRoot {
        /// The root that was given.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// An I/O error while scanning.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The file or directory being read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// `gyeoldaero.toml` exists but can't be parsed.
    #[error("cannot parse {path}: {message}")]
    Config {
        /// Path of the config file.
        path: PathBuf,
        /// What was wrong with it.
        message: String,
    },
}
