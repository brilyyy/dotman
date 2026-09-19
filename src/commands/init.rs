use crate::config::{DotConfig, DEFAULT_BACKUP_DIR, DEFAULT_MANIFEST_NAME};
use crate::ui;
use anyhow::{Context, Result};
use console::style;
use std::fs;
use tracing::info;

pub fn execute() -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let manifest_path = current_dir.join(DEFAULT_MANIFEST_NAME);
    let backup_dir = current_dir.join(DEFAULT_BACKUP_DIR);

    if manifest_path.exists() {
        println!(
            "{} Manifest '{}' already exists in this directory.",
            ui::badge_warn(),
            DEFAULT_MANIFEST_NAME
        );
        info!(path = %manifest_path.display(), "Manifest already exists, skipping initialization");
        return Ok(());
    }

    let default_config = DotConfig::default_template();
    default_config.save_to_path(&manifest_path)?;

    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("Failed to create backup directory '{}'", backup_dir.display()))?;
    }

    info!(manifest = %manifest_path.display(), backup = %backup_dir.display(), "Initialized dotman repository");

    println!("Initialized dotman repository at {}", style(current_dir.display()).bold());
    println!("  {} Created manifest: {}", ui::badge_add(), style(DEFAULT_MANIFEST_NAME).cyan());
    println!("  {} Created backup quarantine: {}", ui::badge_add(), style(format!("{}/", DEFAULT_BACKUP_DIR)).dim());
    println!("\nRun 'dotman add <path>' to start managing configurations.");

    Ok(())
}
