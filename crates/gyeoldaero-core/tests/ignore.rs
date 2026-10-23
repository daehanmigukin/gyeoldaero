//! PRD-001 AC4: ignore rules from architecture.md → Scanning rules.
//!
//! "Ignored entirely" is checked three ways: an ignored directory never becomes a note,
//! an ignored `.md` file produces no stray-file warning, and an ignored file with an
//! asset extension is not an asset. Each case runs at the root and inside a note.

mod common;

use std::path::Path;

use common::{TempNotebook, all_paths, show};
use gyeoldaero_core::Config;

/// Entries that must be ignored. Directories get an `index.md` inside so they would
/// otherwise be notes.
const IGNORED_DIRS: &[(&str, &str)] = &[
    ("hidden directory", ".hidden"),
    (".git", ".git"),
    ("Syncthing folder marker", ".stfolder"),
    ("Syncthing versions", ".stversions"),
    ("Syncthing temp (dot form)", ".syncthing.draft.tmp"),
    ("Syncthing temp (tilde form)", "~syncthing~draft.tmp"),
    ("vim swap", "draft.swp"),
    ("vim swap (second)", "draft.swo"),
    ("backup tilde", "draft~"),
    ("vim write probe", "4913"),
];

/// Files that must be ignored: Markdown-looking ones (would be stray `.md` warnings)
/// and asset-looking ones (would be assets).
const IGNORED_FILES: &[(&str, &str)] = &[
    ("hidden markdown", ".notes.md"),
    ("hidden asset", ".cover.png"),
    (
        "Syncthing temp markdown (dot form)",
        ".syncthing.notes.md.tmp",
    ),
    (
        "Syncthing temp markdown (tilde form)",
        "~syncthing~notes.md",
    ),
    ("Syncthing temp asset (tilde form)", "~syncthing~photo.png"),
    ("vim swap", ".index.md.swp"),
    ("vim swap visible", "index.md.swp"),
    ("vim swap (second)", "index.md.swo"),
    ("backup tilde markdown", "notes.md~"),
    ("backup tilde asset", "photo.png~"),
    ("vim write probe", "4913"),
];

fn asset_names(assets: &[std::path::PathBuf]) -> Vec<String> {
    assets
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .collect()
}

// PRD-001 AC4
#[test]
fn ignored_directories_are_not_notes() {
    for (case, name) in IGNORED_DIRS {
        for parent in ["", "note/"] {
            let nb = TempNotebook::new();
            nb.note("note", "Note.\n");
            nb.note(&format!("{parent}{name}"), "Should be ignored.\n");
            nb.note(&format!("{parent}{name}/inner"), "Also ignored.\n");
            let (notebook, diags) = nb.load();
            assert_eq!(
                all_paths(&notebook),
                ["note"],
                "{case} ({name}) under {parent:?}: became a note"
            );
            assert!(
                diags.is_empty(),
                "{case} ({name}) under {parent:?}: unexpected diagnostics {:?}",
                show(&diags)
            );
        }
    }
}

// PRD-001 AC4
#[test]
fn ignored_files_are_neither_stray_nor_assets() {
    for (case, name) in IGNORED_FILES {
        for parent in ["", "note/"] {
            let nb = TempNotebook::new();
            nb.note("note", "Note.\n");
            nb.file(&format!("{parent}{name}"), "ignored\n");
            let (notebook, diags) = nb.load();
            assert!(
                diags.is_empty(),
                "{case} ({name}) under {parent:?}: unexpected diagnostics {:?}",
                show(&diags)
            );
            let owner = if parent.is_empty() {
                notebook.root()
            } else {
                notebook.by_path("note").expect("note exists")
            };
            assert!(
                asset_names(&notebook.get(owner).assets).is_empty(),
                "{case} ({name}) under {parent:?}: listed as an asset"
            );
        }
    }
}

// PRD-001 AC4
#[test]
fn hidden_directory_without_index_md_gives_no_warning() {
    let nb = TempNotebook::new();
    nb.dir(".obsidian").file(".obsidian/app.json", "{}\n");
    nb.dir(".stversions/old");
    let (notebook, diags) = nb.load();
    assert!(all_paths(&notebook).is_empty());
    assert!(diags.is_empty(), "{:?}", show(&diags));
}

fn config_with_out(root: &Path, out: &Path) -> Config {
    let mut config = Config::load(root).expect("default config loads");
    config.build.out = out.to_path_buf();
    config
}

// PRD-001 AC4
#[test]
fn output_dir_inside_notebook_is_ignored() {
    // (case, output dir relative to the root)
    let cases: &[(&str, &str)] = &[("at the root", "_site"), ("nested in a note", "note/_site")];

    for (case, out_rel) in cases {
        let nb = TempNotebook::new();
        nb.note("note", "Note.\n");
        // A previous build's output looks like a notebook of its own.
        nb.note(out_rel, "<html>\n");
        nb.note(&format!("{out_rel}/page"), "<html>\n");
        nb.file(&format!("{out_rel}/style.md"), "not a note\n");

        // Control: without the config pointing at it, `_site` is an ordinary note.
        let (plain, _) = nb.load();
        assert!(
            all_paths(&plain).iter().any(|p| p.ends_with("_site")),
            "{case}: control load should see _site as a note"
        );

        let config = config_with_out(nb.root(), &nb.path(out_rel));
        let (notebook, diags) = nb.load_with(&config);
        assert_eq!(all_paths(&notebook), ["note"], "{case}: output dir scanned");
        assert!(diags.is_empty(), "{case}: {:?}", show(&diags));
    }
}

// PRD-001 AC4
#[test]
fn output_dir_from_gyeoldaero_toml_is_ignored() {
    let nb = TempNotebook::new();
    nb.note("note", "Note.\n");
    nb.note("_site", "<html>\n");
    let out = nb.path("_site");
    let out_str = out.to_str().expect("temp path is UTF-8");
    nb.file(
        "gyeoldaero.toml",
        &format!("[build]\nout = \"{}\"\n", out_str.replace('\\', "\\\\")),
    );
    let config = Config::load(nb.root()).expect("config loads");
    assert_eq!(config.build.out, out);
    let (notebook, diags) = nb.load_with(&config);
    assert_eq!(all_paths(&notebook), ["note"]);
    assert!(diags.is_empty(), "{:?}", show(&diags));
}
