use crate::cli::RestoreArgs;
use crate::config::DotConfig;
use crate::fs::{copy_preserving_permissions, expand_home};
use crate::ui;
use anyhow::{Context, Result};
use console::style;
use dialoguer::Select;
use std::fs;
use std::path::PathBuf;

pub fn execute(args: RestoreArgs) -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let repo_root = manifest_path
        .parent()
        .context("Manifest path has no parent directory")?
        .to_path_buf();
    let config = DotConfig::load_from_path(&manifest_path)?;

    let backup_dir = repo_root.join(&config.settings.backup_dir);
    if !backup_dir.exists() {
        println!("No backups found in '{}'.", backup_dir.display());
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(ref filter) = args.item {
            if !file_name.contains(filter) {
                continue;
            }
        }
        entries.push((file_name, entry.path()));
    }

    // Sort descending by name (most recent timestamp first)
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    if entries.is_empty() {
        println!("No backups found matching criteria in '{}'.", backup_dir.display());
        return Ok(());
    }

    println!("Found {} backup(s) in quarantine:\n", entries.len());

    let labels: Vec<String> = entries
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    let selection = Select::new()
        .with_prompt("Select backup to restore")
        .items(&labels)
        .default(0)
        .interact_opt()?;

    let selected_idx = match selection {
        Some(idx) => idx,
        None => {
            println!("Restore cancelled.");
            return Ok(());
        }
    };

    let (backup_name, backup_path) = &entries[selected_idx];

    // Deduce original target path from dot.toml items or prompt
    let clean_name = backup_name
        .splitn(3, '_')
        .nth(2)
        .unwrap_or(backup_name);

    let mut target_dest: Option<PathBuf> = None;
    for item_cfg in config.items.values() {
        if let Ok(exp) = expand_home(&item_cfg.target) {
            if exp.file_name().and_then(|n| n.to_str()) == Some(clean_name) {
                target_dest = Some(exp);
                break;
            }
        }
    }

    let target_path = match target_dest {
        Some(dest) => dest,
        None => {
            let default_path = dirs::home_dir()
                .context("Could not determine home directory")?
                .join(clean_name);
            default_path
        }
    };

    println!("\nRestoring '{}' -> '{}'...", backup_name, target_path.display());

    // If target is currently a symlink, remove the symlink
    if target_path.is_symlink() {
        let _ = fs::remove_file(&target_path);
    }

    copy_preserving_permissions(backup_path, &target_path)?;

    println!("{} Restored {}", ui::badge_ok(), style(target_path.display()).bold().cyan());
    Ok(())
}
