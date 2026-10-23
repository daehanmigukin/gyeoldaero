//! Notebook model for gyeoldaero: scanning a notebook folder, parsing notes, ordering,
//! link resolution, diagnostics and configuration.
//!
//! Core knows nothing of HTML, HTTP or watching (architecture.md → Shape).

pub mod config;
pub mod diagnostic;
pub mod error;
mod frontmatter;
mod fsread;
mod ignore;
pub mod links;
mod name;
pub mod notebook;

pub use config::{BuildConfig, CONFIG_FILE, Config, ServeConfig};
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
