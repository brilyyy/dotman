use anyhow::{Context, Result};
use console::style;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManager {
    Pacman,
    Paru,
    Yay,
    AptGet,
    Apt,
    Nala,
    Dnf,
    Brew,
    Zypper,
    Apk,
    Xbps,
    Nix,
    Emerge,
    Port,
    Pkg,
    Cargo,
    Pipx,
    Pip,
    Npm,
    Pnpm,
    Bun,
    Flatpak,
    Snap,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Paru => "paru",
            Self::Yay => "yay",
            Self::AptGet => "apt-get",
            Self::Apt => "apt",
            Self::Nala => "nala",
            Self::Dnf => "dnf",
            Self::Brew => "brew",
            Self::Zypper => "zypper",
            Self::Apk => "apk",
            Self::Xbps => "xbps-install",
            Self::Nix => "nix-env",
            Self::Emerge => "emerge",
            Self::Port => "port",
            Self::Pkg => "pkg",
            Self::Cargo => "cargo",
            Self::Pipx => "pipx",
            Self::Pip => "pip",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Flatpak => "flatpak",
            Self::Snap => "snap",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "pacman" => Some(Self::Pacman),
            "paru" => Some(Self::Paru),
            "yay" => Some(Self::Yay),
            "apt-get" => Some(Self::AptGet),
            "apt" => Some(Self::Apt),
            "nala" => Some(Self::Nala),
            "dnf" => Some(Self::Dnf),
            "brew" | "homebrew" => Some(Self::Brew),
            "zypper" => Some(Self::Zypper),
            "apk" => Some(Self::Apk),
            "xbps" | "xbps-install" => Some(Self::Xbps),
            "nix" | "nix-env" => Some(Self::Nix),
            "emerge" => Some(Self::Emerge),
            "port" | "macports" => Some(Self::Port),
            "pkg" => Some(Self::Pkg),
            "cargo" => Some(Self::Cargo),
            "pipx" => Some(Self::Pipx),
            "pip" => Some(Self::Pip),
            "npm" => Some(Self::Npm),
            "pnpm" => Some(Self::Pnpm),
            "bun" => Some(Self::Bun),
            "flatpak" => Some(Self::Flatpak),
            "snap" => Some(Self::Snap),
            _ => None,
        }
    }

    pub fn build_install_command(&self, packages: &[String]) -> (String, Vec<String>) {
        let is_root = unsafe { libc::geteuid() == 0 };
        let use_sudo = !is_root && binary_exists("sudo");

        let wrap_privilege = |bin: &str, args: &[&str]| -> (String, Vec<String>) {
            let pkg_strings: Vec<String> = packages.to_vec();
            if use_sudo {
                (
                    "sudo".into(),
                    [
                        vec![bin.to_string()],
                        args.iter().map(|s| s.to_string()).collect(),
                        pkg_strings,
                    ]
                    .concat(),
                )
            } else {
                (
                    bin.to_string(),
                    [
                        args.iter().map(|s| s.to_string()).collect(),
                        pkg_strings,
                    ]
                    .concat(),
                )
            }
        };

        match self {
            Self::Paru => ("paru".into(), [vec!["-S".into(), "--needed".into()], packages.to_vec()].concat()),
            Self::Yay => ("yay".into(), [vec!["-S".into(), "--needed".into()], packages.to_vec()].concat()),
            Self::Brew => ("brew".into(), [vec!["install".into()], packages.to_vec()].concat()),
            Self::Cargo => ("cargo".into(), [vec!["install".into()], packages.to_vec()].concat()),
            Self::Pipx => ("pipx".into(), [vec!["install".into()], packages.to_vec()].concat()),
            Self::Pip => ("pip".into(), [vec!["install".into(), "--user".into()], packages.to_vec()].concat()),
            Self::Npm => ("npm".into(), [vec!["install".into(), "-g".into()], packages.to_vec()].concat()),
            Self::Pnpm => ("pnpm".into(), [vec!["add".into(), "-g".into()], packages.to_vec()].concat()),
            Self::Bun => ("bun".into(), [vec!["add".into(), "-g".into()], packages.to_vec()].concat()),
            Self::Nix => ("nix-env".into(), [vec!["-iA".into()], packages.to_vec()].concat()),
            Self::Flatpak => ("flatpak".into(), [vec!["install".into(), "-y".into()], packages.to_vec()].concat()),
            Self::Pacman => wrap_privilege("pacman", &["-S", "--needed", "--noconfirm"]),
            Self::Nala => wrap_privilege("nala", &["install", "-y"]),
            Self::AptGet => wrap_privilege("apt-get", &["install", "-y"]),
            Self::Apt => wrap_privilege("apt", &["install", "-y"]),
            Self::Dnf => wrap_privilege("dnf", &["install", "-y"]),
            Self::Zypper => wrap_privilege("zypper", &["install", "-y"]),
            Self::Apk => wrap_privilege("apk", &["add"]),
            Self::Xbps => wrap_privilege("xbps-install", &["-S", "-y"]),
            Self::Emerge => wrap_privilege("emerge", &["-av", "--noreplace"]),
            Self::Port => wrap_privilege("port", &["install"]),
            Self::Pkg => wrap_privilege("pkg", &["install", "-y"]),
            Self::Snap => wrap_privilege("snap", &["install"]),
        }
    }
}

