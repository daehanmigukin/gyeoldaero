//! PRD-001 Requirement 2: `Config::load` defaults and hard errors from `Notebook::load`.
//! No dedicated AC; these pin the API the other tests rely on.

mod common;

use common::TempNotebook;
use gyeoldaero_core::{Config, Notebook};

// PRD-001 Req2
#[test]
fn config_defaults_when_file_is_absent() {
    let nb = TempNotebook::new();
    let config = Config::load(nb.root()).expect("defaults");
    let folder = nb
        .root()
        .file_name()
        .expect("temp dir has a name")
        .to_string_lossy()
        .into_owned();
    assert_eq!(config.name, folder);
    assert_eq!(config.serve.bind.to_string(), "127.0.0.1:4321");
    let out = &config.build.out;
    assert!(out.is_absolute(), "{out:?}");
    // <cache dir>/gyeoldaero/<folder-name>-<8 hex chars>
    let leaf = out
        .file_name()
        .expect("out has a name")
        .to_string_lossy()
        .into_owned();
    let hash = leaf
        .strip_prefix(&format!("{folder}-"))
        .unwrap_or_else(|| panic!("{leaf:?} starts with the folder name"));
    assert_eq!(hash.len(), 8, "{leaf:?}");
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "{leaf:?}"
    );
    assert!(
        out.parent().is_some_and(|p| p.ends_with("gyeoldaero")),
        "{out:?}"
    );
    if let Some(cache) = dirs::cache_dir() {
        assert!(out.starts_with(cache), "{out:?}");
    }
    assert!(!out.starts_with(nb.root()));
}

// PRD-001 Req2
#[test]
fn default_out_is_stable_and_differs_between_same_named_notebooks() {
    let a = tempfile::tempdir().expect("temp dir");
    let b = tempfile::tempdir().expect("temp dir");
    let (na, nb) = (a.path().join("notes"), b.path().join("notes"));
    std::fs::create_dir(&na).expect("mkdir");
    std::fs::create_dir(&nb).expect("mkdir");
    let out_a = Config::load(&na).expect("config").build.out;
    assert_eq!(
        Config::load(&na).expect("config").build.out,
        out_a,
        "stable"
    );
    let out_b = Config::load(&nb).expect("config").build.out;
    assert_ne!(out_a, out_b, "same folder name, different notebooks");
    // The display name never enters the path.
    std::fs::write(na.join("gyeoldaero.toml"), "name = \"../../escape\"\n").expect("write");
    assert_eq!(Config::load(&na).expect("config").build.out, out_a);
}

// PRD-001 Req2
#[test]
fn config_reads_gyeoldaero_toml() {
    let nb = TempNotebook::new();
    nb.file(
        "gyeoldaero.toml",
        "name = \"my notes\"\n[serve]\nbind = \"100.64.0.1:8080\"\n[build]\nout = \"~/sites/notes\"\n",
    );
    let config = Config::load(nb.root()).expect("config");
    assert_eq!(config.name, "my notes");
    assert_eq!(config.serve.bind.to_string(), "100.64.0.1:8080");
    assert!(!config.build.out.starts_with("~"), "~ should be expanded");
    assert!(config.build.out.ends_with("sites/notes"));
}

// PRD-001 Req2
#[test]
fn malformed_config_is_an_error() {
    let nb = TempNotebook::new();
    nb.file("gyeoldaero.toml", "name = [unclosed\n");
    assert!(Config::load(nb.root()).is_err());
}

// PRD-001 Req2
#[test]
fn unusable_root_is_a_hard_error() {
    let nb = TempNotebook::new();
    let config = Config::load(nb.root()).expect("defaults");
    let missing = nb.path("does-not-exist");
    assert!(Notebook::load(&missing, &config).is_err(), "missing root");
    let file = nb.path("index.md");
    assert!(Notebook::load(&file, &config).is_err(), "root is a file");
}

// PRD-001 AC12
#[test]
fn relative_build_out_resolves_against_the_notebook_root() {
    // (case, `out` as written, expected path relative to the root)
    let cases: &[(&str, &str, &str)] = &[
        ("bare name", "_site", "_site"),
        ("dot slash", "./build/site", "build/site"),
    ];

    for (case, out, rel) in cases {
        let nb = TempNotebook::new();
        nb.file("gyeoldaero.toml", &format!("[build]\nout = \"{out}\"\n"));
        let config = Config::load(nb.root()).expect("config");
        let root = std::path::absolute(nb.root()).expect("absolute root");
        assert!(
            config.build.out.is_absolute(),
            "{case}: {:?}",
            config.build.out
        );
        assert_eq!(
            config.build.out.components().collect::<Vec<_>>(),
            root.join(rel).components().collect::<Vec<_>>(),
            "{case}"
        );
    }
}

// PRD-001 AC12
#[test]
fn relative_build_out_inside_the_notebook_is_ignored_when_scanning() {
    let nb = TempNotebook::new();
    nb.note("note", "Note.\n");
    nb.note("_site", "<html>\n");
    nb.file("gyeoldaero.toml", "[build]\nout = \"_site\"\n");
    let config = Config::load(nb.root()).expect("config");
    let (notebook, diags) = nb.load_with(&config);
    assert_eq!(common::all_paths(&notebook), ["note"]);
    assert!(diags.is_empty(), "{:?}", common::show(&diags));
}

// PRD-001 AC12
#[test]
fn build_out_pointing_at_the_notebook_or_home_is_an_error() {
    // (case, `out` as written)
    let cases: &[(&str, &str)] = &[
        ("the root itself", "."),
        ("the root, spelled with a slash", "./"),
        ("the parent", ".."),
        ("the file system root", "/"),
        ("home", "~"),
    ];
    for (case, out) in cases {
        let nb = TempNotebook::new();
        nb.file("gyeoldaero.toml", &format!("[build]\nout = \"{out}\"\n"));
        let err = Config::load(nb.root()).expect_err(case);
        assert!(
            matches!(err, gyeoldaero_core::CoreError::Config { .. }),
            "{case}: {err:?}"
        );
        assert!(err.to_string().contains("build.out"), "{case}: {err}");
    }
}

// PRD-001 AC12
#[test]
fn build_out_dot_dot_segments_resolve_lexically() {
    let nb = TempNotebook::new();
    nb.file("gyeoldaero.toml", "[build]\nout = \"sub/../site\"\n");
    let config = Config::load(nb.root()).expect("config");
    let root = std::path::absolute(nb.root()).expect("absolute root");
    assert_eq!(config.build.out, root.join("site"));
}

// PRD-001 AC7 (CRLF everywhere)
#[test]
fn config_with_crlf_line_endings_loads() {
    let nb = TempNotebook::new();
    nb.file(
        "gyeoldaero.toml",
        "name = \"crlf\"\r\n[serve]\r\nbind = \"0.0.0.0:1\"\r\n",
    );
    let config = Config::load(nb.root()).expect("config");
    assert_eq!(config.name, "crlf");
    assert_eq!(config.serve.bind.to_string(), "0.0.0.0:1");
}
