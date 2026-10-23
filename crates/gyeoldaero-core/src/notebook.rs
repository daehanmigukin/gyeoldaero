//! The loaded notebook tree.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::error::CoreError;
use crate::links::Link;

/// Handle to a note inside a [`Notebook`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteId(usize);

/// A note's path from the notebook root, as slugs joined by `/` (no order prefixes).
/// The root's path is empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct NotePath(String);

impl NotePath {
    /// The path as a string, e.g. `cpu-architecture/caches`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NotePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One note: a folder with an optional `index.md`.
#[derive(Debug, Clone)]
pub struct Note {
    /// Folder name without the order prefix, as written.
    pub slug: String,
    /// Order prefix (`020-caches` → 20), if any.
    pub order: Option<u32>,
    /// Folder name as on disk.
    pub dir_name: String,
    /// Path from the root.
    pub path: NotePath,
    /// `title` from frontmatter.
    pub title: Option<String>,
    /// Markdown body (the file without its frontmatter).
    pub body: String,
    /// All frontmatter fields, unknown ones included.
    pub frontmatter: serde_yaml_ng::Mapping,
    /// Outgoing wiki links in body order.
    pub links: Vec<Link>,
    /// Asset files of this note.
    pub assets: Vec<PathBuf>,
}

/// A scanned notebook.
///
/// Stub (PRD-001 tests): every method is `todo!()` until implementation.
#[derive(Debug)]
pub struct Notebook {
    notes: Vec<Note>,
}

impl Notebook {
    /// Scans `root` into a model. Content problems come back as diagnostics; only an
    /// unusable root or I/O failure is an error.
    pub fn load(_root: &Path, _config: &Config) -> Result<(Notebook, Vec<Diagnostic>), CoreError> {
        todo!("PRD-001: Notebook::load")
    }

    /// The root note.
    pub fn root(&self) -> NoteId {
        todo!("PRD-001: Notebook::root")
    }

    /// The note behind an id.
    pub fn get(&self, _id: NoteId) -> &Note {
        let _ = &self.notes;
        todo!("PRD-001: Notebook::get")
    }

    /// Children in sibling order.
    pub fn children(&self, _id: NoteId) -> &[NoteId] {
        todo!("PRD-001: Notebook::children")
    }

    /// Parent, or `None` for the root.
    pub fn parent(&self, _id: NoteId) -> Option<NoteId> {
        todo!("PRD-001: Notebook::parent")
    }

    /// Looks a note up by slug path (`cpu-architecture/caches`), comparing NFC +
    /// lowercase keys.
    pub fn by_path(&self, _path: &str) -> Option<NoteId> {
        todo!("PRD-001: Notebook::by_path")
    }
}
