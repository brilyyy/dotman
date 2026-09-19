use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymlinkStatus {
    Valid,
    Conflict(String),
    Broken(String),
    Missing,
}

/// Atomically writes content to a file via a temporary file and atomic rename in the same directory.
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory '{}'", parent.display()))?;
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let tmp_path = parent.join(format!(".{}.tmp.{}", file_name, std::process::id()));

    fs::write(&tmp_path, content)
        .with_context(|| format!("Failed to write temporary file '{}'", tmp_path.display()))?;

    // Copy permissions from target if it already exists
    if let Ok(meta) = fs::metadata(path) {
        let _ = fs::set_permissions(&tmp_path, meta.permissions());
    }

    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to atomically rename '{}' to '{}'",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

/// Expands leading `~`, `$HOME`, or standard XDG environment variables.
pub fn expand_home(path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with("~/") || path_str == "~" {
        let home = dirs::home_dir().context("Could not determine user home directory")?;
        if path_str == "~" {
            Ok(home)
        } else {
            Ok(home.join(&path_str[2..]))
        }
    } else if path_str.starts_with('$') {
        let (var_name, rest) = match path_str.find('/') {
            Some(idx) => (&path_str[1..idx], &path_str[idx + 1..]),
            None => (&path_str[1..], ""),
        };

        let base_path = match var_name {
            "HOME" => dirs::home_dir().context("Could not determine user home directory")?,
            "XDG_CONFIG_HOME" => dirs::config_dir().context("Could not determine config directory")?,
            "XDG_DATA_HOME" => dirs::data_dir().context("Could not determine data directory")?,
            other => match std::env::var(other) {
                Ok(val) => PathBuf::from(val),
                Err(_) => anyhow::bail!("Environment variable '${}' is not set", other),
            },
        };

        if rest.is_empty() {
            Ok(base_path)
        } else {
            Ok(base_path.join(rest))
        }
    } else {
        Ok(PathBuf::from(path_str))
    }
}

/// Replaces user home directory with `~` for concise display.
pub fn contract_home(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.display().to_string()
}

/// Creates a Unix symlink from `source` to `target`.
/// Ensures parent directories of `target` exist.
pub fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory '{}'", parent.display()))?;
        }
    }
    unix_fs::symlink(source, target)
        .with_context(|| format!("Failed to create symlink '{}' -> '{}'", target.display(), source.display()))?;
    Ok(())
}

/// Recursively copies directory or file while preserving file modes/permissions.
pub fn copy_preserving_permissions(src: &Path, dst: &Path) -> Result<()> {
    let meta = fs::metadata(src)
        .with_context(|| format!("Failed to read metadata for '{}'", src.display()))?;

    if meta.is_dir() {
        fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory '{}'", dst.display()))?;
        fs::set_permissions(dst, meta.permissions())?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());
            copy_preserving_permissions(&child_src, &child_dst)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::copy(src, dst)
            .with_context(|| format!("Failed to copy '{}' to '{}'", src.display(), dst.display()))?;
        fs::set_permissions(dst, meta.permissions())?;
    }

    Ok(())
}

/// Checks the status of a target symlink relative to the expected repo source.
pub fn check_symlink_status(repo_source: &Path, target: &Path) -> SymlinkStatus {
    let symlink_metadata = match fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return SymlinkStatus::Missing,
        Err(e) => return SymlinkStatus::Conflict(format!("Error reading metadata: {}", e)),
    };

    if symlink_metadata.file_type().is_symlink() {
        match fs::read_link(target) {
            Ok(link_dest) => {
                let resolved_link = if link_dest.is_relative() {
                    target.parent().unwrap_or_else(|| Path::new(".")).join(&link_dest)
                } else {
                    link_dest.clone()
                };

                let canonical_source = repo_source.canonicalize().unwrap_or_else(|_| repo_source.to_path_buf());
                let canonical_link = resolved_link.canonicalize().unwrap_or_else(|_| resolved_link.clone());

                if canonical_source == canonical_link {
                    if repo_source.exists() {
                        SymlinkStatus::Valid
                    } else {
                        SymlinkStatus::Broken(format!("Source does not exist: {}", repo_source.display()))
                    }
                } else if !resolved_link.exists() {
                    SymlinkStatus::Broken(format!("Dangling symlink points to non-existent '{}'", link_dest.display()))
                } else {
                    SymlinkStatus::Conflict(format!("Symlink points to '{}' instead of '{}'", link_dest.display(), repo_source.display()))
                }
            }
            Err(e) => SymlinkStatus::Broken(format!("Cannot read symlink: {}", e)),
        }
    } else {
        SymlinkStatus::Conflict("Target exists and is a regular file or directory".to_string())
    }
}

/// Computes a unified text diff between two files with subtle ANSI colors.
pub fn compute_diff(path_a: &Path, path_b: &Path) -> Result<String> {
    let text_a = if path_a.is_file() {
        fs::read_to_string(path_a).unwrap_or_else(|_| "<binary or unreadable>".to_string())
    } else if path_a.is_dir() {
        format!("<directory: {}>", path_a.display())
    } else {
        "<missing>".to_string()
    };

    let text_b = if path_b.is_file() {
        fs::read_to_string(path_b).unwrap_or_else(|_| "<binary or unreadable>".to_string())
    } else if path_b.is_dir() {
        format!("<directory: {}>", path_b.display())
    } else {
        "<missing>".to_string()
    };

    let diff = TextDiff::from_lines(&text_a, &text_b);
    let mut output = String::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                output.push_str(&format!("{}", console::style(format!("-{}", change)).red()));
            }
            ChangeTag::Insert => {
                output.push_str(&format!("{}", console::style(format!("+{}", change)).green()));
            }
            ChangeTag::Equal => {
                output.push_str(&format!(" {}", change));
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("atomic.txt");

        atomic_write(&target, "first write").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "first write");

        atomic_write(&target, "second write").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "second write");
    }

    #[test]
    fn test_permission_preservation() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("script.sh");
        let dst = dir.path().join("script_copy.sh");

        fs::write(&src, "#!/bin/sh\necho hi").unwrap();
        let mut perms = fs::metadata(&src).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&src, perms).unwrap();

        copy_preserving_permissions(&src, &dst).unwrap();
        let dst_perms = fs::metadata(&dst).unwrap().permissions();
        assert_eq!(dst_perms.mode() & 0o777, 0o755);
    }

    #[test]
    fn test_symlink_lifecycle() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let target = dir.path().join("target.txt");

        fs::write(&source, "hello world").unwrap();

        assert_eq!(check_symlink_status(&source, &target), SymlinkStatus::Missing);

        create_symlink(&source, &target).unwrap();
        assert_eq!(check_symlink_status(&source, &target), SymlinkStatus::Valid);

        fs::remove_file(&target).unwrap();
        fs::write(&target, "conflict file").unwrap();
        match check_symlink_status(&source, &target) {
            SymlinkStatus::Conflict(_) => (),
            other => panic!("Expected conflict, got {:?}", other),
        }
    }
}
