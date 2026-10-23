//! `gyeoldaero check [PATH]`: print diagnostics grouped by note, then a summary.

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use gyeoldaero_core::{CONFIG_FILE, Config, CoreError, Diagnostic, Level, Notebook};

/// Runs the check and returns the exit code: 0 clean or warnings only, 1 errors,
/// 2 unusable root or `gyeoldaero.toml`.
pub fn run(path: Option<&Path>) -> ExitCode {
    let root = path.unwrap_or(Path::new("."));
    match load(root) {
        Ok((notebook, diagnostics)) => {
            let out = report(&notebook, &diagnostics);
            // A closed stdout (e.g. piped into `head`) shouldn't change the verdict.
            let _ = io::stdout().lock().write_all(out.as_bytes());
            if diagnostics.iter().any(|d| d.level == Level::Error) {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("error: {}", terminal_safe(&format!("{e:#}")));
            eprintln!("{}", terminal_safe(&hint(&e)));
            ExitCode::from(2)
        }
    }
}

fn load(root: &Path) -> anyhow::Result<(Notebook, Vec<Diagnostic>)> {
    let config = Config::load(root)?;
    let (notebook, mut diagnostics) = Notebook::load(root, &config)
        .with_context(|| format!("cannot check {}", root.display()))?;
    let mut all = config.diagnostics;
    all.append(&mut diagnostics);
    Ok((notebook, all))
}

/// What to do about a hard error, depending on which one it is.
fn hint(e: &anyhow::Error) -> String {
    match e.downcast_ref::<CoreError>() {
        Some(CoreError::Config { .. }) => {
            format!("Fix {CONFIG_FILE}, or remove it to use the defaults.")
        }
        Some(CoreError::Io { path, .. }) => {
            format!("Check that {} is readable.", path.display())
        }
        Some(CoreError::UnreadableRoot { .. }) | None => {
            "Pass the folder that holds your notes: gyeoldaero check <path>".to_owned()
        }
    }
}

/// Diagnostics grouped under their note in tree order, then the summary line.
///
/// Groups are keyed by the folder on disk, so notes whose slug paths clash stay apart.
fn report(notebook: &Notebook, diagnostics: &[Diagnostic]) -> String {
    let mut by_dir: HashMap<&PathBuf, Vec<&Diagnostic>> = HashMap::new();
    for d in diagnostics {
        by_dir.entry(&d.dir).or_default().push(d);
    }
    let mut out = String::new();
    let mut group = |items: Vec<&Diagnostic>| {
        let Some(first) = items.first() else { return };
        out.push_str(&terminal_safe(&first.location()));
        out.push('\n');
        for d in items {
            out.push_str(&format!("  {}: {}\n", d.level, terminal_safe(&d.message)));
        }
        out.push('\n');
    };
    for id in notebook.iter() {
        if let Some(items) = by_dir.remove(&notebook.get(id).dir) {
            group(items);
        }
    }
    // Anything not attached to a note in the tree (shouldn't happen) still gets printed.
    let mut rest: Vec<_> = by_dir.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(b.0));
    for (_, items) in rest {
        group(items);
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.level == Level::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    out.push_str(&format!(
        "{}, {} in {}\n",
        count(errors, "error"),
        count(warnings, "warning"),
        count(notebook.len(), "note"),
    ));
    out
}

/// Escapes characters that could steer a terminal: C0 and C1 controls (newline
/// included), DEL, and bidi overrides and isolates. File names and link targets are
/// untrusted, and a raw ESC, CR or newline in them could recolor, overwrite or forge
/// lines of the report.
fn terminal_safe(s: &str) -> Cow<'_, str> {
    let unsafe_char = |c: char| {
        c.is_control()
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}')
            || ('\u{202a}'..='\u{202e}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c)
    };
    if !s.chars().any(unsafe_char) {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if unsafe_char(c) {
            out.push_str(&format!("\\u{{{:x}}}", u32::from(c)));
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

/// `1 error`, `0 errors`, `2 errors`: plural for every count except exactly 1.
fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_except_exactly_one() {
        assert_eq!(count(0, "warning"), "0 warnings");
        assert_eq!(count(1, "warning"), "1 warning");
        assert_eq!(count(2, "error"), "2 errors");
    }

    #[test]
    fn terminal_safe_escapes_controls_and_bidi() {
        assert_eq!(terminal_safe("plain 노트"), "plain 노트");
        assert_eq!(terminal_safe("a\x1b[2Jb\nc"), "a\\u{1b}[2Jb\\u{a}c");
        assert_eq!(terminal_safe("x\u{9b}y\u{202e}z"), "x\\u{9b}y\\u{202e}z");
    }
}
