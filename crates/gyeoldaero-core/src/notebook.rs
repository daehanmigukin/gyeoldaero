//! The loaded notebook tree: scanning, ordering, duplicate detection and link resolution.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::NOTE_FILE;
use crate::config::Config;
use crate::diagnostic::{Diagnostic, Level};
use crate::error::CoreError;
use crate::frontmatter;
use crate::ignore::is_ignored;
use crate::links::{self, Link};
use crate::name;

/// File extensions (lowercase) that make a file an asset of its note. No SVG in v1.
pub const ASSET_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "avif", "pdf", "txt"];

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

    /// The path for people to read: `/` for the root, otherwise the path itself.
    pub fn display_name(&self) -> &str {
        if self.0.is_empty() { "/" } else { &self.0 }
    }

    fn child(&self, slug: &str) -> NotePath {
        if self.0.is_empty() {
            NotePath(slug.to_owned())
        } else {
            NotePath(format!("{}/{slug}", self.0))
        }
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
#[derive(Debug)]
pub struct Notebook {
    notes: Vec<Note>,
    parents: Vec<Option<NoteId>>,
    children: Vec<Vec<NoteId>>,
    /// Per note: child slug key → child. The first in sibling order wins on duplicates.
    by_slug: Vec<HashMap<String, NoteId>>,
    /// Per note: key of a prefixed child's folder name → child, for prefixed links.
    by_dir_name: Vec<HashMap<String, NoteId>>,
}

impl Notebook {
    /// Scans `root` into a model. Content problems come back as diagnostics; only an
    /// unusable root or I/O failure is an error.
    pub fn load(root: &Path, config: &Config) -> Result<(Notebook, Vec<Diagnostic>), CoreError> {
        let meta = fs::metadata(root).map_err(|source| CoreError::UnreadableRoot {
            path: root.to_path_buf(),
            source,
        })?;
        if !meta.is_dir() {
            return Err(CoreError::UnreadableRoot {
                path: root.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotADirectory, "not a directory"),
            });
        }
        let root_name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .or_else(|| {
                let canonical = fs::canonicalize(root).ok()?;
                Some(canonical.file_name()?.to_string_lossy().into_owned())
            })
            .unwrap_or_default();

        let mut scanner = Scanner {
            notebook: Notebook {
                notes: Vec::new(),
                parents: Vec::new(),
                children: Vec::new(),
                by_slug: Vec::new(),
                by_dir_name: Vec::new(),
            },
            diagnostics: Vec::new(),
            output_dirs: output_dirs(root, &config.build.out),
        };
        let root_note = Note {
            slug: String::new(),
            order: None,
            dir_name: root_name,
            path: NotePath::default(),
            title: None,
            body: String::new(),
            frontmatter: serde_yaml_ng::Mapping::new(),
            links: Vec::new(),
            assets: Vec::new(),
        };
        let root_id = scanner.add(root_note, None);
        scanner.scan(root_id, root, Path::new(""))?;

        let Scanner {
            mut notebook,
            mut diagnostics,
            ..
        } = scanner;
        notebook.resolve_links(&mut diagnostics);
        Ok((notebook, diagnostics))
    }

    /// The root note.
    pub fn root(&self) -> NoteId {
        NoteId(0)
    }

    /// The note behind an id. Ids come from this notebook; a foreign id may panic.
    pub fn get(&self, id: NoteId) -> &Note {
        &self.notes[id.0]
    }

    /// Children in sibling order.
    pub fn children(&self, id: NoteId) -> &[NoteId] {
        &self.children[id.0]
    }

    /// Parent, or `None` for the root.
    pub fn parent(&self, id: NoteId) -> Option<NoteId> {
        self.parents[id.0]
    }

    /// Number of notes, the root included.
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    /// Always false: a notebook has at least its root note.
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// Every note, depth first from the root in sibling order.
    pub fn iter(&self) -> impl Iterator<Item = NoteId> + '_ {
        let mut stack = vec![self.root()];
        std::iter::from_fn(move || {
            let id = stack.pop()?;
            stack.extend(self.children(id).iter().rev().copied());
            Some(id)
        })
    }

    /// Looks a note up by slug path (`cpu-architecture/caches`), comparing NFC +
    /// lowercase keys. Order prefixes are not accepted here.
    pub fn by_path(&self, path: &str) -> Option<NoteId> {
        path.split('/')
            .filter(|s| !s.is_empty())
            .try_fold(self.root(), |id, segment| {
                self.by_slug[id.0].get(&name::key(segment)).copied()
            })
    }

    /// Resolves a link target written in note `from`. Returns the note and whether a
    /// segment needed its order prefix to match.
    fn resolve(&self, from: NoteId, target: &str) -> Option<(NoteId, bool)> {
        let relative = target == "."
            || target == ".."
            || target.starts_with("./")
            || target.starts_with("../");
        let mut id = if relative { from } else { self.root() };
        let mut used_prefix = false;
        for segment in target.split('/') {
            match segment.trim() {
                "" | "." => {}
                ".." => id = self.parent(id)?,
                segment => {
                    let key = name::key(segment);
                    id = match self.by_slug[id.0].get(&key) {
                        Some(child) => *child,
                        None => {
                            used_prefix = true;
                            *self.by_dir_name[id.0].get(&key)?
                        }
                    };
                }
            }
        }
        Some((id, used_prefix))
    }

    fn resolve_links(&mut self, diagnostics: &mut Vec<Diagnostic>) {
        for i in 0..self.notes.len() {
            let from = NoteId(i);
            let mut links = std::mem::take(&mut self.notes[i].links);
            for link in &mut links {
                let path = self.notes[i].path.clone();
                match self.resolve(from, &link.target) {
                    Some((to, used_prefix)) => {
                        link.resolved = Some(to);
                        if used_prefix {
                            let slug_path = self.notes[to.0].path.display_name().to_owned();
                            diagnostics.push(Diagnostic {
                                level: Level::Warning,
                                path,
                                message: format!(
                                    "The link [[{}]] uses order prefixes. Link by slug instead, e.g. [[{slug_path}]]: prefixed links break when notes are renumbered.",
                                    link.target
                                ),
                            });
                        }
                    }
                    None => diagnostics.push(Diagnostic {
                        level: Level::Error,
                        path,
                        message: format!(
                            "The link [[{}]] doesn't point to any note. Check the slugs in the path (relative links start with ./ or ../), or create the note.",
                            link.target
                        ),
                    }),
                }
            }
            self.notes[i].links = links;
        }
    }
}

