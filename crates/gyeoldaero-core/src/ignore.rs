//! Entries the scanner skips entirely (architecture.md → Scanning rules).

/// True for names the scanner never looks at: hidden entries (which covers `.git`,
/// `.stfolder`, `.stversions` and `.syncthing.*.tmp`), `~syncthing~*` temp files and
/// editor artifacts (`*.swp`, `*.swo`, `*~`, vim's `4913` write probe).
pub fn is_ignored(name: &str) -> bool {
    name.starts_with('.')
        || name.starts_with("~syncthing~")
        || name.ends_with(".swp")
        || name.ends_with(".swo")
        || name.ends_with('~')
        || name == "4913"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_names_are_kept() {
        for name in ["index.md", "caches", "010-caches", "photo.png", "49130"] {
            assert!(!is_ignored(name), "{name}");
        }
    }
}
