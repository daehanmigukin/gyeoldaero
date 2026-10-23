//! Watching and serving for gyeoldaero: debounced rebuilds of a notebook and an HTTP server
//! for the output directory.
//!
//! Placeholder until PRD-004 and PRD-005.

/// Address the server binds to when neither config nor flags say otherwise.
pub const DEFAULT_BIND: &str = "127.0.0.1:4321";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_localhost() {
        assert!(DEFAULT_BIND.starts_with("127.0.0.1:"));
    }
}
