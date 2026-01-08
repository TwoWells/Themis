use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use theman::adapters::command::RealCommandExecutor;
use theman::adapters::dryrun::{DryRunCommandExecutor, DryRunFileSystem};
use theman::adapters::filesystem::RealFileSystem;
use theman::adapters::template::TeraAdapter;
use theman::core::orchestrator::{Orchestrator, SYSTEM_DATA_DIR};

#[derive(Parser)]
#[command(name = "theman")]
#[command(about = "The General Contractor for your Linux Desktop Theme")]
#[command(version)]
struct Cli {
    /// Path to config directory (defaults to ~/.config/theman)
    #[arg(long, short, env = "THEMAN_CONFIG_DIR")]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(long, short, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load a profile
    Load {
        /// The name of the profile to load (e.g., "nord")
        profile: String,

        /// Simulate actions without writing files or running commands
        #[arg(long)]
        dry_run: bool,
    },

    /// Show current status
    Status,

    /// Initialize configuration
    Init,

    /// Verify configuration is valid
    Verify,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Setup Logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).context("Failed to set up logging")?;

    // 2. Determine Config Dir
    let config_dir = if let Some(path) = cli.config {
        path
    } else {
        ProjectDirs::from("com", "theman", "theman")
            .context("Could not determine home directory")?
            .config_dir()
            .to_path_buf()
    };

    // Hack: 'directories' crate uses 'com.theman.theman' -> ~/.config/theman on Linux usually
    // But verify.
    debug!("Config dir: {:?}", config_dir);

    // 3. Run Command
    match cli.command {
        Commands::Load { profile, dry_run } => {
            if dry_run {
                info!("Running in dry-run mode");
                let orchestrator = Orchestrator::new(
                    DryRunFileSystem,
                    TeraAdapter::new(),
                    DryRunCommandExecutor,
                    config_dir,
                );
                orchestrator.load_profile(&profile)?;
            } else {
                let orchestrator = Orchestrator::new(
                    RealFileSystem,
                    TeraAdapter::new(),
                    RealCommandExecutor,
                    config_dir,
                );
                orchestrator.load_profile(&profile)?;
            }
        }
        Commands::Status => {
            info!("Status command not implemented yet");
        }
        Commands::Init => {
            theman::commands::init::run(&config_dir)?;
        }
        Commands::Verify => {
            let system_dir = PathBuf::from(SYSTEM_DATA_DIR);
            let result = theman::commands::verify::run(&config_dir, &system_dir)?;
            if !result.is_ok() {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
