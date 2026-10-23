//! Helpers shared by the PRD-001 model tests: build small notebooks in temp dirs and
//! query the loaded tree.

#![allow(
    dead_code,
    reason = "shared by several test binaries; each uses a subset"
)]

use std::fs;
use std::path::{Path, PathBuf};

use gyeoldaero_core::{Config, Diagnostic, Level, NoteId, Notebook};
use tempfile::TempDir;

/// A notebook in a temporary directory. The root gets an `index.md` so tests that don't
/// care about the root see no "missing index.md" warning for it.
pub struct TempNotebook {
    dir: TempDir,
}

impl TempNotebook {
    pub fn new() -> Self {
        let nb = TempNotebook {
            dir: tempfile::tempdir().expect("create temp dir"),
        };
        nb.file("index.md", "Root.\n");
        nb
    }

    /// An empty directory with no root `index.md`.
    pub fn empty() -> Self {
        TempNotebook {
            dir: tempfile::tempdir().expect("create temp dir"),
        }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    /// Writes a file, creating parent directories.
    pub fn file(&self, rel: &str, contents: &str) -> &Self {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(&p, contents).expect("write file");
        self
    }

    /// Creates a note folder (relative path) with an `index.md` holding `body`.
    pub fn note(&self, rel: &str, body: &str) -> &Self {
        self.file(&format!("{rel}/index.md"), body)
    }

    /// Creates a directory without `index.md`.
    pub fn dir(&self, rel: &str) -> &Self {
        fs::create_dir_all(self.path(rel)).expect("create dir");
        self
    }

    /// Loads with the default config for this root.
    pub fn load(&self) -> (Notebook, Vec<Diagnostic>) {
        let config = Config::load(self.root()).expect("default config loads");
        self.load_with(&config)
    }

    pub fn load_with(&self, config: &Config) -> (Notebook, Vec<Diagnostic>) {
        Notebook::load(self.root(), config).expect("notebook loads")
    }
}

/// Slugs of a note's children, in model order.
pub fn child_slugs(nb: &Notebook, id: NoteId) -> Vec<String> {
    nb.children(id)
        .iter()
        .map(|c| nb.get(*c).slug.clone())
        .collect()
}

/// Every note id in the tree, depth first from the root.
pub fn all_notes(nb: &Notebook) -> Vec<NoteId> {
    let mut out = Vec::new();
    let mut stack = vec![nb.root()];
    while let Some(id) = stack.pop() {
        out.push(id);
        stack.extend(nb.children(id).iter().rev().copied());
    }
    out
}

/// Slug paths (`a/b`) of every note except the root.
pub fn all_paths(nb: &Notebook) -> Vec<String> {
    all_notes(nb)
        .into_iter()
        .filter(|id| *id != nb.root())
        .map(|id| nb.get(id).path.as_str().to_owned())
        .collect()
}

pub fn errors(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags.iter().filter(|d| d.level == Level::Error).collect()
}

pub fn warnings(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags.iter().filter(|d| d.level == Level::Warning).collect()
}

/// Display strings of all diagnostics, for assertion messages.
pub fn show(diags: &[Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .map(|d| format!("{:?} [{}] {}", d.level, d.path, d.message))
        .collect()
}

/// True if some diagnostic of `level` mentions every needle in its note path or its
/// Display output (which may or may not repeat the path).
pub fn has_diag(diags: &[Diagnostic], level: Level, needles: &[&str]) -> bool {
    diags.iter().any(|d| {
        let text = format!("{} {}", d.path, d);
        d.level == level && needles.iter().all(|n| text.contains(n))
    })
}

/// Body without one trailing newline, so `"text\n"` on disk compares equal to `"text"`.
pub fn body_text(body: &str) -> &str {
    body.strip_suffix('\n').unwrap_or(body)
}
