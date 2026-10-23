//! Content problems found while loading a notebook.

use std::fmt;
use std::path::PathBuf;

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
    /// That note's folder relative to the notebook root, as on disk (empty for the root).
    /// Tells apart notes whose slug paths clash.
    pub dir: PathBuf,
    /// A plain sentence saying what is wrong and how to fix it.
    pub message: String,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Level::Error => "error",
            Level::Warning => "warning",
        })
    }
}

/// `error: cpu-architecture/caches: <message>`; the root note shows as `/`.
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}: {}",
            self.level,
            self.path.display_name(),
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_shows_level_path_and_message() {
        let d = Diagnostic {
            level: Level::Warning,
            path: NotePath::default(),
            dir: PathBuf::new(),
            message: "Something.".to_owned(),
        };
        assert_eq!(d.to_string(), "warning: /: Something.");
    }
}