pub fn binary_exists(bin: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(bin).is_file() {
                return true;
            }
        }
    }
    false
}

pub fn detect_package_manager() -> Option<PackageManager> {
    // 1. Arch AUR helpers & pacman
    if binary_exists("paru") {
        return Some(PackageManager::Paru);
    }
    if binary_exists("yay") {
        return Some(PackageManager::Yay);
    }
    if binary_exists("pacman") {
        return Some(PackageManager::Pacman);
    }

    // 2. Debian / Ubuntu family
    if binary_exists("nala") {
        return Some(PackageManager::Nala);
    }
    if binary_exists("apt") {
        return Some(PackageManager::Apt);
    }
    if binary_exists("apt-get") {
        return Some(PackageManager::AptGet);
    }

    // 3. RedHat / Fedora
    if binary_exists("dnf") {
        return Some(PackageManager::Dnf);
    }

    // 4. macOS / Linuxbrew
    if binary_exists("brew") {
        return Some(PackageManager::Brew);
    }
    if binary_exists("port") {
        return Some(PackageManager::Port);
    }

    // 5. openSUSE
    if binary_exists("zypper") {
        return Some(PackageManager::Zypper);
    }

    // 6. Alpine Linux
    if binary_exists("apk") {
        return Some(PackageManager::Apk);
    }

    // 7. Void Linux
    if binary_exists("xbps-install") {
        return Some(PackageManager::Xbps);
    }

    // 8. Nix
    if binary_exists("nix-env") {
        return Some(PackageManager::Nix);
    }

    // 9. Gentoo
    if binary_exists("emerge") {
        return Some(PackageManager::Emerge);
    }

    // 10. FreeBSD / Termux
    if binary_exists("pkg") {
        return Some(PackageManager::Pkg);
    }

    None
}

pub fn run_install(pm: &PackageManager, packages: &[String], dry_run: bool) -> Result<()> {
    if packages.is_empty() {
        warn!("No packages specified for installation");
        return Ok(());
    }

    let (cmd_bin, args) = pm.build_install_command(packages);
    let full_cmd = format!("{} {}", cmd_bin, args.join(" "));

    if dry_run {
        info!("[Dry-run] Would execute: {}", full_cmd);
        println!("  {} Would execute: {}", style("dry-run").yellow().bold(), style(&full_cmd).dim());
        return Ok(());
    }

    info!(command = %full_cmd, "Executing package manager");
    println!("Executing: {}", style(&full_cmd).bold());

    let status = Command::new(&cmd_bin)
        .args(&args)
        .status()
        .with_context(|| format!("Failed to run package manager command '{}'", full_cmd))?;

    if !status.success() {
        anyhow::bail!("Package installation failed with exit code: {:?}", status.code());
    }

    println!("\nInstalled {} packages successfully.", packages.len());
    Ok(())
}

