use crate::cli::InstallDepsArgs;
use crate::config::{DependencyGroup, DotConfig};
use crate::pm::{
    detect_package_manager, run_custom_command, run_custom_script, run_install, PackageManager,
};
use crate::ui;
use anyhow::{bail, Context, Result};
use console::style;
use std::path::{Path, PathBuf};

pub fn execute(args: InstallDepsArgs) -> Result<()> {
    let manifest_path = DotConfig::find_manifest()?;
    let config = DotConfig::load_from_path(&manifest_path)?;
    let repo_root = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    if config.dependencies.is_empty() && args.script.is_none() && args.cmd.is_none() {
        println!("No dependencies defined in '{}'.", manifest_path.display());
        return Ok(());
    }

    // Determine target categories
    let categories: Vec<(&String, &DependencyGroup)> = if let Some(cat) = &args.category {
        match config.dependencies.get(cat) {
            Some(group) => vec![(cat, group)],
            None => {
                let available: Vec<&String> = config.dependencies.keys().collect();
                bail!(
                    "Category '{}' not found. Available categories: {:?}",
                    cat,
                    available
                );
            }
        }
    } else {
        config.dependencies.iter().collect()
    };

    if categories.is_empty() {
        // Special case: CLI passed --script or --cmd without manifest dependencies
        if let Some(script_path) = &args.script {
            let full_script = resolve_script_path(script_path, repo_root);
            ui::print_header("Installing dependencies", Some("ad-hoc script"));
            println!("  method           script ({})", style(full_script.display()).dim());
            println!();
            return run_custom_script(&full_script, &[], "ad-hoc", &manifest_path, args.dry_run);
        } else if let Some(cmd_template) = &args.cmd {
            ui::print_header("Installing dependencies", Some("ad-hoc command"));
            println!("  method           custom ({})", style(cmd_template).dim());
            println!();
            return run_custom_command(cmd_template, &[], args.dry_run);
        }
        println!("No dependencies to install.");
        return Ok(());
    }

    for (cat_name, dep_group) in categories {
        let packages = dep_group.packages();

        // 1. Script execution check
        if let Some(cli_script) = &args.script {
            let full_script = resolve_script_path(cli_script, repo_root);
            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  method           script ({})", style(full_script.display()).dim());
            if !packages.is_empty() {
                println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            }
            println!();
            run_custom_script(&full_script, packages, cat_name, &manifest_path, args.dry_run)?;
            continue;
        } else if let Some(script_rel) = dep_group.script() {
            let full_script = resolve_script_path(Path::new(script_rel), repo_root);
            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  method           script ({})", style(full_script.display()).dim());
            if !packages.is_empty() {
                println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            }
            println!();
            run_custom_script(&full_script, packages, cat_name, &manifest_path, args.dry_run)?;
            continue;
        }

        // 2. Custom Command Template check
        if let Some(cli_cmd) = &args.cmd {
            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  method           custom ({})", style(cli_cmd).dim());
            if !packages.is_empty() {
                println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            }
            println!();
            run_custom_command(cli_cmd, packages, args.dry_run)?;
            continue;
        } else if let Some(grp_cmd) = dep_group.cmd() {
            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  method           custom ({})", style(grp_cmd).dim());
            if !packages.is_empty() {
                println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            }
            println!();
            run_custom_command(grp_cmd, packages, args.dry_run)?;
            continue;
        }

        // 3. Package Manager resolution
        let mgr_name_opt = args
            .manager
            .as_deref()
            .or_else(|| dep_group.manager())
            .or_else(|| config.settings.package_manager.as_deref());

        if packages.is_empty() {
            println!("No packages to install for category '{}'.", cat_name);
            continue;
        }

        if let Some(mgr_name) = mgr_name_opt {
            // Check if manager is a named installer template in [installers]
            if let Some(tmpl) = config.installers.get(mgr_name) {
                ui::print_header("Installing dependencies", Some(cat_name));
                println!("  method           installer alias '{}' ({})", style(mgr_name).cyan(), style(tmpl).dim());
                println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
                println!();
                run_custom_command(tmpl, packages, args.dry_run)?;
                continue;
            }

            // Check if manager is a known built-in PackageManager
            let pm = PackageManager::from_name(mgr_name).with_context(|| {
                format!(
                    "Unknown package manager or installer alias '{}' specified for category '{}'.",
                    mgr_name, cat_name
                )
            })?;

            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  package manager  {}", style(pm.name()).bold().cyan());
            println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            println!();
            run_install(&pm, packages, args.dry_run)?;
        } else {
            // Auto-detect package manager
            let pm = match detect_package_manager() {
                Some(pm) => pm,
                None => bail!(
                    "Could not auto-detect a supported package manager. Please specify one with '--manager <name>' or define 'manager = \"...\"' in dot.toml"
                ),
            };

            ui::print_header("Installing dependencies", Some(cat_name));
            println!("  package manager  {}", style(pm.name()).bold().cyan());
            println!("  packages ({})    {}", packages.len(), style(packages.join(", ")).dim());
            println!();
            run_install(&pm, packages, args.dry_run)?;
        }
    }

    Ok(())
}

fn resolve_script_path(path: &Path, repo_root: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
