//! Splitting `index.md` into frontmatter and body.

use serde_yaml_ng::{Mapping, Value};

use crate::diagnostic::Level;

/// The result of reading one `index.md` (already LF-normalized).
#[derive(Debug, Default)]
pub struct Split {
    /// `title`, converted to a string when it is a number or boolean.
    pub title: Option<String>,
    /// Every frontmatter field.
    pub frontmatter: Mapping,
    /// The Markdown body.
    pub body: String,
    /// Problems to report for this note.
    pub problems: Vec<(Level, String)>,
}

/// Splits `text` into frontmatter and body. Never fails: problems come back in
/// [`Split::problems`].
pub fn split(text: &str) -> Split {
    let whole = |level: Level, message: String| Split {
        body: text.to_owned(),
        problems: vec![(level, message)],
        ..Split::default()
    };

    let Some(after_open) = strip_dash_line(text) else {
        return Split {
            body: text.to_owned(),
            ..Split::default()
        };
    };
    let Some((yaml, body)) = find_close(after_open) else {
        return whole(
            Level::Warning,
            "index.md starts with a --- line but has no closing --- line, so it is read as \
             Markdown (the whole file is the body). If you meant frontmatter, add a closing \
             --- line."
                .to_owned(),
        );
    };

    let value = if yaml.trim().is_empty() {
        Value::Null
    } else {
        match serde_yaml_ng::from_str::<Value>(yaml) {
            Ok(value) => value,
            Err(e) => {
                return whole(
                    Level::Error,
                    format!(
                        "The frontmatter in index.md is not valid YAML ({e}). The whole file is \
                         shown as the body until the YAML between the --- lines is fixed."
                    ),
                );
            }
        }
    };
    let frontmatter = match value {
        Value::Null => Mapping::new(),
        Value::Mapping(m) => m,
        _ => {
            return whole(
                Level::Warning,
                "The block between the --- lines at the top of index.md is not a list of \
                 `key: value` fields, so it is read as Markdown (the whole file is the body). \
                 If you meant frontmatter, write fields like `title: My note`."
                    .to_owned(),
            );
        }
    };

    let mut problems = Vec::new();
    let title = match frontmatter.get("title") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        Some(_) => {
            problems.push((
                Level::Error,
                "The frontmatter `title` is a list or a map; it must be text. The slug is shown \
                 instead until it is fixed, e.g. `title: My note`."
                    .to_owned(),
            ));
            None
        }
    };
    let title = title.filter(|t| !t.trim().is_empty());

    Split {
        title,
        frontmatter,
        body: body.to_owned(),
        problems,
    }
}

/// If `text` starts with a `---` line, the text after that line.
fn strip_dash_line(text: &str) -> Option<&str> {
    let (line, rest) = text.split_once('\n').unwrap_or((text, ""));
    (line.trim_end() == "---").then_some(rest)
}

/// Finds the closing `---` line: returns the YAML before it and the body after it.
fn find_close(text: &str) -> Option<(&str, &str)> {
    let mut start = 0;
    while start < text.len() {
        let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
        if text[start..end].trim_end() == "---" {
            let body_start = (end + 1).min(text.len());
            return Some((&text[..start], &text[body_start..]));
        }
        start = end + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_line_at_end_of_file_without_newline() {
        let s = split("---\ntitle: T\n---");
        assert_eq!(s.title.as_deref(), Some("T"));
        assert_eq!(s.body, "");
        assert!(s.problems.is_empty());
    }

    #[test]
    fn blank_title_means_no_title() {
        let s = split("---\ntitle: \"  \"\n---\nBody\n");
        assert_eq!(s.title, None);
        assert!(s.problems.is_empty());
    }

    #[test]
    fn scalar_block_is_read_as_markdown() {
        let text = "---\nIntro.\n\nSection\n---\n";
        let s = split(text);
        assert_eq!(s.body, text);
        assert_eq!(s.problems.len(), 1);
        assert_eq!(s.problems[0].0, Level::Warning);
    }
}
