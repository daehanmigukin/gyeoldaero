//! Rendering for gyeoldaero: turning the notebook model into static HTML pages with bundled
//! assets.
//!
//! Placeholder until PRD-002.

/// Directory under the output root that holds bundled static files (CSS, JS, fonts).
pub const ASSET_DIR: &str = "_gyeoldaero";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_dir_is_underscore_prefixed() {
        assert_eq!(ASSET_DIR, "_gyeoldaero");
    }
}
