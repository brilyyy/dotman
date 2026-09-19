use clap::Parser;
use console::style;
use dotman::cli::{Cli, Commands};
use dotman::commands;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn setup_logging(verbose: u8) {
    let default_directive = match verbose {
        0 => "dotman=warn",
        1 => "dotman=info",
        2 => "dotman=debug",
        _ => "dotman=trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_directive));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

fn main() {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    let result = match cli.command {
        Commands::Init => commands::init::execute(),
        Commands::Banner(args) => commands::banner::execute(args),
        Commands::Add(args) => commands::add::execute(args),
        Commands::Remove(args) => commands::remove::execute(args),
        Commands::Deploy(args) => commands::deploy::execute(args),
        Commands::Restore(args) => commands::restore::execute(args),
        Commands::Status => commands::status::execute(),
        Commands::InstallDeps(args) => commands::install_deps::execute(args),
        Commands::Completions(args) => commands::completions::execute(args),
    };

    if let Err(err) = result {
        eprintln!("\n{} {:#}", style("error:").red().bold(), err);
        std::process::exit(1);
    }
}
