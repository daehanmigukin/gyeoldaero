//! Notebook model for gyeoldaero: scanning a notebook folder, parsing notes, ordering,
//! link resolution, diagnostics and configuration.
//!
//! Placeholder until PRD-001.

/// File name that holds a note's body inside its folder.
pub const NOTE_FILE: &str = "index.md";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_file_is_index_md() {
        assert_eq!(NOTE_FILE, "index.md");
    }
}
