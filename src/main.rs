use clap::{Parser, Subcommand};
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::{Context, Result};
use tracing::{info, debug, Level};
use tracing_subscriber::FmtSubscriber;

use theman::core::orchestrator::Orchestrator;
use theman::adapters::filesystem::RealFileSystem;
use theman::adapters::template::TeraAdapter;
use theman::adapters::command::RealCommandExecutor;

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
    },
    
    /// Show current status
    Status,
    
    /// Initialize configuration
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Setup Logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set up logging")?;

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

    // 3. Instantiate Components
    let fs = RealFileSystem;
    let tera = TeraAdapter::new();
    let cmd = RealCommandExecutor;
    
    let orchestrator = Orchestrator::new(fs, tera, cmd, config_dir);

    // 4. Run Command
    match cli.command {
        Commands::Load { profile } => {
            orchestrator.load_profile(&profile)?;
        }
        Commands::Status => {
            info!("Status command not implemented yet");
        }
        Commands::Init => {
            info!("Init command not implemented yet");
        }
    }

    Ok(())
}