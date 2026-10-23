//! PRD-001 AC8: wiki links — absolute, `./`, `../`, labels, broken links, links in code.

mod common;

use common::{TempNotebook, errors, has_diag, show};
use gyeoldaero_core::{Level, Notebook};

/// Notebook used by the resolution table:
///
/// ```text
/// /            (root)
/// a/           a/child/  a/child/gc/  a/sibling/
/// 010-b/       010-b/c/
/// ```
fn notebook_with_source(source: &str, body: &str) -> (Notebook, Vec<gyeoldaero_core::Diagnostic>) {
    let nb = TempNotebook::new();
    nb.note("a", "A.\n")
        .note("a/child", "Child.\n")
        .note("a/child/gc", "Grandchild.\n")
        .note("a/sibling", "Sibling.\n")
        .note("010-b", "B.\n")
        .note("010-b/c", "C.\n");
    let index = if source.is_empty() {
        "index.md".to_owned()
    } else {
        format!("{source}/index.md")
    };
    nb.file(&index, body);
    nb.load()
}

fn source_id(nb: &Notebook, source: &str) -> gyeoldaero_core::NoteId {
    if source.is_empty() {
        nb.root()
    } else {
        // Disk paths may carry the `010-` prefix; lookups use slugs.
        nb.by_path(&source.replace("010-b", "b"))
            .expect("source note exists")
    }
}

/// (case, source note dir on disk ("" = root), link text inside `[[ ]]`, expected
/// target, expected label, expected resolved slug path or `None` if broken)
type LinkCase = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
);

// PRD-001 AC8
#[test]
fn links_resolve_absolute_and_relative() {
    let cases: &[LinkCase] = &[
        ("absolute", "a/child", "a", "a", None, Some("a")),
        (
            "absolute nested",
            "a/child",
            "a/sibling",
            "a/sibling",
            None,
            Some("a/sibling"),
        ),
        (
            "absolute ignores prefixes",
            "a/child",
            "b/c",
            "b/c",
            None,
            Some("b/c"),
        ),
        (
            "absolute from root",
            "",
            "a/child/gc",
            "a/child/gc",
            None,
            Some("a/child/gc"),
        ),
        (
            "./ child",
            "a/child",
            "./gc",
            "./gc",
            None,
            Some("a/child/gc"),
        ),
        ("./ from root", "", "./a", "./a", None, Some("a")),
        (
            "../ sibling",
            "a/child",
            "../sibling",
            "../sibling",
            None,
            Some("a/sibling"),
        ),
        (
            "../../ cousin",
            "a/child",
            "../../b/c",
            "../../b/c",
            None,
            Some("b/c"),
        ),
        (
            "../ from a prefixed note",
            "010-b/c",
            "../../a",
            "../../a",
            None,
            Some("a"),
        ),
        (
            "label",
            "a/child",
            "a/sibling|the sibling",
            "a/sibling",
            Some("the sibling"),
            Some("a/sibling"),
        ),
        (
            "label on relative",
            "a/child",
            "./gc|grandkid",
            "./gc",
            Some("grandkid"),
            Some("a/child/gc"),
        ),
        ("broken absolute", "a/child", "nope", "nope", None, None),
        ("broken nested", "a/child", "a/nope", "a/nope", None, None),
        ("broken relative", "a/child", "./nope", "./nope", None, None),
        ("../ above the root", "", "../a", "../a", None, None),
    ];

    for (case, source, inner, target, label, resolved) in cases {
        let body = format!("See [[{inner}]] here.\n");
        let (nb, diags) = notebook_with_source(source, &body);
        let note = nb.get(source_id(&nb, source));
        assert_eq!(note.links.len(), 1, "{case}: links {:?}", note.links);
        let link = &note.links[0];
        assert_eq!(link.target, *target, "{case}: target");
        assert_eq!(link.label.as_deref(), *label, "{case}: label");
        let got = link.resolved.map(|id| nb.get(id).path.as_str().to_owned());
        assert_eq!(got.as_deref(), *resolved, "{case}: resolved");
        if resolved.is_some() {
            assert!(diags.is_empty(), "{case}: {:?}", show(&diags));
        } else {
            assert!(
                has_diag(&diags, Level::Error, &[target]),
                "{case}: expected an error naming {target}, got {:?}",
                show(&diags)
            );
            assert_eq!(errors(&diags).len(), 1, "{case}: {:?}", show(&diags));
        }
    }
}

