//! Reading notebook files without following symbolic links, with a size cap.

use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::Path;

/// Largest `index.md` or `gyeoldaero.toml` gyeoldaero reads: 8 MiB.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// What [`read_regular`] found.
#[derive(Debug)]
pub enum Contents {
    /// The file's bytes.
    Bytes(Vec<u8>),
    /// Not a regular file (symlink, FIFO, folder, …), or it changed between the check and
    /// the open. Nothing was read.
    NotRegular,
    /// Larger than the cap. Nothing useful was kept.
    TooLarge,
}

/// Reads `path` only if it is a regular file of at most `max` bytes.
///
/// `lstat` first, so a symlink or FIFO found there is not opened (opening a FIFO would
/// block). After opening, the handle's metadata must describe the same regular file
/// `lstat` saw (same device and inode on Unix), so a swap for a symlink in between is
/// caught before anything is read. The swap itself still makes `open` follow the link,
/// or block on a FIFO; closing that needs `O_NOFOLLOW | O_NONBLOCK` (see PRD-001 Notes,
/// "Must do before PRD-002 renders any page").
pub fn read_regular(path: &Path, max: u64) -> io::Result<Contents> {
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() {
        return Ok(Contents::NotRegular);
    }
    if before.len() > max {
        return Ok(Contents::TooLarge);
    }
    let file = File::open(path)?;
    let after = file.metadata()?;
    if !after.is_file() || !same_file(&before, &after) {
        return Ok(Contents::NotRegular);
    }
    let capacity = usize::try_from(after.len().min(max)).unwrap_or(0);
    let mut bytes = Vec::with_capacity(capacity);
    file.take(max.saturating_add(1)).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max {
        return Ok(Contents::TooLarge);
    }
    Ok(Contents::Bytes(bytes))
}

#[cfg(unix)]
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(not(unix))]
fn same_file(_a: &Metadata, _b: &Metadata) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_small_files_and_caps_large_ones() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("f");
        fs::write(&path, b"12345").expect("write");
        assert!(matches!(read_regular(&path, 5), Ok(Contents::Bytes(b)) if b == b"12345"));
        assert!(matches!(read_regular(&path, 4), Ok(Contents::TooLarge)));
        assert!(matches!(
            read_regular(dir.path(), 5),
            Ok(Contents::NotRegular)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_read() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("target"), b"secret").expect("write");
        std::os::unix::fs::symlink(dir.path().join("target"), dir.path().join("link"))
            .expect("symlink");
        assert!(matches!(
            read_regular(&dir.path().join("link"), 100),
            Ok(Contents::NotRegular)
        ));
    }
}
