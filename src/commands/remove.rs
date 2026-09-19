use crate::cli::RemoveArgs;
use crate::config::DotConfig;
use crate::fs::{copy_preserving_permissions, expand_home};
use crate::ui;
use anyhow::{bail, Context, Result};
use console::style;
use std::fs;
use tracing::info;

pub fn execute(args: RemoveArgs) -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let repo_root = manifest_path
        .parent()
        .context("Manifest path has no parent directory")?
        .to_path_buf();
    let mut config = DotConfig::load_from_path(&manifest_path)?;

    let item_cfg = match config.items.shift_remove(&args.item) {
        Some(item) => item,
        None => {
            let available: Vec<&String> = config.items.keys().collect();
            bail!(
                "Item '{}' is not tracked in dot.toml. Tracked items: {:?}",
                args.item,
                available
            );
        }
    };

    let repo_source = repo_root.join(&args.item);
    let target = expand_home(&item_cfg.target)?;

    if args.purge {
        info!(key = %args.item, "Purging item completely from repo and system");
        if target.is_symlink() {
            let _ = fs::remove_file(&target);
        }
        if repo_source.exists() {
            if repo_source.is_dir() {
                fs::remove_dir_all(&repo_source)?;
            } else {
                fs::remove_file(&repo_source)?;
            }
        }
        config.save_to_path(&manifest_path)?;

        println!("{} Purged {}", ui::badge_fix(), style(&args.item).bold().red());
        println!("  Removed from repository and deleted system symlink: {}", style(target.display()).dim());
    } else {
        info!(key = %args.item, "Demigrating item back to system target");
        // Remove symlink if it currently exists at target
        if target.is_symlink() {
            fs::remove_file(&target).with_context(|| {
                format!("Failed to remove symlink at '{}'", target.display())
            })?;
        }

        // Restore real file/directory back from repo to target
        if repo_source.exists() {
            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::rename(&repo_source, &target).or_else(|_| {
                copy_preserving_permissions(&repo_source, &target)?;
                if repo_source.is_dir() {
                    fs::remove_dir_all(&repo_source)?;
                } else {
                    fs::remove_file(&repo_source)?;
                }
                Ok::<(), anyhow::Error>(())
            })?;
        }

        config.save_to_path(&manifest_path)?;

        println!("{} Demigrated {}", ui::badge_ok(), style(&args.item).bold().cyan());
        println!("  Restored real file: {}", style(target.display()).dim());
        println!("  Removed from dot.toml");
    }

    Ok(())
}