pub fn run_custom_command(cmd_template: &str, packages: &[String], dry_run: bool) -> Result<()> {
    let rendered = if cmd_template.contains("{packages}") {
        cmd_template.replace("{packages}", &packages.join(" "))
    } else if packages.is_empty() {
        cmd_template.to_string()
    } else {
        format!("{} {}", cmd_template, packages.join(" "))
    };

    if dry_run {
        info!("[Dry-run] Would execute custom command: {}", rendered);
        println!("  {} Would execute: {}", style("dry-run").yellow().bold(), style(&rendered).dim());
        return Ok(());
    }

    info!(command = %rendered, "Executing custom install command");
    println!("Executing: {}", style(&rendered).bold());

    let status = Command::new("sh")
        .arg("-c")
        .arg(&rendered)
        .status()
        .with_context(|| format!("Failed to run custom command '{}'", rendered))?;

    if !status.success() {
        anyhow::bail!("Custom command failed with exit code: {:?}", status.code());
    }

    if !packages.is_empty() {
        println!("\nInstalled {} packages successfully.", packages.len());
    }
    Ok(())
}

pub fn run_custom_script(
    script_path: &Path,
    packages: &[String],
    category: &str,
    manifest_path: &Path,
    dry_run: bool,
) -> Result<()> {
    if !script_path.exists() {
        anyhow::bail!("Custom installer script not found: '{}'", script_path.display());
    }

    let pkgs_joined = packages.join(" ");
    let display_cmd = if packages.is_empty() {
        format!("sh {}", script_path.display())
    } else {
        format!("sh {} {}", script_path.display(), pkgs_joined)
    };

    if dry_run {
        info!("[Dry-run] Would execute custom script: {}", display_cmd);
        println!("  {} Would execute: {}", style("dry-run").yellow().bold(), style(&display_cmd).dim());
        return Ok(());
    }

    info!(script = %script_path.display(), category = %category, "Executing custom installer script");
    println!("Executing script: {}", style(&display_cmd).bold());

    let mut cmd = Command::new("sh");
    cmd.arg(script_path);
    cmd.args(packages);
    cmd.env("DOTMAN_PACKAGES", &pkgs_joined);
    cmd.env("DOTMAN_CATEGORY", category);
    cmd.env("DOTMAN_MANIFEST", manifest_path.as_os_str());
    cmd.env("DOTMAN_DRY_RUN", if dry_run { "1" } else { "0" });

    let status = cmd
        .status()
        .with_context(|| format!("Failed to run script '{}'", script_path.display()))?;

    if !status.success() {
        anyhow::bail!("Custom installer script failed with exit code: {:?}", status.code());
    }

    println!("\nScript '{}' completed successfully.", script_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_parsing() {
        assert_eq!(PackageManager::from_name("pacman"), Some(PackageManager::Pacman));
        assert_eq!(PackageManager::from_name("brew"), Some(PackageManager::Brew));
        assert_eq!(PackageManager::from_name("cargo"), Some(PackageManager::Cargo));
        assert_eq!(PackageManager::from_name("xbps"), Some(PackageManager::Xbps));
        assert_eq!(PackageManager::from_name("unknown_xyz"), None);
    }

    #[test]
    fn test_cargo_and_brew_commands() {
        let pkgs = vec!["ripgrep".into(), "bat".into()];
        let (bin, args) = PackageManager::Cargo.build_install_command(&pkgs);
        assert_eq!(bin, "cargo");
        assert_eq!(args, vec!["install", "ripgrep", "bat"]);

        let (bin, args) = PackageManager::Brew.build_install_command(&pkgs);
        assert_eq!(bin, "brew");
        assert_eq!(args, vec!["install", "ripgrep", "bat"]);
    }

    #[test]
    fn test_custom_command_dry_run() {
        let pkgs = vec!["tool1".into(), "tool2".into()];
        let res = run_custom_command("my-installer --install {packages}", &pkgs, true);
        assert!(res.is_ok());

        let res_no_token = run_custom_command("my-installer", &pkgs, true);
        assert!(res_no_token.is_ok());
    }
}
