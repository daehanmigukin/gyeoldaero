//! PRD-001 AC2: folder names → order prefix + slug, and sibling ordering.

mod common;

use common::{TempNotebook, child_slugs};

// PRD-001 AC2
#[test]
fn siblings_are_ordered_prefixed_then_natural_then_by_slug() {
    // (case, folder names on disk, expected child slugs in model order)
    let cases: &[(&str, &[&str], &[&str])] = &[
        (
            "prefixed before unprefixed",
            &["alpha", "020-zeta", "beta", "010-omega"],
            &["omega", "zeta", "alpha", "beta"],
        ),
        (
            "prefixes compare as numbers, not strings",
            &["100-c", "3-a", "020-b", "9999-d"],
            &["a", "b", "c", "d"],
        ),
        (
            "unprefixed use natural sort",
            &["note-10", "note-2", "note-1", "note-20"],
            &["note-1", "note-2", "note-10", "note-20"],
        ),
        (
            "natural sort with several number runs",
            &["v1-10", "v1-9", "v2-1", "v10-1"],
            &["v1-9", "v1-10", "v2-1", "v10-1"],
        ),
        (
            "equal prefix numbers break ties by slug",
            &["010-beta", "10-alpha", "0010-gamma"],
            &["alpha", "beta", "gamma"],
        ),
        (
            "five digits is not a prefix",
            &["12345-x", "001-y"],
            &["y", "12345-x"],
        ),
        (
            "digits without a dash are not a prefix",
            &["10x", "2-y"],
            &["y", "10x"],
        ),
    ];

    for (case, dirs, expected) in cases {
        let nb = TempNotebook::new();
        for d in *dirs {
            nb.note(d, "Body.\n");
        }
        let (notebook, diags) = nb.load();
        assert!(
            diags.is_empty(),
            "{case}: unexpected diagnostics {:?}",
            common::show(&diags)
        );
        assert_eq!(
            child_slugs(&notebook, notebook.root()),
            *expected,
            "{case}: wrong sibling order"
        );
    }
}

// PRD-001 AC2
#[test]
fn folder_name_splits_into_order_and_slug() {
    // (folder name, expected order, expected slug)
    let cases: &[(&str, Option<u32>, &str)] = &[
        ("020-caches", Some(20), "caches"),
        ("1-a", Some(1), "a"),
        ("0000-zero", Some(0), "zero"),
        ("9999-max", Some(9999), "max"),
        ("caches", None, "caches"),
        ("12345-five-digits", None, "12345-five-digits"),
        ("2024notes", None, "2024notes"),
    ];

    for (dir, order, slug) in cases {
        let nb = TempNotebook::new();
        nb.note(dir, "Body.\n");
        let (notebook, _) = nb.load();
        let child = notebook.children(notebook.root())[0];
        let note = notebook.get(child);
        assert_eq!(note.order, *order, "{dir}: order");
        assert_eq!(note.slug, *slug, "{dir}: slug");
        assert_eq!(note.dir_name, *dir, "{dir}: dir_name");
        assert_eq!(note.path.as_str(), *slug, "{dir}: path uses the slug");
        assert_eq!(notebook.by_path(slug), Some(child), "{dir}: by_path(slug)");
        assert_eq!(
            notebook.parent(child),
            Some(notebook.root()),
            "{dir}: parent"
        );
    }
}

// PRD-001 AC2
#[test]
fn ordering_applies_at_every_depth() {
    let nb = TempNotebook::new();
    nb.note("topic", "Topic.\n")
        .note("topic/note-10", "x\n")
        .note("topic/note-9", "x\n")
        .note("topic/005-first", "x\n");
    let (notebook, diags) = nb.load();
    assert!(diags.is_empty(), "{:?}", common::show(&diags));
    let topic = notebook.by_path("topic").expect("topic exists");
    assert_eq!(
        child_slugs(&notebook, topic),
        ["first", "note-9", "note-10"]
    );
    assert_eq!(
        notebook
            .get(
                notebook
                    .by_path("topic/first")
                    .expect("prefix dropped in path")
            )
            .path
            .as_str(),
        "topic/first"
    );
}
