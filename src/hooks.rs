use anyhow::{Context, Result};
use console::style;
use std::process::Command;
use tracing::info;

pub fn run_hook(hook_cmd: &str) -> Result<()> {
    if hook_cmd.trim().is_empty() {
        return Ok(());
    }

    info!(command = %hook_cmd, "Running post_deploy hook");
    println!("  {} Running hook: {}", style("hook").magenta().bold(), style(hook_cmd).dim());

    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(hook_cmd)
        .status()
        .with_context(|| format!("Failed to execute hook '{}'", hook_cmd))?;

    if !status.success() {
        anyhow::bail!("Hook '{}' failed with exit code: {:?}", hook_cmd, status.code());
    }

    Ok(())
}

pub fn run_hooks(hook_cmds: &[String]) -> Result<()> {
    for cmd in hook_cmds {
        run_hook(cmd)?;
    }
    Ok(())
}
