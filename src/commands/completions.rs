use crate::cli::{Cli, CompletionsArgs};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

pub fn execute(args: CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "dotman", &mut std::io::stdout());
    Ok(())
}
