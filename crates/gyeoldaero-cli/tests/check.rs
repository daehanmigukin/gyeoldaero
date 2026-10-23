//! PRD-001 AC9: `gyeoldaero check [PATH]` output and exit codes (Requirement 4).
//!
//! Message wording is not pinned. Pinned: the summary line format
//! (`3 errors, 1 warning in 21 notes`, singular for 1), grouping by note path, that each
//! problem's note or file name appears, and exit codes 0 / 1 / 2.

use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn all(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }
}

fn check(args: &[&std::ffi::OsStr], cwd: Option<&std::path::Path>) -> Run {
    let mut cmd = cargo_bin_cmd!("gyeoldaero");
    cmd.arg("check").args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().expect("run gyeoldaero");
    Run {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn check_path(path: &std::path::Path) -> Run {
    check(&[path.as_os_str()], None)
}

/// Parses `E error(s), W warning(s) in N note(s)` with correct singular/plural.
fn parse_summary(line: &str) -> Option<(u32, u32, u32)> {
    fn count(part: &str, noun: &str) -> Option<u32> {
        let (n, word) = part.trim().split_once(' ')?;
        let n: u32 = n.parse().ok()?;
        let want = if n == 1 {
            noun.to_owned()
        } else {
            format!("{noun}s")
        };
        (word == want).then_some(n)
    }
    let (errs, rest) = line.trim().split_once(", ")?;
    let (warns, notes) = rest.split_once(" in ")?;
    Some((
        count(errs, "error")?,
        count(warns, "warning")?,
        count(notes, "note")?,
    ))
}

/// The single summary line in the output.
fn summary(run: &Run) -> (u32, u32, u32) {
    let all = run.all();
    let found: Vec<_> = all.lines().filter_map(parse_summary).collect();
    assert_eq!(
        found.len(),
        1,
        "expected exactly one summary line, got output:\n{all}"
    );
    found[0]
}

// PRD-001 AC9
#[test]
fn summary_line_parser_accepts_the_documented_form() {
    assert_eq!(
        parse_summary("3 errors, 1 warning in 21 notes"),
        Some((3, 1, 21))
    );
    assert_eq!(
        parse_summary("1 error, 0 warnings in 1 note"),
        Some((1, 0, 1))
    );
    assert_eq!(parse_summary("1 errors, 0 warnings in 2 notes"), None);
    assert_eq!(parse_summary("error: something"), None);
}

// PRD-001 AC9
#[test]
fn clean_fixture_exits_zero_with_summary() {
    let run = check_path(&fixture("notebook"));
    assert_eq!(run.code, Some(0), "output:\n{}", run.all());
    assert_eq!(summary(&run), (0, 0, 23));
}

// PRD-001 AC9
#[test]
fn path_defaults_to_current_directory() {
    let run = check(&[], Some(&fixture("notebook")));
    assert_eq!(run.code, Some(0), "output:\n{}", run.all());
    assert_eq!(summary(&run), (0, 0, 23));
}

// PRD-001 AC9
#[test]
fn warnings_only_exits_zero() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("index.md"), "Root.\n").expect("write");
    std::fs::write(dir.path().join("draft.md"), "Stray.\n").expect("write");
    let run = check_path(dir.path());
    assert_eq!(run.code, Some(0), "output:\n{}", run.all());
    assert_eq!(summary(&run), (0, 1, 1));
    assert!(run.all().contains("draft.md"), "output:\n{}", run.all());
}

// PRD-001 AC9
#[test]
fn broken_fixture_exits_one_and_reports_every_problem() {
    let run = check_path(&fixture("broken"));
    let all = run.all();
    assert_eq!(run.code, Some(1), "output:\n{all}");
    let (errors, warnings, _notes) = summary(&run);
    assert_eq!((errors, warnings), (3, 3), "output:\n{all}");
    // Each problem names its note or file.
    for needle in [
        "010-dup",
        "bad-frontmatter",
        "links",
        "nowhere-at-all",
        "no-index",
        "draft.md",
        "other-notes.md",
    ] {
        assert!(all.contains(needle), "missing {needle:?} in output:\n{all}");
    }
}

