//! PRD-001 AC11: Unicode and case — NFC + lowercase comparison keys (Requirement 8).

mod common;

use std::io::ErrorKind;

use common::{TempNotebook, child_slugs, errors, has_diag, show};
use gyeoldaero_core::Level;
use unicode_normalization::UnicodeNormalization;

fn nfd(s: &str) -> String {
    s.nfd().collect()
}

/// Creates a sibling note; returns false if the file system folds it onto an existing
/// name (macOS case- or normalization-insensitive volumes), so the test can skip.
fn try_note(nb: &TempNotebook, dir: &str) -> bool {
    match std::fs::create_dir(nb.path(dir)) {
        Ok(()) => {
            nb.file(&format!("{dir}/index.md"), "Body.\n");
            true
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => false,
        Err(e) => panic!("create {dir}: {e}"),
    }
}

// PRD-001 AC11
#[test]
fn hangul_slug_loads_without_diagnostics() {
    let nb = TempNotebook::new();
    nb.note("법령", "법령 노트.\n")
        .note("법령/010-개인정보", "PIPA.\n");
    nb.file("index.md", "See [[법령/개인정보]].\n");
    let (notebook, diags) = nb.load();
    assert!(diags.is_empty(), "{:?}", show(&diags));
    assert_eq!(child_slugs(&notebook, notebook.root()), ["법령"]);
    let pipa = notebook
        .by_path("법령/개인정보")
        .expect("Hangul path lookup");
    assert_eq!(notebook.get(pipa).slug, "개인정보");
    assert_eq!(notebook.get(pipa).order, Some(10));
    let root = notebook.get(notebook.root());
    assert_eq!(root.links[0].resolved, Some(pipa));
}

// PRD-001 AC11
#[test]
fn nfd_folder_matches_nfc_link_and_lookup() {
    // (case, NFC spelling)
    let cases: &[(&str, &str)] = &[("Hangul", "노트"), ("Latin accent", "café")];

    for (case, nfc) in cases {
        let decomposed = nfd(nfc);
        assert_ne!(*nfc, decomposed, "{case}: test data must differ in NFD");
        let nb = TempNotebook::new();
        nb.note(&decomposed, "Stored in NFD.\n");
        nb.note("other", &format!("Link: [[{nfc}]] and [[../{nfc}]].\n"));
        let (notebook, diags) = nb.load();
        assert!(diags.is_empty(), "{case}: {:?}", show(&diags));

        let id = notebook
            .by_path(nfc)
            .unwrap_or_else(|| panic!("{case}: NFC by_path should find the NFD folder"));
        assert_eq!(
            notebook.by_path(&decomposed),
            Some(id),
            "{case}: NFD lookup"
        );
        assert_eq!(
            notebook.get(id).slug,
            decomposed,
            "{case}: slug kept as written"
        );

        let other = notebook.get(notebook.by_path("other").expect("other"));
        for link in &other.links {
            assert_eq!(
                link.resolved,
                Some(id),
                "{case}: link {} resolves",
                link.target
            );
        }
        assert_eq!(other.links.len(), 2, "{case}");
    }
}

// PRD-001 AC11
#[test]
fn siblings_equal_under_nfc_and_case_are_duplicates() {
    // (case, first folder, second folder)
    let cases: &[(&str, String, String)] = &[
        ("NFC vs NFD Hangul", "노트".to_owned(), nfd("노트")),
        ("NFC vs NFD Latin", "café".to_owned(), nfd("café")),
        ("case only", "Caches".to_owned(), "caches".to_owned()),
        (
            "case with prefix",
            "010-Caches".to_owned(),
            "caches".to_owned(),
        ),
        ("case and NFD", "Café".to_owned(), nfd("café")),
    ];

    for (case, a, b) in cases {
        let nb = TempNotebook::new();
        assert!(try_note(&nb, a), "{case}: create {a}");
        if !try_note(&nb, b) {
            eprintln!("skipping {case}: file system folds {a:?} and {b:?}");
            continue;
        }
        let (_, diags) = nb.load();
        assert!(
            has_diag(&diags, Level::Error, &[a, b]),
            "{case}: expected a duplicate error naming both, got {:?}",
            show(&diags)
        );
        assert_eq!(errors(&diags).len(), 1, "{case}: {:?}", show(&diags));
    }
}

// PRD-001 AC11
#[test]
fn link_and_lookup_differing_only_in_case_resolve() {
    let nb = TempNotebook::new();
    nb.note("CPU", "x\n")
        .note("CPU/010-Caches", "x\n")
        .note("notes", "[[cpu/caches]] [[CPU/CACHES|c]] [[../Cpu]]\n");
    let (notebook, diags) = nb.load();
    assert!(diags.is_empty(), "{:?}", show(&diags));

    let caches = notebook.by_path("cpu/caches").expect("lowercase lookup");
    assert_eq!(notebook.by_path("CPU/Caches"), Some(caches));
    assert_eq!(notebook.by_path("Cpu/CACHES"), Some(caches));
    assert_eq!(notebook.get(caches).slug, "Caches", "slug kept as written");
    assert_eq!(notebook.get(caches).path.as_str(), "CPU/Caches");

    let cpu = notebook.by_path("cpu").expect("cpu");
    let notes = notebook.get(notebook.by_path("NOTES").expect("notes"));
    let resolved: Vec<_> = notes.links.iter().map(|l| l.resolved).collect();
    assert_eq!(resolved, [Some(caches), Some(caches), Some(cpu)]);
}
