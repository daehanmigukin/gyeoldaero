//! Notebook model for gyeoldaero: scanning a notebook folder, parsing notes, ordering,
//! link resolution, diagnostics and configuration.
//!
//! The public API is being built by PRD-001. Items marked as stubs below exist so the
//! PRD-001 tests compile; they are filled in during implementation.

pub mod config;
pub mod diagnostic;
pub mod error;
pub mod links;
pub mod notebook;

pub use config::{BuildConfig, Config, ServeConfig};
pub use diagnostic::{Diagnostic, Level};
pub use error::CoreError;
pub use links::Link;
pub use notebook::{Note, NoteId, NotePath, Notebook};

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
