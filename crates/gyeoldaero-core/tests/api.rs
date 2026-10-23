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
    assert!(config.build.out.is_absolute(), "{:?}", config.build.out);
    assert!(
        config.build.out.ends_with(format!("gyeoldaero/{folder}")),
        "{:?}",
        config.build.out
    );
    assert!(!config.build.out.starts_with(nb.root()));
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
