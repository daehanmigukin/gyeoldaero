//! PRD-001 AC13: robustness against unreadable entries, deep nesting, huge files,
//! dot slugs and non-regular `index.md` files.

mod common;

use common::{TempNotebook, all_paths, errors, has_diag, show};
use gyeoldaero_core::Level;

/// True when permission bits are enforced for this process (not root).
#[cfg(unix)]
fn permissions_enforced(nb: &TempNotebook) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let probe = nb.path("probe");
    std::fs::create_dir(&probe).expect("mkdir probe");
    std::fs::set_permissions(&probe, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    let enforced = std::fs::read_dir(&probe).is_err();
    std::fs::set_permissions(&probe, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    std::fs::remove_dir(&probe).expect("rmdir probe");
    enforced
}

#[cfg(unix)]
fn chmod(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
}

// PRD-001 AC13
#[cfg(unix)]
#[test]
fn unreadable_folder_is_an_error_and_the_scan_continues() {
    let nb = TempNotebook::new();
    if !permissions_enforced(&nb) {
        eprintln!("skipping: permissions not enforced (running as root?)");
        return;
    }
    nb.note("a", "A.\n")
        .note("locked", "L.\n")
        .note("locked/child", "C.\n")
        .note("z", "Z.\n");
    chmod(&nb.path("locked"), 0o000);
    let result = gyeoldaero_core::Notebook::load(
        nb.root(),
        &gyeoldaero_core::Config::load(nb.root()).expect("config"),
    );
    chmod(&nb.path("locked"), 0o755);

    let (notebook, diags) = result.expect("an unreadable subfolder is not a hard error");
    assert_eq!(all_paths(&notebook), ["a", "locked", "z"]);
    assert!(
        has_diag(&diags, Level::Error, &["locked"]),
        "{:?}",
        show(&diags)
    );
    assert_eq!(diags.len(), 1, "{:?}", show(&diags));
}

// PRD-001 AC13
#[cfg(unix)]
#[test]
fn unreadable_index_md_is_an_error_with_an_empty_body() {
    let nb = TempNotebook::new();
    if !permissions_enforced(&nb) {
        eprintln!("skipping: permissions not enforced (running as root?)");
        return;
    }
    nb.note("a", "Secret-ish.\n").note("b", "B.\n");
    chmod(&nb.path("a/index.md"), 0o000);
    let result = gyeoldaero_core::Notebook::load(
        nb.root(),
        &gyeoldaero_core::Config::load(nb.root()).expect("config"),
    );
    chmod(&nb.path("a/index.md"), 0o644);

    let (notebook, diags) = result.expect("an unreadable index.md is not a hard error");
    assert_eq!(all_paths(&notebook), ["a", "b"]);
    assert_eq!(notebook.get(notebook.by_path("a").expect("a")).body, "");
    assert!(
        has_diag(&diags, Level::Error, &["a", "index.md"]),
        "{:?}",
        show(&diags)
    );
    assert_eq!(diags.len(), 1, "{:?}", show(&diags));
}

// PRD-001 AC13
#[test]
fn nesting_deeper_than_64_levels_is_cut_off_without_overflowing_the_stack() {
    let nb = TempNotebook::new();
    // 1,500 levels overflowed an 8 MiB stack with unbounded recursion.
    let deep: String = std::iter::repeat_n("d", 1500).collect::<Vec<_>>().join("/");
    std::fs::create_dir_all(nb.path(&deep)).expect("create deep tree");
    let mut rel = String::new();
    for _ in 0..70 {
        rel = if rel.is_empty() {
            "d".to_owned()
        } else {
            format!("{rel}/d")
        };
        nb.file(&format!("{rel}/index.md"), "Level.\n");
    }

    let root = nb.root().to_path_buf();
    let handle = std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(move || {
            let config = gyeoldaero_core::Config::load(&root).expect("config");
            gyeoldaero_core::Notebook::load(&root, &config).expect("loads")
        })
        .expect("spawn");
    let (notebook, diags) = handle.join().expect("no stack overflow");

    assert_eq!(notebook.len(), 1 + 64, "root plus 64 levels");
    assert_eq!(errors(&diags).len(), 1, "{:?}", show(&diags));
    assert!(
        has_diag(&diags, Level::Error, &["64"]),
        "{:?}",
        show(&diags)
    );
}

// PRD-001 AC13
#[test]
fn index_md_over_8_mib_is_an_error_with_an_empty_body() {
    let nb = TempNotebook::new();
    let big = "a".repeat(8 * 1024 * 1024 + 1);
    nb.file("big/index.md", &big)
        .note("small", "Small [[nowhere]].\n");
    let (notebook, diags) = nb.load();
    let note = notebook.get(notebook.by_path("big").expect("big is still a note"));
    assert_eq!(note.body, "");
    assert!(note.links.is_empty());
    assert!(
        has_diag(&diags, Level::Error, &["big", "8 MiB"]),
        "{:?}",
        show(&diags)
    );
    // The scan went on: the broken link in `small` is still reported.
    assert_eq!(errors(&diags).len(), 2, "{:?}", show(&diags));
}

// PRD-001 AC13
#[test]
fn folders_whose_slug_would_start_with_a_dot_are_errors_and_skipped() {
    let names = ["001-..", "002-.", "003-.git", "004-.hidden"];
    let nb = TempNotebook::new();
    nb.note("ok", "Ok.\n");
    for name in names {
        nb.note(name, "Should not be a note.\n");
        nb.note(&format!("{name}/inner"), "Nor this.\n");
    }
    let (notebook, diags) = nb.load();
    assert_eq!(all_paths(&notebook), ["ok"]);
    for id in common::all_notes(&notebook) {
        for segment in notebook.get(id).path.as_str().split('/') {
            assert!(!segment.starts_with('.'), "{segment:?}");
        }
    }
    for name in names {
        assert!(
            has_diag(&diags, Level::Error, &[name]),
            "expected an error naming {name}: {:?}",
            show(&diags)
        );
    }
    assert_eq!(errors(&diags).len(), names.len(), "{:?}", show(&diags));
}

// PRD-001 AC13
#[cfg(unix)]
#[test]
fn fifo_named_index_md_is_not_read_and_does_not_block() {
    let nb = TempNotebook::new();
    nb.dir("pipe");
    let made = std::process::Command::new("mkfifo")
        .arg(nb.path("pipe/index.md"))
        .status();
    if !made.is_ok_and(|s| s.success()) {
        eprintln!("skipping: mkfifo unavailable");
        return;
    }
    let root = nb.root().to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let config = gyeoldaero_core::Config::load(&root).expect("config");
        let _ = tx.send(gyeoldaero_core::Notebook::load(&root, &config).map(|(_, d)| d));
    });
    let diags = rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("load must not block on a FIFO")
        .expect("loads");
    assert!(
        has_diag(&diags, Level::Warning, &["pipe", "index.md"]),
        "{:?}",
        show(&diags)
    );
    assert!(errors(&diags).is_empty(), "{:?}", show(&diags));
}

// PRD-001 AC14
#[test]
fn diagnostics_carry_the_folder_on_disk() {
    let nb = TempNotebook::new();
    nb.note("010-a", "First [[nowhere]].\n")
        .note("a", "Second [[nowhere]].\n")
        .note("020-b", "B.\n")
        .note("020-b/005-c", "[[missing]]\n");
    let (notebook, diags) = nb.load();
    let broken: Vec<String> = diags
        .iter()
        .filter(|d| d.message.contains("nowhere") || d.message.contains("missing"))
        .map(|d| d.dir.to_string_lossy().into_owned())
        .collect();
    assert_eq!(broken, ["010-a", "a", "020-b/005-c"], "{:?}", show(&diags));
    let c = notebook.get(notebook.by_path("b/c").expect("b/c"));
    assert_eq!(c.dir, std::path::Path::new("020-b/005-c"));
    assert_eq!(notebook.get(notebook.root()).dir, std::path::Path::new(""));
}
