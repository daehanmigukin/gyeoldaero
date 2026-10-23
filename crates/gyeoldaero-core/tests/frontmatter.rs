//! PRD-001 AC7: frontmatter (`title`, unknown fields preserved, malformed YAML).

mod common;

use common::{TempNotebook, body_text, errors, has_diag, show};
use gyeoldaero_core::Level;
use serde_yaml_ng::Value;

fn load_note(contents: &str) -> (gyeoldaero_core::Note, Vec<gyeoldaero_core::Diagnostic>) {
    let nb = TempNotebook::new();
    nb.file("topic/index.md", contents);
    let (notebook, diags) = nb.load();
    let note = notebook
        .get(notebook.by_path("topic").expect("topic"))
        .clone();
    (note, diags)
}

// PRD-001 AC7
#[test]
fn well_formed_frontmatter_is_split_from_the_body() {
    // (case, file contents, expected title, expected body without trailing newline)
    let cases: &[(&str, &str, Option<&str>, &str)] = &[
        (
            "title used",
            "---\ntitle: Caches, briefly\n---\nBody text.\n",
            Some("Caches, briefly"),
            "Body text.",
        ),
        (
            "quoted title",
            "---\ntitle: \"캐시: 개요\"\n---\nBody.\n",
            Some("캐시: 개요"),
            "Body.",
        ),
        (
            "no title field",
            "---\ntags: [a]\n---\nBody.\n",
            None,
            "Body.",
        ),
        ("empty block", "---\n---\nBody.\n", None, "Body."),
        ("no frontmatter", "Just a body.\n", None, "Just a body."),
        (
            "dashes later are not frontmatter",
            "Intro\n\n---\ntitle: nope\n---\n",
            None,
            "Intro\n\n---\ntitle: nope\n---",
        ),
    ];

    for (case, contents, title, body) in cases {
        let (note, diags) = load_note(contents);
        assert!(diags.is_empty(), "{case}: {:?}", show(&diags));
        assert_eq!(note.title.as_deref(), *title, "{case}: title");
        assert_eq!(body_text(&note.body), *body, "{case}: body");
    }
}

// PRD-001 AC7
#[test]
fn unknown_frontmatter_fields_are_preserved() {
    let (note, diags) = load_note(
        "---\ntitle: T\ntags: [rust, cache]\nreviewed: 2026-10-01\nextra:\n  nested: 3\n---\nBody.\n",
    );
    assert!(diags.is_empty(), "{:?}", show(&diags));
    assert_eq!(note.title.as_deref(), Some("T"));
    let fm = &note.frontmatter;
    assert_eq!(
        fm.get("tags"),
        Some(&Value::Sequence(vec![
            Value::String("rust".into()),
            Value::String("cache".into())
        ]))
    );
    assert_eq!(
        fm.get("reviewed"),
        Some(&Value::String("2026-10-01".into()))
    );
    let extra = fm
        .get("extra")
        .and_then(Value::as_mapping)
        .expect("extra mapping");
    assert_eq!(extra.get("nested").and_then(Value::as_u64), Some(3));
}

// PRD-001 AC7
#[test]
fn no_frontmatter_means_empty_mapping() {
    let (note, _) = load_note("Body.\n");
    assert!(note.frontmatter.is_empty());
}

// PRD-001 AC7
#[test]
fn malformed_frontmatter_is_an_error_and_the_whole_file_is_the_body() {
    let cases: &[(&str, &str)] = &[
        (
            "unclosed flow sequence",
            "---\ntitle: [unclosed\n---\nBody.\n",
        ),
        (
            "bad indentation",
            "---\ntitle: a\n  bad: indent\n---\nBody.\n",
        ),
        ("tab indentation", "---\nkey:\n\t- x\n---\nBody.\n"),
    ];

    for (case, contents) in cases {
        let (note, diags) = load_note(contents);
        assert!(
            has_diag(&diags, Level::Error, &["topic"]),
            "{case}: expected an error for topic, got {:?}",
            show(&diags)
        );
        assert_eq!(errors(&diags).len(), 1, "{case}: {:?}", show(&diags));
        assert_eq!(
            note.body, *contents,
            "{case}: whole file should be the body"
        );
        assert_eq!(note.title, None, "{case}");
        assert!(note.frontmatter.is_empty(), "{case}");
    }
}

// PRD-001 AC7
#[test]
fn yaml_error_positions_are_lines_of_index_md() {
    // Both errors are on line 3 of the file (line 2 of the YAML block).
    let cases: &[(&str, &str)] = &[
        (
            "unclosed flow sequence",
            "---\ntitle: [unclosed\n---\nBody.\n",
        ),
        (
            "bad indentation",
            "---\ntitle: a\n  bad: indent\n---\nBody.\n",
        ),
    ];
    for (case, contents) in cases {
        let (_, diags) = load_note(contents);
        let message = &errors(&diags)[0].message;
        assert!(message.contains("line 3"), "{case}: {message}");
        assert!(!message.contains("line 2"), "{case}: {message}");
        assert!(!message.contains("line 1"), "{case}: {message}");
    }
}