/// Relative paths (from the root, as on disk) that the output dir may appear under.
fn output_dirs(root: &Path, out: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let pairs = [
        (fs::canonicalize(root).ok(), fs::canonicalize(out).ok()),
        (
            std::path::absolute(root).ok(),
            std::path::absolute(out).ok(),
        ),
    ];
    for (root, out) in pairs {
        if let (Some(root), Some(out)) = (root, out)
            && let Ok(rel) = out.strip_prefix(&root)
            && !rel.as_os_str().is_empty()
            && !found.iter().any(|f| f == rel)
        {
            found.push(rel.to_path_buf());
        }
    }
    found
}

struct Scanner {
    notebook: Notebook,
    diagnostics: Vec<Diagnostic>,
    output_dirs: Vec<PathBuf>,
}

impl Scanner {
    fn add(&mut self, note: Note, parent: Option<NoteId>) -> NoteId {
        let nb = &mut self.notebook;
        let id = NoteId(nb.notes.len());
        nb.notes.push(note);
        nb.parents.push(parent);
        nb.children.push(Vec::new());
        nb.by_slug.push(HashMap::new());
        nb.by_dir_name.push(HashMap::new());
        id
    }

    fn diag(&mut self, id: NoteId, level: Level, message: String) {
        let path = self.notebook.notes[id.0].path.clone();
        self.diagnostics.push(Diagnostic {
            level,
            path,
            message,
        });
    }

