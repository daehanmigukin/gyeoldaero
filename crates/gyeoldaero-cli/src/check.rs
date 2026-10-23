//! `gyeoldaero check [PATH]`: print diagnostics grouped by note, then a summary.

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use anyhow::Context;
use gyeoldaero_core::{Config, Diagnostic, Level, NotePath, Notebook};

/// Runs the check and returns the exit code: 0 clean or warnings only, 1 errors,
/// 2 unusable root.
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
            eprintln!("error: {e:#}");
            eprintln!("Pass the folder that holds your notes: gyeoldaero check <path>");
            ExitCode::from(2)
        }
    }
}

fn load(root: &Path) -> anyhow::Result<(Notebook, Vec<Diagnostic>)> {
    let config = Config::load(root)
        .with_context(|| format!("cannot use {} as a notebook", root.display()))?;
    let (notebook, mut diagnostics) = Notebook::load(root, &config)
        .with_context(|| format!("cannot check {}", root.display()))?;
    let mut all = config.diagnostics;
    all.append(&mut diagnostics);
    Ok((notebook, all))
}

/// Diagnostics grouped under their note path in tree order, then the summary line.
fn report(notebook: &Notebook, diagnostics: &[Diagnostic]) -> String {
    let mut by_path: HashMap<&NotePath, Vec<&Diagnostic>> = HashMap::new();
    for d in diagnostics {
        by_path.entry(&d.path).or_default().push(d);
    }
    let mut out = String::new();
    let mut group = |path: &NotePath, items: Vec<&Diagnostic>| {
        out.push_str(path.display_name());
        out.push('\n');
        for d in items {
            out.push_str(&format!("  {}: {}\n", d.level, d.message));
        }
        out.push('\n');
    };
    for id in notebook.iter() {
        let path = &notebook.get(id).path;
        if let Some(items) = by_path.remove(path) {
            group(path, items);
        }
    }
    // Anything not attached to a note in the tree (shouldn't happen) still gets printed.
    let mut rest: Vec<_> = by_path.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(b.0));
    for (path, items) in rest {
        group(path, items);
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
}
