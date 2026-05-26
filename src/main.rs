use clap::{Parser, Subcommand};
use small::{check, clean, init, install, run, test};

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
    Init {
        /// Accept all defaults (for CI/automation)
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// One-shot install: detect → download runtimes → install deps → test
    Install {
        /// Skip confirmation prompt (for CI/automation)
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Dry-run check — report what would be installed
    Check,
    /// Remove .small_venv / node_modules (preserves runtime cache)
    Clean,
    /// Launch the project via entrypoint
    Run,
    /// Run the test command from small.yaml
    Test,
    /// Print version
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { yes } => init::run(yes)?,
        Commands::Install { yes } => install::run(yes)?,
        Commands::Check => check::run()?,
        Commands::Clean => clean::run()?,
        Commands::Run => run::run()?,
        Commands::Test => test::run()?,
        Commands::Version => println!("small {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}
