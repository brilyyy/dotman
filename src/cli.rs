use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "dotman",
    author,
    version,
    about = "Fast, safe, and transparent dotfile manager in Rust"
)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true, help = "Increase logging verbosity (-v, -vv)")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Initialize the current directory as a dotfile repository")]
    Init,

    #[command(about = "Add a file or directory to dotman management")]
    Add(AddArgs),

    #[command(about = "Remove/demigrate an item from dotman management")]
    Remove(RemoveArgs),

    #[command(about = "Deploy managed dotfiles by linking them to target system paths")]
    Deploy(DeployArgs),

    #[command(about = "Restore a quarantined backup from .bak/")]
    Restore(RestoreArgs),

    #[command(about = "Check status of all managed dotfile symlinks")]
    Status,

    #[command(name = "install-deps", about = "Install declarative system dependencies")]
    InstallDeps(InstallDepsArgs),

    #[command(about = "Generate shell completions script")]
    Completions(CompletionsArgs),
}

#[derive(Debug, Args)]
pub struct AddArgs {
    #[arg(help = "Path to the file or directory to add")]
    pub path: PathBuf,

    #[arg(short, long, help = "Custom relative destination path inside the dotfile repository")]
    pub name: Option<String>,

    #[arg(short, long = "tag", help = "Tags to associate with this item (e.g. -t dev)")]
    pub tags: Vec<String>,

    #[arg(long, help = "Manage this item as a regular copy instead of a symlink")]
    pub copy: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    #[arg(help = "Key name of the item to remove (e.g. zshrc, nvim)")]
    pub item: String,

    #[arg(long, help = "Completely delete the item from both repo and system")]
    pub purge: bool,
}

#[derive(Debug, Args)]
pub struct RestoreArgs {
    #[arg(help = "Optional filter name of the backup to restore")]
    pub item: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeployArgs {
    #[arg(short, long, help = "Filter items to deploy by tag")]
    pub tag: Option<String>,

    #[arg(long, help = "Preview deployment changes without modifying filesystem")]
    pub dry_run: bool,

    #[arg(short, long, help = "Automatically overwrite and quarantine conflicting targets without interactive prompt")]
    pub force: bool,

    #[arg(long, help = "Deploy items as regular copies instead of symlinks")]
    pub copy: bool,
}

#[derive(Debug, Args)]
pub struct InstallDepsArgs {
    #[arg(short, long, help = "Specific dependency category to install (e.g. core, rust_tools)")]
    pub category: Option<String>,

    #[arg(short, long, help = "Override package manager (e.g. brew, cargo, pacman, xbps)")]
    pub manager: Option<String>,

    #[arg(long, help = "Custom install command template (e.g. 'cargo binstall -y {packages}')")]
    pub cmd: Option<String>,

    #[arg(long, help = "Path to custom installer script to execute")]
    pub script: Option<PathBuf>,

    #[arg(long, help = "Preview package installation command without executing")]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(help = "Target shell to generate completions for (bash, zsh, fish, powershell, elvish)")]
    pub shell: Shell,
}
