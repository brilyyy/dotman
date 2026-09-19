use crate::cli::DeployArgs;
use crate::config::{DeployMethod, DotConfig};
use crate::fs::{
    check_copy_status, check_symlink_status, compute_diff, copy_preserving_permissions,
    create_symlink, expand_home, CopyStatus, SymlinkStatus,
};
use crate::hooks::{run_hook, run_hooks};
use crate::safety::{prompt_conflict_action, quarantine, ConflictAction};
use crate::ui;
use anyhow::{Context, Result};
use console::style;
use std::fs;
use tracing::{debug, info};

pub fn execute(args: DeployArgs) -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let repo_root = manifest_path
        .parent()
        .context("Manifest path has no parent directory")?
        .to_path_buf();
    let config = DotConfig::load_from_path(&manifest_path)?;

    let backup_dir = repo_root.join(&config.settings.backup_dir);

    let repo_display = repo_root.display().to_string();
    ui::print_header("Deploying dotfiles", Some(&repo_display));

    if let Some(ref tag) = args.tag {
        println!("  filter: {}", style(format!("tag={}", tag)).yellow());
    }
    if args.copy {
        println!("  mode:   {}", style("copy (deploying regular files/directories)").cyan().bold());
    }
    if args.dry_run {
        println!("  mode:   {}", style("dry-run (no changes will be made)").yellow().bold());
    }
    println!();

    let mut deployed_count = 0;
    let mut already_linked_count = 0;
    let mut skipped_count = 0;

    for (key, item) in &config.items {
        if let Some(ref tag) = args.tag {
            if !item.tags.contains(tag) {
                continue;
            }
        }

        let repo_source = repo_root.join(key);
        let target = expand_home(&item.target)?;

        if !repo_source.exists() {
            debug!(
                key = %key,
                source = %repo_source.display(),
                "Repo source does not exist, skipping"
            );
            println!("  {} {:<16} source missing in repo ({})", ui::badge_warn(), style(key).bold(), style(repo_source.display()).dim());
            skipped_count += 1;
            continue;
        }

        let effective_method = if args.copy {
            DeployMethod::Copy
        } else {
            item.method
        };

        match effective_method {
            DeployMethod::Copy => {
                let status = check_copy_status(&repo_source, &target);

                match status {
                    CopyStatus::InSync => {
                        info!(key = %key, target = %target.display(), "Copy already in sync");
                        println!("  {} {:<16} already copied (in sync) -> {}", ui::badge_ok(), key, style(target.display()).dim());
                        already_linked_count += 1;
                    }
                    CopyStatus::Missing => {
                        if args.dry_run {
                            println!("  {} {:<16} [dry-run] would copy -> {}", ui::badge_add(), key, style(target.display()).dim());
                        } else {
                            copy_preserving_permissions(&repo_source, &target)?;
                            info!(key = %key, target = %target.display(), "Copied file/directory");
                            println!("  {} {:<16} copied -> {}", ui::badge_add(), style(key).cyan(), style(target.display()).dim());
                            deployed_count += 1;

                            if let Some(ref hook) = item.post_deploy {
                                run_hook(hook)?;
                            }
                        }
                    }
                    other_status => {
                        let reason = match other_status {
                            CopyStatus::Modified(ref r) => r.as_str(),
                            CopyStatus::SymlinkConflict => "target is currently a symlink, expected copy",
                            CopyStatus::Conflict(ref r) => r.as_str(),
                            _ => "conflict",
                        };

                        if args.dry_run {
                            println!(
                                "  {} {:<16} [dry-run] would overwrite copy -> {} ({})",
                                ui::badge_fix(),
                                key,
                                style(target.display()).dim(),
                                reason
                            );
                            continue;
                        }

                        loop {
                            let action = if args.force {
                                ConflictAction::Overwrite
                            } else {
                                prompt_conflict_action(key, &target)?
                            };

                            match action {
                                ConflictAction::Overwrite => {
                                    if config.settings.backup_enabled {
                                        let bak = quarantine(&target, &backup_dir)?;
                                        println!("  {} Quarantined target to {}", ui::badge_dim(), style(bak.display()).dim());
                                    } else {
                                        if target.is_symlink() || target.is_file() {
                                            let _ = fs::remove_file(&target);
                                        } else if target.is_dir() {
                                            let _ = fs::remove_dir_all(&target);
                                        }
                                    }
                                    copy_preserving_permissions(&repo_source, &target)?;
                                    println!("  {} {:<16} updated copy -> {}", ui::badge_fix(), style(key).cyan(), style(target.display()).dim());
                                    deployed_count += 1;

                                    if let Some(ref hook) = item.post_deploy {
                                        run_hook(hook)?;
                                    }
                                    break;
                                }
                                ConflictAction::Keep => {
                                    println!("  {} {:<16} kept existing target, skipped", ui::badge_dim(), key);
                                    skipped_count += 1;
                                    break;
                                }
                                ConflictAction::Diff => {
                                    println!("\n{}", style("--- Unified Diff (Repo vs System) ---").dim());
                                    match compute_diff(&repo_source, &target) {
                                        Ok(diff) if !diff.trim().is_empty() => println!("{}", diff),
                                        Ok(_) => println!("(Contents are identical)"),
                                        Err(e) => println!("Error generating diff: {}", e),
                                    }
                                    println!("{}\n", style("-------------------------------------").dim());
                                }
                            }
                        }
                    }
                }
            }
            DeployMethod::Symlink => {
                let status = check_symlink_status(&repo_source, &target);

                match status {
                    SymlinkStatus::Valid => {
                        info!(key = %key, target = %target.display(), "Symlink already valid");
                        println!("  {} {:<16} already linked -> {}", ui::badge_ok(), key, style(target.display()).dim());
                        already_linked_count += 1;
                    }
                    SymlinkStatus::Missing => {
                        if args.dry_run {
                            println!("  {} {:<16} [dry-run] would link -> {}", ui::badge_add(), key, style(target.display()).dim());
                        } else {
                            create_symlink(&repo_source, &target)?;
                            info!(key = %key, target = %target.display(), "Created symlink");
                            println!("  {} {:<16} linked -> {}", ui::badge_add(), style(key).cyan(), style(target.display()).dim());
                            deployed_count += 1;

                            if let Some(ref hook) = item.post_deploy {
                                run_hook(hook)?;
                            }
                        }
                    }
                    SymlinkStatus::Broken(ref reason) => {
                        if args.dry_run {
                            println!("  {} {:<16} [dry-run] would repair broken link -> {}", ui::badge_fix(), key, style(target.display()).dim());
                        } else {
                            let _ = fs::remove_file(&target);
                            create_symlink(&repo_source, &target)?;
                            info!(key = %key, target = %target.display(), reason = %reason, "Repaired broken symlink");
                            println!("  {} {:<16} repaired broken link -> {}", ui::badge_fix(), style(key).magenta(), style(target.display()).dim());
                            deployed_count += 1;

                            if let Some(ref hook) = item.post_deploy {
                                run_hook(hook)?;
                            }
                        }
                    }
                    SymlinkStatus::Conflict(ref reason) => {
                        if args.dry_run {
                            println!("  {} {:<16} [dry-run] conflict: {}", ui::badge_warn(), key, reason);
                            continue;
                        }

                        loop {
                            let action = if args.force {
                                ConflictAction::Overwrite
                            } else {
                                prompt_conflict_action(key, &target)?
                            };

                            match action {
                                ConflictAction::Overwrite => {
                                    if config.settings.backup_enabled {
                                        let bak = quarantine(&target, &backup_dir)?;
                                        println!("  {} Quarantined target to {}", ui::badge_dim(), style(bak.display()).dim());
                                    } else {
                                        if target.is_dir() {
                                            fs::remove_dir_all(&target)?;
                                        } else {
                                            fs::remove_file(&target)?;
                                        }
                                    }
                                    create_symlink(&repo_source, &target)?;
                                    println!("  {} {:<16} replaced with symlink -> {}", ui::badge_fix(), style(key).cyan(), style(target.display()).dim());
                                    deployed_count += 1;

                                    if let Some(ref hook) = item.post_deploy {
                                        run_hook(hook)?;
                                    }
                                    break;
                                }
                                ConflictAction::Keep => {
                                    println!("  {} {:<16} kept existing target, skipped", ui::badge_dim(), key);
                                    skipped_count += 1;
                                    break;
                                }
                                ConflictAction::Diff => {
                                    println!("\n{}", style("--- Unified Diff (Repo vs System) ---").dim());
                                    match compute_diff(&repo_source, &target) {
                                        Ok(diff) if !diff.trim().is_empty() => println!("{}", diff),
                                        Ok(_) => println!("(Contents are identical)"),
                                        Err(e) => println!("Error generating diff: {}", e),
                                    }
                                    println!("{}\n", style("-------------------------------------").dim());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Run global post-deploy hooks if any were deployed or if hooks exist
    if !args.dry_run && !config.hooks.post_deploy.is_empty() {
        println!();
        run_hooks(&config.hooks.post_deploy)?;
    }

    println!();
    println!(
        "Summary: {} deployed · {} already linked · {} skipped",
        style(deployed_count).bold(),
        style(already_linked_count).dim(),
        style(skipped_count).dim()
    );

    Ok(())
}
