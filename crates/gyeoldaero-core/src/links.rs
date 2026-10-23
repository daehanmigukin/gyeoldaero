//! Wiki links (`[[path]]`, `[[path|label]]`).

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
