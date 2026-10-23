//! Hard errors from `gyeoldaero-core`. Content problems are diagnostics, not errors.

use std::path::PathBuf;

/// A failure that stops loading a notebook or its configuration. Problems below the
/// notebook root are diagnostics, not errors.
///
/// Messages don't repeat the source error; print the chain (`{:#}` with `anyhow`).
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The notebook root doesn't exist, isn't a directory, or can't be read.
    #[error("cannot read notebook root {path}")]
    UnreadableRoot {
        /// The root that was given.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// `gyeoldaero.toml` exists but can't be read.
    #[error("cannot read {path}")]
    Io {
        /// The file or directory being read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// `gyeoldaero.toml` can't be parsed or holds a value gyeoldaero refuses.
    #[error("{path}: {message}")]
    Config {
        /// Path of the config file.
        path: PathBuf,
        /// What was wrong with it.
        message: String,
    },
}