    /// Reads one note folder: its body, assets, problems, then its children.
    fn scan(&mut self, id: NoteId, dir: &Path, rel: &Path) -> Result<(), CoreError> {
        let io = |path: &Path| {
            let path = path.to_path_buf();
            move |source| CoreError::Io { path, source }
        };
        let mut entries = Vec::new();
        for entry in fs::read_dir(dir).map_err(io(dir))? {
            let entry = entry.map_err(io(dir))?;
            let file_type = entry.file_type().map_err(io(&entry.path()))?;
            entries.push((entry.file_name(), file_type));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut index = None;
        let mut index_symlinked = false;
        let mut subdirs = Vec::new();
        for (os_name, file_type) in entries {
            let Some(entry_name) = os_name.to_str() else {
                self.diag(
                    id,
                    Level::Error,
                    format!(
                        "{} is skipped: its name isn't valid UTF-8, so no link can point to it. Rename it.",
                        os_name.to_string_lossy()
                    ),
                );
                continue;
            };
            if is_ignored(entry_name) {
                continue;
            }
            let path = dir.join(entry_name);
            if file_type.is_symlink() {
                if entry_name == NOTE_FILE {
                    index_symlinked = true;
                    self.diag(
                        id,
                        Level::Warning,
                        format!(
                            "{NOTE_FILE} is a symbolic link and is not read (gyeoldaero doesn't follow links), so this note has an empty body. Replace it with a regular file."
                        ),
                    );
                } else {
                    self.diag(
                        id,
                        Level::Warning,
                        format!(
                            "{entry_name} is a symbolic link and is skipped (gyeoldaero doesn't follow links). Replace it with the real folder or file."
                        ),
                    );
                }
                continue;
            }
            if file_type.is_dir() {
                let child_rel = rel.join(entry_name);
                if !self.output_dirs.contains(&child_rel) {
                    subdirs.push((entry_name.to_owned(), path, child_rel));
                }
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            if entry_name == NOTE_FILE {
                index = Some(path);
                continue;
            }
            let ext = Path::new(entry_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase);
            match ext.as_deref() {
                Some("md") if entry_name.eq_ignore_ascii_case(NOTE_FILE) => self.diag(
                    id,
                    Level::Warning,
                    format!(
                        "{entry_name} is ignored: the note file must be named {NOTE_FILE} exactly, in lowercase. Rename it."
                    ),
                ),
                Some("md") => {
                    let stem = entry_name
                        .get(..entry_name.len().saturating_sub(3))
                        .unwrap_or(entry_name);
                    self.diag(
                        id,
                        Level::Warning,
                        format!(
                            "{entry_name} is ignored: only {NOTE_FILE} is read. If it is meant to be a note, move it to {stem}/{NOTE_FILE}."
                        ),
                    );
                }
                Some(ext) if ASSET_EXTENSIONS.contains(&ext) => {
                    self.notebook.notes[id.0].assets.push(path);
                }
                _ => {}
            }
        }

        match index {
            Some(path) => self.read_body(id, &path)?,
            None if !index_symlinked => {
                let shown = self.notebook.notes[id.0].path.display_name().to_owned();
                self.diag(
                    id,
                    Level::Warning,
                    format!(
                        "{shown} has no {NOTE_FILE}, so it is shown with an empty body. Add an {NOTE_FILE} to give it one."
                    ),
                );
            }
            None => {}
        }

        // Children in sibling order; then duplicates; then recurse.
        let parent_path = self.notebook.notes[id.0].path.clone();
        let mut kids: Vec<Note> = subdirs
            .iter()
            .map(|(dir_name, _, _)| {
                let (order, slug) = name::split_prefix(dir_name);
                Note {
                    slug: slug.to_owned(),
                    order,
                    dir_name: dir_name.clone(),
                    path: parent_path.child(slug),
                    title: None,
                    body: String::new(),
                    frontmatter: serde_yaml_ng::Mapping::new(),
                    links: Vec::new(),
                    assets: Vec::new(),
                }
            })
            .collect();
        let mut order: Vec<usize> = (0..kids.len()).collect();
        order.sort_by(|a, b| {
            let (x, y) = (&kids[*a], &kids[*b]);
            name::sibling_cmp(
                (x.order, &x.slug, &x.dir_name),
                (y.order, &y.slug, &y.dir_name),
            )
        });

        let mut placed = Vec::new();
        let mut clashes: HashMap<String, Vec<String>> = HashMap::new();
        let mut clash_order = Vec::new();
        for i in order {
            let note = std::mem::replace(&mut kids[i], empty_note());
            let key = name::key(&note.slug);
            let dir_key = note.order.map(|_| name::key(&note.dir_name));
            let dir_name = note.dir_name.clone();
            let child = self.add(note, Some(id));
            self.notebook.children[id.0].push(child);
            let first = self.notebook.by_slug[id.0]
                .entry(key.clone())
                .or_insert(child);
            if *first != child {
                let first_dir = self.notebook.notes[first.0].dir_name.clone();
                let names = clashes.entry(key.clone()).or_insert_with(|| {
                    clash_order.push(key);
                    vec![first_dir]
                });
                names.push(dir_name);
            }
            if let Some(dir_key) = dir_key {
                self.notebook.by_dir_name[id.0]
                    .entry(dir_key)
                    .or_insert(child);
            }
            placed.push((child, i));
        }
        for key in clash_order {
            let names = clashes.remove(&key).unwrap_or_default();
            let slug = name::split_prefix(names.first().map_or("", String::as_str)).1;
            self.diag(
                id,
                Level::Error,
                format!(
                    "Folders {} have the same slug `{slug}` (slugs are compared ignoring case and Unicode normalization), so links and URLs can't tell them apart. Rename or merge them.",
                    join_names(&names)
                ),
            );
        }
        for (child, i) in placed {
            let (_, path, child_rel) = &subdirs[i];
            self.scan(child, path, child_rel)?;
        }
        Ok(())
    }

    fn read_body(&mut self, id: NoteId, path: &Path) -> Result<(), CoreError> {
        let bytes = fs::read(path).map_err(|source| CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes);
        let text = match std::str::from_utf8(bytes) {
            Ok(text) => text.to_owned(),
            Err(_) => {
                self.diag(
                    id,
                    Level::Warning,
                    format!(
                        "{NOTE_FILE} isn't valid UTF-8; unreadable bytes are shown as replacement characters. Save it as UTF-8."
                    ),
                );
                String::from_utf8_lossy(bytes).into_owned()
            }
        };
        let text = if text.contains('\r') {
            text.replace("\r\n", "\n")
        } else {
            text
        };
        let split = frontmatter::split(&text);
        for (level, message) in split.problems {
            self.diag(id, level, message);
        }
        let note = &mut self.notebook.notes[id.0];
        note.title = split.title;
        note.frontmatter = split.frontmatter;
        note.links = links::extract(&split.body);
        note.body = split.body;
        Ok(())
    }
}

fn empty_note() -> Note {
    Note {
        slug: String::new(),
        order: None,
        dir_name: String::new(),
        path: NotePath::default(),
        title: None,
        body: String::new(),
        frontmatter: serde_yaml_ng::Mapping::new(),
        links: Vec::new(),
        assets: Vec::new(),
    }
}

/// `a and b`, `a, b and c`.
fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}
