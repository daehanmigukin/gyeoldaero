//! The loaded notebook tree: scanning, ordering, duplicate detection and link resolution.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::NOTE_FILE;
use crate::config::Config;
use crate::diagnostic::{Diagnostic, Level};
use crate::error::CoreError;
use crate::frontmatter;
use crate::fsread::{self, Contents, MAX_FILE_BYTES};
use crate::ignore::is_ignored;
use crate::links::{self, Link};
use crate::name;

/// Deepest nesting below the root that is scanned; deeper folders are skipped with an
/// error. Also bounds the scanner's recursion.
pub const MAX_DEPTH: usize = 64;

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
    /// Folder relative to the notebook root, as on disk (empty for the root).
    pub dir: PathBuf,
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

impl Note {
    /// A note for a folder, before its `index.md` is read.
    fn folder(
        dir_name: String,
        order: Option<u32>,
        slug: String,
        dir: PathBuf,
        path: NotePath,
    ) -> Note {
        Note {
            slug,
            order,
            dir_name,
            dir,
            path,
            title: None,
            body: String::new(),
            frontmatter: serde_yaml_ng::Mapping::new(),
            links: Vec::new(),
            assets: Vec::new(),
        }
    }

    /// A diagnostic attached to this note.
    fn diagnostic(&self, level: Level, message: String) -> Diagnostic {
        Diagnostic {
            level,
            path: self.path.clone(),
            dir: self.dir.clone(),
            message,
        }
    }
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
    /// Scans `root` into a model. Content problems, including I/O errors below the root,
    /// come back as diagnostics; only an unusable root is an error.
    pub fn load(root: &Path, config: &Config) -> Result<(Notebook, Vec<Diagnostic>), CoreError> {
        let unusable = |source| CoreError::UnreadableRoot {
            path: root.to_path_buf(),
            source,
        };
        if !fs::metadata(root).map_err(unusable)?.is_dir() {
            return Err(unusable(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "not a directory",
            )));
        }
        // The root itself may be a symlink (e.g. `~/notes`): it is given by the user, not
        // found inside the notebook.
        let root_entries = read_entries(root).map_err(unusable)?;
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
        let root_note = Note::folder(
            root_name,
            None,
            String::new(),
            PathBuf::new(),
            NotePath::default(),
        );
        let root_id = scanner.add(root_note, None);
        scanner.scan_entries(root_id, root, 0, root_entries);

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
                let (level, message) = match self.resolve(from, &link.target) {
                    Some((to, used_prefix)) => {
                        link.resolved = Some(to);
                        if !used_prefix {
                            continue;
                        }
                        let slug_path = self.notes[to.0].path.display_name();
                        (
                            Level::Warning,
                            format!(
                                "The link [[{}]] uses order prefixes. Link by slug instead, e.g. [[{slug_path}]]: prefixed links break when notes are renumbered.",
                                link.target
                            ),
                        )
                    }
                    None => (
                        Level::Error,
                        format!(
                            "The link [[{}]] doesn't point to any note. Check the slugs in the path (relative links start with ./ or ../), or create the note.",
                            link.target
                        ),
                    ),
                };
                diagnostics.push(self.notes[i].diagnostic(level, message));
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

/// A folder's entries without following symlinks, sorted by name. The outer error means
/// the folder itself can't be read; per-entry errors come back inside.
type Entries = Vec<std::io::Result<(OsString, fs::FileType)>>;

fn read_entries(dir: &Path) -> std::io::Result<Entries> {
    let mut entries: Entries = fs::read_dir(dir)?
        .map(|entry| {
            let entry = entry?;
            Ok((entry.file_name(), entry.file_type()?))
        })
        .collect();
    entries.sort_by(|a, b| match (a, b) {
        (Ok(a), Ok(b)) => a.0.cmp(&b.0),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => std::cmp::Ordering::Equal,
    });
    Ok(entries)
}

/// A child folder found while scanning, before it becomes a note.
struct Subfolder {
    dir_name: String,
    order: Option<u32>,
    slug: String,
    /// Path to read it from.
    path: PathBuf,
    /// Relative to the root, as on disk.
    rel: PathBuf,
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
        let diagnostic = self.notebook.notes[id.0].diagnostic(level, message);
        self.diagnostics.push(diagnostic);
    }

