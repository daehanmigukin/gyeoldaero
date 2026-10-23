//! Wiki links (`[[path]]`, `[[path|label]]`).

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};

use crate::notebook::NoteId;

/// One outgoing wiki link of a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The path as written between `[[` and `]]` (without the label).
    pub target: String,
    /// The text after `|`, if any.
    pub label: Option<String>,
    /// The note it points to, or `None` when it doesn't resolve.
    pub resolved: Option<NoteId>,
}

/// The comrak options gyeoldaero parses notes with. Wiki links use comrak's
/// `[[url|title]]` extension, so code spans, fenced and indented code, escaping and
/// trimming are handled exactly as CommonMark defines them.
pub fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.wikilinks_title_after_pipe = true;
    options
}

/// Extracts wiki links from a Markdown body, in body order, unresolved.
pub fn extract(body: &str) -> Vec<Link> {
    let arena = Arena::new();
    let root = parse_document(&arena, body, &markdown_options());
    let mut links = Vec::new();
    for node in root.descendants() {
        let target = match &node.data().value {
            NodeValue::WikiLink(link) => link.url.trim().to_owned(),
            _ => continue,
        };
        if target.is_empty() {
            continue;
        }
        let mut text = String::new();
        for child in node.descendants().skip(1) {
            if let NodeValue::Text(t) = &child.data().value {
                text.push_str(t);
            }
        }
        let text = text.trim();
        // comrak uses the target as the text when there is no label.
        let label = (!text.is_empty() && text != target).then(|| text.to_owned());
        links.push(Link {
            target,
            label,
            resolved: None,
        });
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(body: &str) -> Vec<(String, Option<String>)> {
        extract(body)
            .into_iter()
            .map(|l| (l.target, l.label))
            .collect()
    }

    #[test]
    fn empty_links_and_empty_labels() {
        assert!(targets("[[]] and [[ ]]").is_empty());
        assert_eq!(targets("[[a|]]"), [("a".to_owned(), None)]);
        assert_eq!(
            targets("[[ a/b | c ]]"),
            [("a/b".to_owned(), Some("c".to_owned()))]
        );
    }

    #[test]
    fn embeds_are_links() {
        assert_eq!(targets("![[a]]"), [("a".to_owned(), None)]);
    }
}
