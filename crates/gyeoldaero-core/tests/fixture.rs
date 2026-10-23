//! PRD-001 AC1: the fixture notebook loads cleanly and matches the mockup's `TREE`.
//!
//! The expected tree is read from `docs/reference/mockup.html` at test time, so the
//! mockup stays the single source of truth.
//!
//! Comparison rules:
//! - Bodies: fixture `index.md` files are the mockup body plus one trailing newline. The
//!   model body is compared with one trailing `\n` removed (if present), so the test
//!   holds whether or not the loader keeps the file's final newline.
//! - Sibling order follows AC2, not the mockup's hand-picked order. Where every sibling
//!   is prefixed (the root, `cpu-architecture`) the prefixes reproduce the mockup's
//!   order, so it is compared exactly. Elsewhere siblings are unprefixed and sort
//!   naturally; the fixture's slugs there have no digits, so natural sort is plain
//!   string order.
//! - The root's own slug is not compared: the mockup calls it `notebook`, and the
//!   architecture doc doesn't say what the root note's slug is.

mod common;

use std::path::PathBuf;

use common::{body_text, show};
use gyeoldaero_core::{Config, NoteId, Notebook};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ---------------------------------------------------------------------------
// Test-only parser for the mockup's `TREE` literal: objects with identifier keys,
// single-quoted strings, template literals (backslash escapes, no `${…}`), arrays.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct MockNote {
    slug: String,
    body: String,
    children: Vec<MockNote>,
}

enum Js {
    Str(String),
    Arr(Vec<Js>),
    Obj(Vec<(String, Js)>),
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while self.chars.peek().is_some_and(|c| c.is_whitespace()) {
            self.chars.next();
        }
    }

    fn expect(&mut self, want: char) {
        self.skip_ws();
        let got = self.chars.next();
        assert_eq!(got, Some(want), "TREE parser: expected {want:?}");
    }

    fn value(&mut self) -> Js {
        self.skip_ws();
        match self.chars.peek().copied() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some(q @ ('\'' | '"' | '`')) => {
                self.chars.next();
                Js::Str(self.string(q))
            }
            other => panic!("TREE parser: unexpected {other:?}"),
        }
    }

    fn string(&mut self, quote: char) -> String {
        let mut out = String::new();
        loop {
            match self.chars.next().expect("TREE parser: unterminated string") {
                '\\' => match self.chars.next().expect("TREE parser: dangling escape") {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    c => out.push(c),
                },
                '$' if quote == '`' && self.chars.peek() == Some(&'{') => {
                    panic!("TREE parser: template substitutions are not supported")
                }
                c if c == quote => return out,
                c => out.push(c),
            }
        }
    }

    fn array(&mut self) -> Js {
        self.expect('[');
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.peek() == Some(&']') {
                self.chars.next();
                return Js::Arr(items);
            }
            items.push(self.value());
            self.skip_ws();
            if self.chars.peek() == Some(&',') {
                self.chars.next();
            }
        }
    }

    fn object(&mut self) -> Js {
        self.expect('{');
        let mut fields = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.peek() == Some(&'}') {
                self.chars.next();
                return Js::Obj(fields);
            }
            let mut key = String::new();
            while self
                .chars
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
            {
                key.push(self.chars.next().unwrap_or_default());
            }
            assert!(!key.is_empty(), "TREE parser: expected a key");
            self.expect(':');
            fields.push((key, self.value()));
            self.skip_ws();
            if self.chars.peek() == Some(&',') {
                self.chars.next();
            }
        }
    }
}

fn to_mock(js: Js) -> MockNote {
    let Js::Obj(fields) = js else {
        panic!("TREE parser: note is not an object")
    };
    let mut note = MockNote {
        slug: String::new(),
        body: String::new(),
        children: Vec::new(),
    };
    for (key, value) in fields {
        match (key.as_str(), value) {
            ("slug", Js::Str(s)) => note.slug = s,
            ("body", Js::Str(s)) => note.body = s,
            ("children", Js::Arr(items)) => {
                note.children = items.into_iter().map(to_mock).collect()
            }
            (k, _) => panic!("TREE parser: unexpected field {k}"),
        }
    }
    note
}

fn mockup_tree() -> MockNote {
    let html = std::fs::read_to_string(repo_root().join("docs/reference/mockup.html"))
        .expect("read mockup.html");
    let start = html.find("const TREE =").expect("mockup defines TREE") + "const TREE =".len();
    let mut parser = Parser {
        chars: html[start..].chars().peekable(),
    };
    to_mock(parser.value())
}

// ---------------------------------------------------------------------------

fn load_fixture() -> (Notebook, Vec<gyeoldaero_core::Diagnostic>) {
    let root = repo_root().join("tests/fixtures/notebook");
    let config = Config::load(&root).expect("fixture config loads");
    Notebook::load(&root, &config).expect("fixture loads")
}

