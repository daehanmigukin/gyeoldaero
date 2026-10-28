//! PRD-001 AC3: duplicate slugs among siblings.

mod common;

use common::{TempNotebook, errors, has_diag, show};
use gyeoldaero_core::Level;

// PRD-001 AC3
#[test]
fn duplicate_sibling_slugs_are_an_error_naming_both_folders() {
    // (case, parent folder ("" = root), the two clashing folder names)
    let cases: &[(&str, &str, [&str; 2])] = &[
        ("prefixed and unprefixed", "", ["010-topic", "topic"]),
        ("two different prefixes", "", ["01-topic", "002-topic"]),
        ("nested under a note", "parent", ["005-child", "07-child"]),
    ];

    for (case, parent, [a, b]) in cases {
        let nb = TempNotebook::new();
        let prefix = if parent.is_empty() {
            String::new()
        } else {
            nb.note(parent, "Parent.\n");
            format!("{parent}/")
        };
        nb.note(&format!("{prefix}{a}"), "A.\n");
        nb.note(&format!("{prefix}{b}"), "B.\n");
        let (_, diags) = nb.load();
        assert!(
            has_diag(&diags, Level::Error, &[a, b]),
            "{case}: expected one error naming {a} and {b}, got {:?}",
            show(&diags)
        );
        assert_eq!(
            errors(&diags).len(),
            1,
            "{case}: one error per clash, got {:?}",
            show(&diags)
        );
    }
}

// PRD-001 AC3
#[test]
fn same_slug_in_different_parents_is_fine() {
    let nb = TempNotebook::new();
    nb.note("a", "A.\n")
        .note("a/topic", "x\n")
        .note("b", "B.\n")
        .note("b/010-topic", "x\n");
    let (_, diags) = nb.load();
    assert!(diags.is_empty(), "{:?}", show(&diags));
}
