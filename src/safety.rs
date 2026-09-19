use crate::ui;
use anyhow::{Context, Result};
use console::style;
use dialoguer::Select;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictAction {
    Overwrite,
    Keep,
    Diff,
}

fn get_backup_timestamp() -> String {
    let now = unsafe { libc::time(std::ptr::null_mut()) };
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&now, &mut tm) };
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

/// Moves `target_path` to `.bak/<timestamp>_<item_name>/` for safe recovery.
pub fn quarantine(target_path: &Path, backup_dir: &Path) -> Result<PathBuf> {
    if !backup_dir.exists() {
        fs::create_dir_all(backup_dir)
            .with_context(|| format!("Failed to create backup directory '{}'", backup_dir.display()))?;
    }

    let timestamp = get_backup_timestamp();
    let name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("item");
    let backup_dest = backup_dir.join(format!("{}_{}", timestamp, name));

    info!(
        target = %target_path.display(),
        backup = %backup_dest.display(),
        "Quarantining existing file/directory before replacement"
    );

    if backup_dest.exists() {
        let unique_dest = backup_dir.join(format!("{}_{}_{}", timestamp, name, std::process::id()));
        fs::rename(target_path, &unique_dest)
            .or_else(|_| copy_and_remove(target_path, &unique_dest))
            .with_context(|| format!("Failed to move '{}' to backup '{}'", target_path.display(), unique_dest.display()))?;
        return Ok(unique_dest);
    }

    fs::rename(target_path, &backup_dest)
        .or_else(|_| copy_and_remove(target_path, &backup_dest))
        .with_context(|| format!("Failed to move '{}' to backup '{}'", target_path.display(), backup_dest.display()))?;

    Ok(backup_dest)
}

fn copy_and_remove(from: &Path, to: &Path) -> Result<()> {
    if from.is_dir() {
        copy_dir_all(from, to)?;
        fs::remove_dir_all(from)?;
    } else {
        fs::copy(from, to)?;
        fs::remove_file(from)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Prompts user for resolution when a conflict is detected.
pub fn prompt_conflict_action(item_name: &str, target_path: &Path) -> Result<ConflictAction> {
    println!();
    println!("{} Conflict detected at {}", ui::badge_warn(), style(target_path.display()).bold());
    println!("  Item:   {}", style(item_name).cyan());
    println!("  Target: {}", style(target_path.display()).dim());
    println!("  Status: System target already exists as a regular file or directory\n");

    let choices = &[
        "Overwrite (quarantine to .bak/ & create symlink)",
        "Keep existing (skip this item)",
        "View unified diff",
    ];

    let selection = match Select::new()
        .with_prompt("Choose an action")
        .items(choices)
        .default(0)
        .interact_opt()
    {
        Ok(s) => s,
        Err(e) => {
            warn!("Interactive prompt unavailable ({}). Defaulting to keep. Pass --force to overwrite.", e);
            println!("  {} Non-terminal environment: keeping existing item. Pass --force to overwrite.", ui::badge_dim());
            return Ok(ConflictAction::Keep);
        }
    };

    match selection {
        Some(0) => Ok(ConflictAction::Overwrite),
        Some(1) => Ok(ConflictAction::Keep),
        Some(2) => Ok(ConflictAction::Diff),
        _ => {
            warn!("No selection made, skipping by default");
            Ok(ConflictAction::Keep)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_quarantine_file() {
        let dir = tempdir().unwrap();
        let target = dir.path().join(".zshrc");
        let backup_dir = dir.path().join(".bak");

        fs::write(&target, "export FOO=BAR").unwrap();
        let backed_up = quarantine(&target, &backup_dir).unwrap();

        assert!(!target.exists(), "Original file should have been moved");
        assert!(backed_up.exists(), "Backup should exist");
        assert_eq!(fs::read_to_string(backed_up).unwrap(), "export FOO=BAR");
    }
}