fn compare(nb: &Notebook, id: NoteId, mock: &MockNote, path: &str) {
    let note = nb.get(id);
    assert_eq!(body_text(&note.body), mock.body, "{path}: body");
    assert_eq!(note.title, None, "{path}: fixture has no frontmatter");
    assert!(note.frontmatter.is_empty(), "{path}: frontmatter");

    let children = nb.children(id);
    let got: Vec<&str> = children.iter().map(|c| nb.get(*c).slug.as_str()).collect();
    let mut want: Vec<&str> = mock.children.iter().map(|c| c.slug.as_str()).collect();
    let all_prefixed = children.iter().all(|c| nb.get(*c).order.is_some());
    if !all_prefixed {
        assert!(
            want.iter().all(|s| !s.chars().any(|c| c.is_ascii_digit())),
            "{path}: natural sort is plain string order only without digits"
        );
        want.sort_unstable();
    }
    assert_eq!(got, want, "{path}: children in AC2 order");

    for child_mock in &mock.children {
        let child = children
            .iter()
            .copied()
            .find(|c| nb.get(*c).slug == child_mock.slug)
            .unwrap_or_else(|| panic!("{path}: missing child {}", child_mock.slug));
        let child_path = if path.is_empty() {
            child_mock.slug.clone()
        } else {
            format!("{path}/{}", child_mock.slug)
        };
        let note = nb.get(child);
        assert_eq!(note.path.as_str(), child_path, "path");
        assert_eq!(
            nb.by_path(&child_path),
            Some(child),
            "{child_path}: by_path"
        );
        assert_eq!(nb.parent(child), Some(id), "{child_path}: parent");
        compare(nb, child, child_mock, &child_path);
    }
}

// PRD-001 AC1
#[test]
fn fixture_loads_with_zero_diagnostics() {
    let (_, diags) = load_fixture();
    assert!(diags.is_empty(), "{:?}", show(&diags));
}

// PRD-001 AC1
#[test]
fn fixture_tree_matches_mockup_tree() {
    let mock = mockup_tree();
    assert_eq!(mock.children.len(), 4, "sanity: mockup TREE parsed");
    let (nb, _) = load_fixture();
    assert_eq!(nb.parent(nb.root()), None);
    compare(&nb, nb.root(), &mock, "");
}

// PRD-001 AC1
#[test]
fn fixture_order_prefixes_match() {
    let (nb, _) = load_fixture();
    // (slug path, expected order, expected folder name)
    let prefixed: &[(&str, u32, &str)] = &[
        ("cpu-architecture", 10, "010-cpu-architecture"),
        ("korean-infosec-law", 20, "020-korean-infosec-law"),
        ("homelab", 30, "030-homelab"),
        ("inbox", 40, "040-inbox"),
        ("cpu-architecture/caches", 10, "010-caches"),
        ("cpu-architecture/pipelining", 20, "020-pipelining"),
        (
            "cpu-architecture/memory-ordering",
            30,
            "030-memory-ordering",
        ),
    ];
    for (path, order, dir) in prefixed {
        let note = nb.get(nb.by_path(path).unwrap_or_else(|| panic!("{path} exists")));
        assert_eq!(note.order, Some(*order), "{path}");
        assert_eq!(note.dir_name, *dir, "{path}");
    }
    for id in common::all_notes(&nb) {
        let note = nb.get(id);
        if prefixed.iter().any(|(p, _, _)| *p == note.path.as_str()) {
            continue;
        }
        assert_eq!(note.order, None, "{}: no prefix in fixture", note.path);
        if id != nb.root() {
            assert_eq!(note.dir_name, note.slug, "{}", note.path);
        }
    }
    assert_eq!(common::all_notes(&nb).len(), 23, "fixture has 23 notes");
}

// PRD-001 AC1, AC2
#[test]
fn fixture_sibling_order_follows_ac2() {
    let (nb, _) = load_fixture();
    // Prefixed: same as the mockup.
    assert_eq!(
        common::child_slugs(&nb, nb.root()),
        ["cpu-architecture", "korean-infosec-law", "homelab", "inbox"]
    );
    // Unprefixed: natural sort, which differs from the mockup's order.
    let law = nb
        .by_path("korean-infosec-law")
        .expect("korean-infosec-law");
    assert_eq!(
        common::child_slugs(&nb, law),
        ["credit-information-act", "network-act", "pipa"]
    );
    let pipelining = nb
        .by_path("cpu-architecture/pipelining")
        .expect("pipelining");
    assert_eq!(
        common::child_slugs(&nb, pipelining),
        ["branch-prediction", "hazards"]
    );
}

// PRD-001 AC1
#[test]
fn fixture_link_resolves() {
    let (nb, _) = load_fixture();
    let coherence = nb.get(
        nb.by_path("cpu-architecture/caches/cache-coherence")
            .expect("cache-coherence"),
    );
    assert_eq!(coherence.links.len(), 1);
    let link = &coherence.links[0];
    assert_eq!(link.target, "cpu-architecture/caches/false-sharing");
    assert_eq!(
        link.resolved,
        nb.by_path("cpu-architecture/caches/false-sharing")
    );
    assert!(link.resolved.is_some());
}

/// Finds the folder for `slug` among `dir`'s entries (`slug` or `NNN-slug`).
fn fixture_dir(dir: &std::path::Path, slug: &str) -> PathBuf {
    std::fs::read_dir(dir)
        .expect("read fixture dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned());
            name.is_some_and(|n| {
                n == slug
                    || n.split_once('-').is_some_and(|(pre, rest)| {
                        rest == slug && pre.chars().all(|c| c.is_ascii_digit())
                    })
            })
        })
        .unwrap_or_else(|| panic!("no fixture folder for {slug} in {}", dir.display()))
}

fn check_files(dir: &std::path::Path, mock: &MockNote, count: &mut usize) {
    *count += 1;
    let text = std::fs::read_to_string(dir.join("index.md")).expect("read index.md");
    assert_eq!(text, format!("{}\n", mock.body), "{}", dir.display());
    for child in &mock.children {
        check_files(&fixture_dir(dir, &child.slug), child, count);
    }
}

// PRD-001 AC1 (self-check of the test's TREE parser; independent of the model)
#[test]
fn mockup_tree_parser_matches_fixture_files() {
    let mock = mockup_tree();
    let mut count = 0;
    check_files(
        &repo_root().join("tests/fixtures/notebook"),
        &mock,
        &mut count,
    );
    assert_eq!(count, 23);
}