// PRD-001 AC8
#[test]
fn links_are_kept_in_body_order() {
    let (nb, diags) = notebook_with_source(
        "a",
        "First [[a/sibling]], then [[./child|kid]].\n\n- and [[b]]\n- and [[missing]]\n",
    );
    let note = nb.get(nb.by_path("a").expect("a"));
    let targets: Vec<&str> = note.links.iter().map(|l| l.target.as_str()).collect();
    assert_eq!(targets, ["a/sibling", "./child", "b", "missing"]);
    assert_eq!(errors(&diags).len(), 1, "{:?}", show(&diags));
}

// PRD-001 AC8
#[test]
fn links_in_code_are_ignored() {
    // Every body here has only links inside code; all point nowhere so a wrongly
    // extracted link would also show up as a broken-link error.
    let cases: &[(&str, &str)] = &[
        ("inline code", "Use `[[nowhere]]` syntax.\n"),
        ("double-backtick code", "Use ``[[nowhere]]`` too.\n"),
        ("fenced backticks", "Text.\n\n```\n[[nowhere]]\n```\n"),
        (
            "fenced with info string",
            "Text.\n\n```markdown\nSee [[nowhere|x]]\n```\n",
        ),
        ("fenced tildes", "Text.\n\n~~~\n[[nowhere]]\n~~~\n"),
        ("indented code block", "Text.\n\n    [[nowhere]]\n"),
        (
            "fence inside a list",
            "- item\n\n  ```\n  [[nowhere]]\n  ```\n",
        ),
    ];

    for (case, body) in cases {
        let (nb, diags) = notebook_with_source("a", body);
        let note = nb.get(nb.by_path("a").expect("a"));
        assert!(note.links.is_empty(), "{case}: extracted {:?}", note.links);
        assert!(diags.is_empty(), "{case}: {:?}", show(&diags));
    }
}

// PRD-001 AC8
#[test]
fn links_next_to_code_are_still_found() {
    let (nb, diags) = notebook_with_source(
        "a",
        "`[[nowhere]]` but [[a/sibling]]\n\n```\n[[nowhere]]\n```\n\nAfter: [[b/c]]\n",
    );
    let note = nb.get(nb.by_path("a").expect("a"));
    let targets: Vec<&str> = note.links.iter().map(|l| l.target.as_str()).collect();
    assert_eq!(targets, ["a/sibling", "b/c"]);
    assert!(diags.is_empty(), "{:?}", show(&diags));
}

// PRD-001 AC8
#[test]
fn prefixed_links_resolve_with_a_warning_to_link_by_slug() {
    // (case, source note dir on disk, link text, expected resolved slug path)
    let cases: &[(&str, &str, &str, &str)] = &[
        ("absolute, prefixed note", "a/child", "010-b", "b"),
        ("absolute, prefixed parent", "a/child", "010-b/c", "b/c"),
        ("relative, prefixed", "a/child", "../../010-b/c", "b/c"),
        ("with label", "a/child", "010-b|bee", "b"),
    ];

    for (case, source, inner, resolved) in cases {
        let body = format!("See [[{inner}]] here.\n");
        let (nb, diags) = notebook_with_source(source, &body);
        let note = nb.get(source_id(&nb, source));
        assert_eq!(note.links.len(), 1, "{case}: links {:?}", note.links);
        let link = &note.links[0];
        let got = link.resolved.map(|id| nb.get(id).path.as_str().to_owned());
        assert_eq!(got.as_deref(), Some(*resolved), "{case}: resolved");
        let target = inner.split('|').next().unwrap_or(inner);
        assert_eq!(link.target, target, "{case}: target kept as written");
        assert!(errors(&diags).is_empty(), "{case}: {:?}", show(&diags));
        assert!(
            has_diag(&diags, Level::Warning, &[target, resolved]),
            "{case}: expected a warning naming {target} and the slug path {resolved}, got {:?}",
            show(&diags)
        );
        assert_eq!(diags.len(), 1, "{case}: {:?}", show(&diags));
    }
}
