//! Content problems found while loading a notebook.

use std::fmt;

use crate::notebook::NotePath;

/// How serious a diagnostic is. `gyeoldaero check` exits non-zero on any `Error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// A problem that makes the notebook wrong (broken link, duplicate slug, …).
    Error,
    /// Something probably unintended (missing `index.md`, stray `.md`, symlink, …).
    Warning,
}

/// One problem in a notebook, attached to the note it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Error or warning.
    pub level: Level,
    /// The note the problem belongs to.
    pub path: NotePath,
    /// A plain sentence saying what is wrong and how to fix it.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("PRD-001: Diagnostic Display")
    }
}