    /// Scans a folder below the root. Problems reading it are diagnostics on its note.
    fn scan(&mut self, id: NoteId, dir: &Path, depth: usize) {
        // Narrows (doesn't close) the race of the folder being swapped for a symlink since
        // its parent was read. See PRD-001 Notes.
        match fs::symlink_metadata(dir) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                self.diag(
                    id,
                    Level::Warning,
                    "This folder was replaced by a symbolic link or a file during the scan and is not read (gyeoldaero doesn't follow links).".to_owned(),
                );
                return;
            }
            Err(e) => {
                self.diag(
                    id,
                    Level::Error,
                    format!("This folder can't be read ({e}). Check its permissions."),
                );
                return;
            }
        }
        match read_entries(dir) {
            Ok(entries) => self.scan_entries(id, dir, depth, entries),
            Err(e) => self.diag(
                id,
                Level::Error,
                format!("This folder can't be read ({e}), so its body and notes are missing. Check its permissions."),
            ),
        }
    }

    /// Reads one note folder's entries: its body, assets, problems, then its children.
    fn scan_entries(&mut self, id: NoteId, dir: &Path, depth: usize, entries: Entries) {
        let rel = self.notebook.notes[id.0].dir.clone();
        let mut index = None;
        let mut index_reported = false;
        let mut subfolders = Vec::new();
        let mut too_deep = Vec::new();
        for entry in entries {
            let (os_name, file_type) = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    self.diag(
                        id,
                        Level::Error,
                        format!(
                            "An entry in this folder can't be read ({e}). Check its permissions."
                        ),
                    );
                    continue;
                }
            };
            // Ignore rules first: every pattern is ASCII, so the lossy name is enough, and
            // an ignored temp file with a non-UTF-8 name stays ignored.
            if is_ignored(&os_name.to_string_lossy()) {
                continue;
            }
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
            let path = dir.join(entry_name);
            if file_type.is_symlink() {
                let message = if entry_name == NOTE_FILE {
                    index_reported = true;
                    format!(
                        "{NOTE_FILE} is a symbolic link and is not read (gyeoldaero doesn't follow links), so this note has an empty body. Replace it with a regular file."
                    )
                } else {
                    format!(
                        "{entry_name} is a symbolic link and is skipped (gyeoldaero doesn't follow links). Replace it with the real folder or file."
                    )
                };
                self.diag(id, Level::Warning, message);
                continue;
            }
            if file_type.is_dir() {
                let child_rel = rel.join(entry_name);
                if self.output_dirs.contains(&child_rel) {
                    continue;
                }
                if depth + 1 > MAX_DEPTH {
                    too_deep.push(entry_name.to_owned());
                    continue;
                }
                let (order, slug) = name::split_prefix(entry_name);
                if slug.starts_with('.') {
                    self.diag(
                        id,
                        Level::Error,
                        format!("Folder {entry_name} gives the slug `{slug}`, which can't be used: slugs can't start with a dot. Rename the folder; it is skipped until then."),
                    );
                    continue;
                }
                subfolders.push(Subfolder {
                    dir_name: entry_name.to_owned(),
                    order,
                    slug: slug.to_owned(),
                    path,
                    rel: child_rel,
                });
                continue;
            }
            if entry_name == NOTE_FILE {
                if file_type.is_file() {
                    index = Some(path);
                } else {
                    index_reported = true;
                    self.diag(id, Level::Warning, not_regular_message());
                }
                continue;
            }
            if !file_type.is_file() {
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
                    format!("{entry_name} is ignored: the note file must be named {NOTE_FILE} exactly, in lowercase. Rename it."),
                ),
                Some("md") => {
                    let stem = entry_name.get(..entry_name.len().saturating_sub(3)).unwrap_or(entry_name);
                    self.diag(
                        id,
                        Level::Warning,
                        format!("{entry_name} is ignored: only {NOTE_FILE} is read. If it is meant to be a note, move it to {stem}/{NOTE_FILE}."),
                    );
                }
                Some(ext) if ASSET_EXTENSIONS.contains(&ext) => {
                    self.notebook.notes[id.0].assets.push(path);
                }
                _ => {}
            }
        }

        match index {
            Some(path) => self.read_body(id, &path),
            None if !index_reported => self.missing_index(id),
            None => {}
        }
        if !too_deep.is_empty() {
            self.diag(
                id,
                Level::Error,
                format!(
                    "Folders {} are nested deeper than {MAX_DEPTH} levels below the notebook root and are skipped, with everything inside them. Move them higher up.",
                    join_names(&too_deep)
                ),
            );
        }
        for (child, path) in self.add_children(id, subfolders) {
            self.scan(child, &path, depth + 1);
        }
    }

    /// Adds subfolders as children in sibling order, reports duplicate slugs, and returns
    /// each new child with the path to scan it from.
    fn add_children(
        &mut self,
        id: NoteId,
        mut subfolders: Vec<Subfolder>,
    ) -> Vec<(NoteId, PathBuf)> {
        subfolders.sort_by(|a, b| {
            name::sibling_cmp(
                (a.order, &a.slug, &a.dir_name),
                (b.order, &b.slug, &b.dir_name),
            )
        });
        let parent_path = self.notebook.notes[id.0].path.clone();
        let mut added = Vec::new();
        let mut clashes: Vec<(String, Vec<String>)> = Vec::new();
        for sub in subfolders {
            let key = name::key(&sub.slug);
            let dir_key = sub.order.map(|_| name::key(&sub.dir_name));
            let dir_name = sub.dir_name.clone();
            let note = Note::folder(
                sub.dir_name,
                sub.order,
                sub.slug.clone(),
                sub.rel,
                parent_path.child(&sub.slug),
            );
            let child = self.add(note, Some(id));
            self.notebook.children[id.0].push(child);
            let first = *self.notebook.by_slug[id.0]
                .entry(key.clone())
                .or_insert(child);
            if first != child {
                match clashes.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, names)) => names.push(dir_name),
                    None => {
                        let first_dir = self.notebook.notes[first.0].dir_name.clone();
                        clashes.push((key, vec![first_dir, dir_name]));
                    }
                }
            }
            if let Some(dir_key) = dir_key {
                self.notebook.by_dir_name[id.0]
                    .entry(dir_key)
                    .or_insert(child);
            }
            added.push((child, sub.path));
        }
        for (_, names) in clashes {
            let slug = names.first().map_or("", |n| name::split_prefix(n).1);
            self.diag(
                id,
                Level::Error,
                format!(
                    "Folders {} have the same slug `{slug}` (slugs are compared ignoring case and Unicode normalization), so links and URLs can't tell them apart. Rename or merge them.",
                    join_names(&names)
                ),
            );
        }
        added
    }

    fn missing_index(&mut self, id: NoteId) {
        let shown = self.notebook.notes[id.0].path.display_name().to_owned();
        self.diag(
            id,
            Level::Warning,
            format!("{shown} has no {NOTE_FILE}, so it is shown with an empty body. Add an {NOTE_FILE} to give it one."),
        );
    }

    fn read_body(&mut self, id: NoteId, path: &Path) {
        let bytes = match fsread::read_regular(path, MAX_FILE_BYTES) {
            Ok(Contents::Bytes(bytes)) => bytes,
            Ok(Contents::NotRegular) => {
                self.diag(id, Level::Warning, not_regular_message());
                return;
            }
            Ok(Contents::TooLarge) => {
                self.diag(
                    id,
                    Level::Error,
                    format!("{NOTE_FILE} is larger than 8 MiB and is not read, so this note has an empty body. Split it into smaller notes."),
                );
                return;
            }
            // Deleted between listing and reading (an editor or sync tool saving).
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.missing_index(id);
                return;
            }
            Err(e) => {
                self.diag(
                    id,
                    Level::Error,
                    format!("{NOTE_FILE} can't be read ({e}), so this note has an empty body. Check its permissions."),
                );
                return;
            }
        };
        let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes);
        let text = match std::str::from_utf8(bytes) {
            Ok(text) => text.to_owned(),
            Err(_) => {
                self.diag(
                    id,
                    Level::Warning,
                    format!("{NOTE_FILE} isn't valid UTF-8; unreadable bytes are shown as replacement characters. Save it as UTF-8."),
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
    }
}

fn not_regular_message() -> String {
    format!(
        "{NOTE_FILE} is not a regular file (it may be a pipe, or was replaced by a link while being read) and is not read, so this note has an empty body. Replace it with a regular file."
    )
}

/// `a and b`, `a, b and c`.
fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}