// PRD-001 AC7
#[test]
fn opening_dashes_without_closing_are_a_warning_and_the_whole_file_is_the_body() {
    let cases: &[(&str, &str)] = &[
        ("looks like frontmatter", "---\ntitle: T\nBody.\n"),
        (
            "horizontal rule at the top",
            "---\n\nA note that starts with a rule.\n",
        ),
        ("only the dashes", "---\n"),
    ];

    for (case, contents) in cases {
        let (note, diags) = load_note(contents);
        assert!(errors(&diags).is_empty(), "{case}: {:?}", show(&diags));
        assert!(
            has_diag(&diags, Level::Warning, &["topic"]),
            "{case}: expected a warning for topic, got {:?}",
            show(&diags)
        );
        assert_eq!(diags.len(), 1, "{case}: {:?}", show(&diags));
        assert_eq!(note.body, *contents, "{case}: whole file is the body");
        assert_eq!(note.title, None, "{case}");
        assert!(note.frontmatter.is_empty(), "{case}");
    }
}

// PRD-001 AC7
#[test]
fn crlf_line_endings_are_accepted() {
    let nb = TempNotebook::new();
    nb.note("target", "Target.\r\n");
    nb.file(
        "topic/index.md",
        "---\r\ntitle: Windows note\r\ntags: [a]\r\n---\r\nFirst line.\r\nSee [[target]].\r\n",
    );
    let (notebook, diags) = nb.load();
    assert!(diags.is_empty(), "{:?}", show(&diags));
    let note = notebook.get(notebook.by_path("topic").expect("topic"));
    assert_eq!(note.title.as_deref(), Some("Windows note"));
    assert!(note.frontmatter.get("tags").is_some());
    assert_eq!(
        body_text(&note.body).replace("\r\n", "\n"),
        "First line.\nSee [[target]]."
    );
    assert_eq!(note.links.len(), 1);
    assert_eq!(note.links[0].target, "target");
    assert_eq!(note.links[0].resolved, notebook.by_path("target"));
}

// PRD-001 AC7
#[test]
fn crlf_without_frontmatter_and_unclosed_block() {
    let (note, diags) = load_note("Just a body.\r\n");
    assert!(diags.is_empty(), "{:?}", show(&diags));
    assert_eq!(body_text(&note.body).trim_end_matches('\r'), "Just a body.");

    let (note, diags) = load_note("---\r\ntitle: T\r\nBody.\r\n");
    assert!(errors(&diags).is_empty(), "{:?}", show(&diags));
    assert_eq!(diags.len(), 1, "unclosed block warns: {:?}", show(&diags));
    assert_eq!(note.title, None);
    assert!(note.body.contains("title: T"));
}

// PRD-001 AC7
#[test]
fn numeric_and_boolean_titles_become_strings() {
    // (case, title value as written, expected title)
    let cases: &[(&str, &str, &str)] = &[
        ("integer", "2026", "2026"),
        ("float", "3.5", "3.5"),
        ("boolean", "true", "true"),
        ("negative", "-1", "-1"),
    ];

    for (case, value, expected) in cases {
        let (note, diags) = load_note(&format!("---\ntitle: {value}\n---\nBody.\n"));
        assert!(diags.is_empty(), "{case}: {:?}", show(&diags));
        assert_eq!(note.title.as_deref(), Some(*expected), "{case}");
        assert_eq!(body_text(&note.body), "Body.", "{case}");
    }
}

// PRD-001 AC7
#[test]
fn list_or_map_title_is_an_error() {
    let cases: &[(&str, &str)] = &[
        ("flow list", "---\ntitle: [a, b]\ntags: [x]\n---\nBody.\n"),
        (
            "block map",
            "---\ntitle:\n  en: A\n  ko: 가\ntags: [x]\n---\nBody.\n",
        ),
    ];

    for (case, contents) in cases {
        let (note, diags) = load_note(contents);
        assert!(
            has_diag(&diags, Level::Error, &["topic", "title"]),
            "{case}: expected an error about the title, got {:?}",
            show(&diags)
        );
        assert_eq!(errors(&diags).len(), 1, "{case}: {:?}", show(&diags));
        assert_eq!(note.title, None, "{case}");
        // The frontmatter itself is valid YAML: it is still split from the body.
        assert_eq!(body_text(&note.body), "Body.", "{case}");
        assert!(note.frontmatter.get("tags").is_some(), "{case}");
    }
}
