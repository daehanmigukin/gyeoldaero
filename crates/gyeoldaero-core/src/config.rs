//! `gyeoldaero.toml` in the notebook root.

use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// Notebook configuration. Every field has a default; the file itself is optional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Shown at the root of the path bar. Default: the notebook folder's name.
    pub name: String,
    /// `[serve]` section.
    pub serve: ServeConfig,
    /// `[build]` section.
    pub build: BuildConfig,
}

/// `[serve]` section of `gyeoldaero.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeConfig {
    /// Address to bind. Default `127.0.0.1:4321`.
    pub bind: String,
}

/// `[build]` section of `gyeoldaero.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfig {
    /// Output directory, `~` expanded. Default under the XDG cache dir. Ignored by the
    /// scanner when it sits inside the notebook.
    pub out: PathBuf,
}

impl Config {
    /// Reads `<root>/gyeoldaero.toml`, or returns defaults when it is absent.
    pub fn load(_root: &Path) -> Result<Config, CoreError> {
        todo!("PRD-001: Config::load")
    }
}
