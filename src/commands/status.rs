use crate::config::{DeployMethod, DotConfig};
use crate::fs::{
    check_copy_status, check_symlink_status, contract_home, expand_home, CopyStatus, SymlinkStatus,
};
use crate::ui;
use anyhow::{Context, Result};
use console::style;

pub fn execute() -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let repo_root = manifest_path
        .parent()
        .context("Manifest path has no parent directory")?
        .to_path_buf();
    let config = DotConfig::load_from_path(&manifest_path)?;

    let repo_display = repo_root.display().to_string();
    ui::print_header("dotman status", Some(&repo_display));
    println!();

    let mut valid = 0;
    let mut conflict = 0;
    let mut broken = 0;
    let mut missing = 0;

    for (key, item) in &config.items {
        let repo_source = repo_root.join(key);
        let target = expand_home(&item.target)?;
        let display_target = contract_home(&target);

        match item.method {
            DeployMethod::Copy => {
                let status = check_copy_status(&repo_source, &target);
                match status {
                    CopyStatus::InSync => {
                        valid += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_ok(),
                            style(key).bold(),
                            style(display_target).dim(),
                            style("(copy, in sync)").cyan().dim()
                        );
                    }
                    CopyStatus::Modified(reason) => {
                        conflict += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_warn(),
                            style(key).yellow(),
                            style(display_target).dim(),
                            style(format!("(copy modified: {})", reason)).yellow()
                        );
                    }
                    CopyStatus::Missing => {
                        missing += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_dim(),
                            key,
                            style(display_target).dim(),
                            style("(copy, not deployed)").dim()
                        );
                    }
                    CopyStatus::SymlinkConflict => {
                        conflict += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_warn(),
                            style(key).yellow(),
                            style(display_target).dim(),
                            style("(conflict: is a symlink, expected copy)").yellow()
                        );
                    }
                    CopyStatus::Conflict(reason) => {
                        conflict += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_warn(),
                            style(key).yellow(),
                            style(display_target).dim(),
                            style(format!("({})", reason)).yellow()
                        );
                    }
                }
            }
            DeployMethod::Symlink => {
                let status = check_symlink_status(&repo_source, &target);
                match status {
                    SymlinkStatus::Valid => {
                        valid += 1;
                        println!("  {} {:<16} -> {}", ui::badge_ok(), style(key).bold(), style(display_target).dim());
                    }
                    SymlinkStatus::Missing => {
                        missing += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_dim(),
                            key,
                            style(display_target).dim(),
                            style("(not deployed)").dim()
                        );
                    }
                    SymlinkStatus::Broken(reason) => {
                        broken += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_err(),
                            style(key).red(),
                            style(display_target).dim(),
                            style(format!("({})", reason)).red()
                        );
                    }
                    SymlinkStatus::Conflict(reason) => {
                        conflict += 1;
                        println!(
                            "  {} {:<16} -> {} {}",
                            ui::badge_warn(),
                            style(key).yellow(),
                            style(display_target).dim(),
                            style(format!("({})", reason)).yellow()
                        );
                    }
                }
            }
        }
    }

    println!();
    println!(
        "Summary: {} ok · {} conflict · {} broken · {} not deployed",
        if valid > 0 { style(valid).green().bold() } else { style(valid).dim() },
        if conflict > 0 { style(conflict).yellow().bold() } else { style(conflict).dim() },
        if broken > 0 { style(broken).red().bold() } else { style(broken).dim() },
        style(missing).dim()
    );

    Ok(())
}
