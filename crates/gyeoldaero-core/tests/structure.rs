//! PRD-001 AC6: directories without `index.md` and stray `.md` files. Also covers the
//! asset rule from architecture.md (Requirement 1), which has no AC of its own.

mod common;

use common::{TempNotebook, body_text, errors, has_diag, show, warnings};
use gyeoldaero_core::Level;

// PRD-001 AC6
#[test]
fn directory_without_index_md_is_an_empty_note_with_a_warning() {
    // (case, directories to create without index.md, the one expected to warn)
    let cases: &[(&str, &str, &str)] = &[
        ("top-level folder", "empty", "empty"),
        ("nested folder", "topic/empty", "topic/empty"),
        ("folder holding only notes", "holder", "holder"),
    ];

    for (case, dir, warned) in cases {
        let nb = TempNotebook::new();
        nb.note("topic", "Topic.\n");
        nb.dir(dir);
        if *case == "folder holding only notes" {
            nb.note("holder/child", "Child.\n");
        }
        let (notebook, diags) = nb.load();
        let id = notebook
            .by_path(warned)
            .unwrap_or_else(|| panic!("{case}: {warned} should still be a note"));
        let note = notebook.get(id);
        assert_eq!(note.body, "", "{case}: body should be empty");
        assert_eq!(note.title, None, "{case}");
        assert!(note.links.is_empty(), "{case}");
        let leaf = warned.rsplit('/').next().unwrap_or(warned);
        assert!(
            has_diag(&diags, Level::Warning, &[leaf]),
            "{case}: expected a warning naming {leaf}, got {:?}",
            show(&diags)
        );
        assert_eq!(warnings(&diags).len(), 1, "{case}: {:?}", show(&diags));
        assert!(errors(&diags).is_empty(), "{case}: {:?}", show(&diags));
    }
}

// PRD-001 AC6
#[test]
fn root_without_index_md_is_an_empty_note_with_a_warning() {
    let nb = TempNotebook::empty();
    nb.note("topic", "Topic.\n");
    let (notebook, diags) = nb.load();
    assert_eq!(notebook.get(notebook.root()).body, "");
    assert_eq!(warnings(&diags).len(), 1, "{:?}", show(&diags));
    assert!(errors(&diags).is_empty(), "{:?}", show(&diags));
}

// PRD-001 AC6
#[test]
fn stray_markdown_files_warn_and_are_ignored() {
    // (case, stray file path)
    let cases: &[(&str, &str)] = &[
        ("at the root", "draft.md"),
        ("inside a note", "topic/README.md"),
        ("next to a missing index.md", "bare/notes.md"),
    ];

    for (case, file) in cases {
        let nb = TempNotebook::new();
        nb.note("topic", "Topic body.\n");
        nb.file(file, "Stray content that must not leak.\n");
        let (notebook, diags) = nb.load();
        let name = file.rsplit('/').next().unwrap_or(file);
        assert!(
            has_diag(&diags, Level::Warning, &[name]),
            "{case}: expected a warning naming {name}, got {:?}",
            show(&diags)
        );
        assert!(errors(&diags).is_empty(), "{case}: {:?}", show(&diags));
        for id in common::all_notes(&notebook) {
            let note = notebook.get(id);
            assert!(
                !note.body.contains("Stray content"),
                "{case}: stray file read as body"
            );
            assert!(
                note.assets.iter().all(|a| !a.ends_with(name)),
                "{case}: stray .md listed as an asset"
            );
        }
        assert_eq!(
            body_text(&notebook.get(notebook.by_path("topic").expect("topic")).body),
            "Topic body."
        );
    }
}

// PRD-001 Req1 (asset rule; no dedicated AC)
#[test]
fn assets_are_allowed_extensions_only() {
    let nb = TempNotebook::new();
    nb.note("topic", "Topic.\n");
    for f in [
        "a.png", "b.jpg", "c.jpeg", "d.gif", "e.webp", "f.avif", "g.pdf", "h.txt", "x.svg",
        "y.exe", "z.html",
    ] {
        nb.file(&format!("topic/{f}"), "data");
    }
    let (notebook, diags) = nb.load();
    assert!(errors(&diags).is_empty(), "{:?}", show(&diags));
    let mut names: Vec<String> = notebook
        .get(notebook.by_path("topic").expect("topic"))
        .assets
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "a.png", "b.jpg", "c.jpeg", "d.gif", "e.webp", "f.avif", "g.pdf", "h.txt"
        ]
    );
}
