//! `gyeoldaero.toml` in the notebook root.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::diagnostic::{Diagnostic, Level};
use crate::error::CoreError;
use crate::notebook::NotePath;

/// Name of the configuration file in the notebook root.
pub const CONFIG_FILE: &str = "gyeoldaero.toml";

/// Notebook configuration. Every field has a default; the file itself is optional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Shown at the root of the path bar. Default: the notebook folder's name.
    /// Display only: never part of a path.
    pub name: String,
    /// `[serve]` section.
    pub serve: ServeConfig,
    /// `[build]` section.
    pub build: BuildConfig,
    /// Warnings about the file itself (unknown settings), reported by `check`.
    pub diagnostics: Vec<Diagnostic>,
}

/// `[serve]` section of `gyeoldaero.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeConfig {
    /// Address to bind. Default `127.0.0.1:4321`.
    pub bind: String,
}

/// `[build]` section of `gyeoldaero.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfig {
    /// Output directory, absolute. `~` is expanded and a relative path resolves against
    /// the notebook root. Default: `<cache dir>/gyeoldaero/<folder-name>-<hash>`. Ignored
    /// by the scanner when it sits inside the notebook.
    pub out: PathBuf,
}

impl Config {
    /// Reads `<root>/gyeoldaero.toml`, or returns defaults when it is absent.
    pub fn load(root: &Path) -> Result<Config, CoreError> {
        let unreadable = |source| CoreError::UnreadableRoot {
            path: root.to_path_buf(),
            source,
        };
        let abs_root = std::path::absolute(root).map_err(unreadable)?;
        let canonical = fs::canonicalize(root).map_err(unreadable)?;
        if !fs::metadata(&canonical).map_err(unreadable)?.is_dir() {
            return Err(unreadable(std::io::Error::new(
                ErrorKind::NotADirectory,
                "not a directory",
            )));
        }
        let folder = abs_root
            .file_name()
            .or_else(|| canonical.file_name())
            .map_or_else(
                || "notebook".to_owned(),
                |n| n.to_string_lossy().into_owned(),
            );

        let mut config = Config {
            name: folder.clone(),
            serve: ServeConfig {
                bind: "127.0.0.1:4321".to_owned(),
            },
            build: BuildConfig {
                out: default_out(&folder, &canonical),
            },
            diagnostics: Vec::new(),
        };

        let file = abs_root.join(CONFIG_FILE);
        let meta = match fs::symlink_metadata(&file) {
            Ok(meta) => meta,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(config),
            Err(source) => return Err(CoreError::Io { path: file, source }),
        };
        if !meta.is_file() {
            config.warn(format!(
                "{CONFIG_FILE} is not a regular file (gyeoldaero doesn't follow symbolic links), \
                 so the defaults are used. Replace it with a regular file."
            ));
            return Ok(config);
        }
        let text = fs::read_to_string(&file).map_err(|source| CoreError::Io {
            path: file.clone(),
            source,
        })?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let table: toml::Table = toml::from_str(text).map_err(|e| CoreError::Config {
            path: file.clone(),
            message: e.to_string(),
        })?;
        config.apply(&table, &file, &abs_root)?;
        Ok(config)
    }

    fn warn(&mut self, message: String) {
        self.diagnostics.push(Diagnostic {
            level: Level::Warning,
            path: NotePath::default(),
            message,
        });
    }

    fn apply(&mut self, table: &toml::Table, file: &Path, root: &Path) -> Result<(), CoreError> {
        let bad = |message: String| CoreError::Config {
            path: file.to_path_buf(),
            message,
        };
        for (key, value) in table {
            match (key.as_str(), value) {
                ("name", toml::Value::String(name)) => {
                    if !name.trim().is_empty() {
                        self.name = name.clone();
                    }
                }
                ("name", _) => return Err(bad("`name` must be a string".to_owned())),
                ("serve", toml::Value::Table(serve)) => {
                    for (key, value) in serve {
                        match (key.as_str(), value) {
                            ("bind", toml::Value::String(bind)) => self.serve.bind = bind.clone(),
                            ("bind", _) => {
                                return Err(bad("`serve.bind` must be a string".to_owned()));
                            }
                            (other, _) => self.unknown(&format!("serve.{other}")),
                        }
                    }
                }
                ("build", toml::Value::Table(build)) => {
                    for (key, value) in build {
                        match (key.as_str(), value) {
                            ("out", toml::Value::String(out)) => {
                                self.build.out = resolve_out(out, root).map_err(bad)?;
                            }
                            ("out", _) => {
                                return Err(bad("`build.out` must be a string".to_owned()));
                            }
                            (other, _) => self.unknown(&format!("build.{other}")),
                        }
                    }
                }
                ("serve" | "build", _) => return Err(bad(format!("`[{key}]` must be a table"))),
                (other, _) => self.unknown(other),
            }
        }
        Ok(())
    }

    fn unknown(&mut self, key: &str) {
        self.warn(format!(
            "{CONFIG_FILE}: unknown setting `{key}` is ignored. Check the spelling; the known \
             settings are `name`, `serve.bind` and `build.out`."
        ));
    }
}

/// `<cache dir>/gyeoldaero/<folder-name>-<8 hex chars of FNV-1a of the canonical root>`.
fn default_out(folder: &str, canonical_root: &Path) -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    let hash = fnv1a_32(canonical_root.as_os_str().as_encoded_bytes());
    base.join("gyeoldaero").join(format!("{folder}-{hash:08x}"))
}

/// 32-bit FNV-1a. Written out because `std`'s hasher isn't stable across Rust versions,
/// and the output dir must not move when gyeoldaero is rebuilt.
fn fnv1a_32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5_u32, |hash, b| {
        (hash ^ u32::from(*b)).wrapping_mul(0x0100_0193)
    })
}

/// Expands `~` / `~/…` and resolves a relative path against the notebook root.
fn resolve_out(out: &str, root: &Path) -> Result<PathBuf, String> {
    let home_relative = if out == "~" {
        Some("")
    } else {
        out.strip_prefix("~/")
    };
    let path = match home_relative {
        Some(rest) => {
            let home = dirs::home_dir().ok_or_else(|| {
                "`build.out` starts with ~ but the home directory is unknown; write the full \
                 path instead"
                    .to_owned()
            })?;
            home.join(rest)
        }
        None => root.join(out),
    };
    Ok(path
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_reference_values() {
        assert_eq!(fnv1a_32(b""), 0x811c_9dc5);
        assert_eq!(fnv1a_32(b"a"), 0xe40c_292c);
        assert_eq!(fnv1a_32(b"foobar"), 0xbf9c_f968);
    }

    #[test]
    fn absolute_out_is_kept_and_tilde_user_is_not_expanded() {
        let root = Path::new("/notes");
        assert_eq!(
            resolve_out("/srv/site", root),
            Ok(PathBuf::from("/srv/site"))
        );
        assert_eq!(
            resolve_out("~bob/site", root),
            Ok(PathBuf::from("/notes/~bob/site"))
        );
    }
}
