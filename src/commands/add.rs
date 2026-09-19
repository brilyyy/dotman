use crate::cli::AddArgs;
use crate::config::{DotConfig, ItemConfig, ItemType};
use crate::fs::{contract_home, copy_preserving_permissions, create_symlink, expand_home};
use crate::ui;
use anyhow::{bail, Context, Result};
use console::style;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub fn execute(args: AddArgs) -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let repo_root = manifest_path
        .parent()
        .context("Manifest path has no parent directory")?
        .to_path_buf();

    let raw_path_str = args.path.to_string_lossy().to_string();
    let trimmed_raw = raw_path_str.trim_end_matches('/');
    let source_path = expand_home(trimmed_raw)?;

    if !source_path.exists() {
        bail!("Source path does not exist: '{}'", source_path.display());
    }

    let meta = fs::symlink_metadata(&source_path)
        .with_context(|| format!("Failed to read metadata for '{}'", source_path.display()))?;
    if meta.file_type().is_symlink() {
        bail!(
            "'{}' is already a symlink. Only real files or directories can be added.",
            source_path.display()
        );
    }

    let item_type = if source_path.is_dir() {
        ItemType::Folder
    } else {
        ItemType::File
    };

    let rel_repo_path = match args.name {
        Some(name) => PathBuf::from(name),
        None => {
            let file_name = source_path
                .file_name()
                .context("Invalid file name")?
                .to_string_lossy()
                .to_string();
            let clean_name = file_name.strip_prefix('.').unwrap_or(&file_name).to_string();
            PathBuf::from(clean_name)
        }
    };

    let repo_dest_path = repo_root.join(&rel_repo_path);
    if repo_dest_path.exists() {
        bail!(
            "Destination in repo already exists: '{}'. Specify a different name using --name.",
            repo_dest_path.display()
        );
    }

    if let Some(parent) = repo_dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if args.copy {
        info!(
            from = %source_path.display(),
            to = %repo_dest_path.display(),
            "Copying target to repository (--copy mode)"
        );

        copy_preserving_permissions(&source_path, &repo_dest_path)?;

        let rollback = || {
            if repo_dest_path.is_dir() {
                let _ = fs::remove_dir_all(&repo_dest_path);
            } else {
                let _ = fs::remove_file(&repo_dest_path);
            }
        };

        let mut config = match DotConfig::load_from_path(&manifest_path) {
            Ok(c) => c,
            Err(err) => {
                rollback();
                return Err(err.context("Failed to load dot.toml. Rolled back copied file."));
            }
        };

        let target_contracted = contract_home(&source_path);
        let key = rel_repo_path.to_string_lossy().to_string();

        config.items.insert(
            key.clone(),
            ItemConfig {
                target: target_contracted.clone(),
                item_type,
                method: crate::config::DeployMethod::Copy,
                tags: args.tags.clone(),
                post_deploy: None,
            },
        );

        if let Err(err) = config.save_to_path(&manifest_path) {
            rollback();
            return Err(err.context("Failed to save dot.toml. Rolled back copied file."));
        }

        println!("{} Added {} {}", ui::badge_ok(), style(&key).bold().cyan(), style("(copy mode)").yellow());
        println!("  source    {}", style(source_path.display()).dim());
        println!("  repo      {}", style(repo_dest_path.display()).dim());
        println!("  target    {}", style(target_contracted).dim());
        println!("  mode      copy (regular file/folder preserved on system)");
        if !args.tags.is_empty() {
            println!("  tags      {}", style(args.tags.join(", ")).cyan());
        }

        return Ok(());
    }

    info!(
        from = %source_path.display(),
        to = %repo_dest_path.display(),
        "Moving original target to repository"
    );

    // Step 1: Move file/directory into repo (with permission preservation)
    fs::rename(&source_path, &repo_dest_path).or_else(|_| {
        copy_preserving_permissions(&source_path, &repo_dest_path)?;
        if source_path.is_dir() {
            fs::remove_dir_all(&source_path)?;
        } else {
            fs::remove_file(&source_path)?;
        }
        Ok::<(), anyhow::Error>(())
    })?;

    // Rollback closure in case symlink creation or manifest save fails
    let rollback = || {
        if source_path.is_symlink() {
            let _ = fs::remove_file(&source_path);
        }
        if repo_dest_path.exists() {
            let _ = fs::rename(&repo_dest_path, &source_path).or_else(|_| {
                copy_preserving_permissions(&repo_dest_path, &source_path)?;
                if repo_dest_path.is_dir() {
                    let _ = fs::remove_dir_all(&repo_dest_path);
                } else {
                    let _ = fs::remove_file(&repo_dest_path);
                }
                Ok::<(), anyhow::Error>(())
            });
        }
    };

    // Step 2: Create symlink
    if let Err(err) = create_symlink(&repo_dest_path, &source_path) {
        rollback();
        return Err(err.context("Failed to create symlink. Rolled back original file."));
    }

    info!(
        target = %source_path.display(),
        source = %repo_dest_path.display(),
        "Created symlink"
    );

    // Step 3: Update manifest
    let mut config = match DotConfig::load_from_path(&manifest_path) {
        Ok(c) => c,
        Err(err) => {
            rollback();
            return Err(err.context("Failed to load dot.toml. Rolled back original file."));
        }
    };

    let target_contracted = contract_home(&source_path);
    let key = rel_repo_path.to_string_lossy().to_string();

    config.items.insert(
        key.clone(),
        ItemConfig {
            target: target_contracted.clone(),
            item_type,
            method: crate::config::DeployMethod::Symlink,
            tags: args.tags.clone(),
            post_deploy: None,
        },
    );

    if let Err(err) = config.save_to_path(&manifest_path) {
        rollback();
        return Err(err.context("Failed to save dot.toml. Rolled back original file."));
    }

    println!("{} Added {}", ui::badge_ok(), style(&key).bold().cyan());
    println!("  source    {}", style(source_path.display()).dim());
    println!("  repo      {}", style(repo_dest_path.display()).dim());
    println!("  target    {}", style(&target_contracted).dim());
    if !args.tags.is_empty() {
        println!("  tags      {}", style(args.tags.join(", ")).cyan());
    }

    Ok(())
}
