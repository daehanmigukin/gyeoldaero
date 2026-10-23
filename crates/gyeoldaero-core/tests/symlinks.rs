//! PRD-001 AC5: symlinks are not followed and produce a warning.

#![cfg(unix)]

mod common;

use std::os::unix::fs::symlink;

use common::{TempNotebook, all_paths, errors, has_diag, show};
use gyeoldaero_core::Level;

const SECRET: &str = "outside-the-notebook-secret";

/// A directory outside the notebook holding a note and an asset.
fn outside() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("index.md"), format!("{SECRET}\n")).expect("write");
    std::fs::write(dir.path().join("secret.md"), format!("{SECRET}\n")).expect("write");
    std::fs::write(dir.path().join("photo.png"), b"png").expect("write");
    std::fs::create_dir(dir.path().join("child")).expect("mkdir");
    std::fs::write(dir.path().join("child/index.md"), format!("{SECRET}\n")).expect("write");
    dir
}

// PRD-001 AC5
#[test]
fn symlinks_are_not_followed_and_warn() {
    // (case, link location in the notebook, link target relative to the outside dir)
    let cases: &[(&str, &str, &str)] = &[
        ("directory symlink at root", "linked", ""),
        ("directory symlink in a note", "note/linked", ""),
        ("symlinked markdown file", "note/linked.md", "secret.md"),
        ("symlinked asset", "note/linked.png", "photo.png"),
    ];

    for (case, link, target) in cases {
        let ext = outside();
        let nb = TempNotebook::new();
        nb.note("note", "Note.\n");
        symlink(ext.path().join(target), nb.path(link)).expect("create symlink");

        let (notebook, diags) = nb.load();
        let link_name = link.rsplit('/').next().unwrap_or(link);
        assert_eq!(
            all_paths(&notebook),
            ["note"],
            "{case}: symlink became a note"
        );
        for id in common::all_notes(&notebook) {
            let note = notebook.get(id);
            assert!(
                !note.body.contains(SECRET),
                "{case}: target content was read"
            );
            assert!(
                note.assets.iter().all(|a| !a.ends_with(link_name)),
                "{case}: symlink listed as an asset"
            );
        }
        assert!(
            has_diag(&diags, Level::Warning, &[link_name]),
            "{case}: expected a warning naming {link_name}, got {:?}",
            show(&diags)
        );
        assert!(errors(&diags).is_empty(), "{case}: {:?}", show(&diags));
    }
}

// PRD-001 AC5
#[test]
fn symlinked_index_md_is_not_read() {
    let ext = outside();
    let nb = TempNotebook::new();
    nb.dir("note");
    symlink(ext.path().join("index.md"), nb.path("note/index.md")).expect("symlink");

    let (notebook, diags) = nb.load();
    let note = notebook.get(notebook.by_path("note").expect("folder is still a note"));
    assert!(
        !note.body.contains(SECRET),
        "symlinked index.md was followed"
    );
    assert!(
        has_diag(&diags, Level::Warning, &["index.md"]),
        "expected a warning about the symlinked index.md, got {:?}",
        show(&diags)
    );
}

// PRD-001 AC5
#[test]
fn symlink_loop_terminates_with_a_warning() {
    let nb = TempNotebook::new();
    nb.note("a", "A.\n");
    symlink(nb.root(), nb.path("a/loop")).expect("symlink to root");

    let (notebook, diags) = nb.load();
    assert_eq!(all_paths(&notebook), ["a"]);
    assert!(
        has_diag(&diags, Level::Warning, &["loop"]),
        "{:?}",
        show(&diags)
    );
}
