use clap::{Parser, Subcommand};
use small::{check, clean, init, install, run};

#[derive(Parser)]
#[command(name = "small")]
#[command(about = "Project bootstrap orchestrator — get any project running in one command")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan project files + interactive prompts → generate small.yaml
    Init,
    /// One-shot install: detect → download runtimes → install deps → test
    Install,
    /// Dry-run check — report what would be installed
    Check,
    /// Remove .small_venv / node_modules (preserves runtime cache)
    Clean,
    /// Launch the project via entrypoint
    Run,
    /// Print version
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init::run()?,
        Commands::Install => install::run()?,
        Commands::Check => check::run()?,
        Commands::Clean => clean::run()?,
        Commands::Run => run::run()?,
        Commands::Version => println!("small {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}