// PRD-001 AC9
#[test]
fn broken_fixture_groups_diagnostics_by_note_path() {
    let run = check_path(&fixture("broken"));
    let all = run.all();
    let lines: Vec<&str> = all.lines().collect();
    let line_of = |needle: &str| {
        lines
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("missing {needle:?} in output:\n{all}"))
    };
    // The two stray files belong to the same note (`stray`): no other note's problem
    // may be printed between them.
    let (a, b) = {
        let (x, y) = (line_of("draft.md"), line_of("other-notes.md"));
        (x.min(y), x.max(y))
    };
    for other in ["010-dup", "bad-frontmatter", "nowhere-at-all", "no-index"] {
        let at = line_of(other);
        assert!(
            !(a < at && at < b),
            "{other:?} printed inside the `stray` group:\n{all}"
        );
    }
    // The group is headed by the note path `stray` on the line just above it.
    assert!(a > 0, "no header above the `stray` group:\n{all}");
    assert_eq!(
        lines[a - 1].trim(),
        "stray",
        "header of the `stray` group:\n{all}"
    );
}

// PRD-001 AC9
#[test]
fn unusable_root_exits_two() {
    let dir = tempfile::tempdir().expect("temp dir");
    let missing = dir.path().join("no-such-notebook");
    let file = dir.path().join("a-file.md");
    std::fs::write(&file, "x\n").expect("write");

    for (case, path, name) in [
        ("nonexistent path", &missing, "no-such-notebook"),
        ("path is a file", &file, "a-file.md"),
    ] {
        let run = check_path(path);
        assert_eq!(run.code, Some(2), "{case}: output:\n{}", run.all());
        assert!(
            run.stderr.contains(name),
            "{case}: stderr should say which path is unusable:\n{}",
            run.stderr
        );
    }
}

// PRD-001 AC14
#[test]
fn group_header_shows_the_folder_on_disk_when_it_differs() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("index.md"), "Root.\n").expect("write");
    std::fs::create_dir_all(dir.path().join("010-a/020-b")).expect("mkdir");
    std::fs::write(dir.path().join("010-a/index.md"), "A.\n").expect("write");
    std::fs::write(dir.path().join("010-a/020-b/index.md"), "[[nowhere]]\n").expect("write");
    let run = check_path(dir.path());
    assert_eq!(run.code, Some(1), "output:\n{}", run.all());
    assert!(
        run.stdout.lines().any(|l| l == "a/b  (010-a/020-b)"),
        "output:\n{}",
        run.all()
    );
}

// PRD-001 AC14
#[test]
fn malformed_config_exits_two_and_names_the_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("index.md"), "Root.\n").expect("write");
    std::fs::write(dir.path().join("gyeoldaero.toml"), "name = [unclosed\n").expect("write");
    let run = check_path(dir.path());
    assert_eq!(run.code, Some(2), "output:\n{}", run.all());
    assert!(run.stderr.contains("gyeoldaero.toml"), "{}", run.stderr);
    assert!(
        !run.stderr.contains("Pass the folder that holds your notes"),
        "wrong hint for a config error:\n{}",
        run.stderr
    );
}

// PRD-001 AC14
#[cfg(unix)]
#[test]
fn control_and_bidi_characters_are_escaped_in_output() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("index.md"), "Root.\n").expect("write");
    let evil = dir
        .path()
        .join(OsStr::from_bytes(b"a\x1b[2Jb\nerror: fake\x08\x07"));
    std::fs::create_dir(&evil).expect("mkdir");
    std::fs::write(
        evil.join("index.md"),
        "[[p\rq\u{9b}x\u{202e}y]] [[z\u{2066}w]]\n",
    )
    .expect("write");
    std::fs::write(dir.path().join("stray\x1b]52;c;aGk=\x07.md"), "x\n").expect("write");

    let run = check_path(dir.path());
    let all = run.all();
    assert_eq!(run.code, Some(1), "output:\n{all}");
    for bad in [
        '\x1b', '\r', '\x08', '\x07', '\u{9b}', '\u{202e}', '\u{2066}',
    ] {
        assert!(!all.contains(bad), "{bad:?} reached the terminal:\n{all:?}");
    }
    // No forged lines: every line is a header, an indented diagnostic, blank, or the
    // summary.
    let _ = summary(&run);
    for line in run.stdout.lines() {
        assert!(
            !line.starts_with("error:") && !line.starts_with("warning:"),
            "forged line {line:?} in:\n{all}"
        );
    }
}
