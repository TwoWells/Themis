// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;

use themis::adapters::command::RealCommandExecutor;
use themis::adapters::dryrun::{DryRunCommandExecutor, DryRunFileSystem};
use themis::adapters::filesystem::RealFileSystem;
use themis::adapters::template::TeraAdapter;
use themis::core::orchestrator::{Orchestrator, SYSTEM_DATA_DIR};
use themis::core::state::State;

#[derive(Parser)]
#[command(name = "themis")]
#[command(about = "The General Contractor for your Linux Desktop Theme")]
#[command(version)]
struct Cli {
    /// Path to config directory (defaults to ~/.config/themis)
    #[arg(long, short, env = "THEMIS_CONFIG_DIR")]
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

    /// Check app configurations for proper setup
    Doctor,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
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
        ProjectDirs::from("com", "themis", "themis")
            .context("Could not determine home directory")?
            .config_dir()
            .to_path_buf()
    };

    // Hack: 'directories' crate uses 'com.themis.themis' -> ~/.config/themis on Linux usually
    // But verify.
    debug!("Config dir: {:?}", config_dir);

    // 3. Run Command
    match cli.command {
        Commands::Load { profile, dry_run } => {
            let result = if dry_run {
                info!("Running in dry-run mode");
                let orchestrator = Orchestrator::new(
                    DryRunFileSystem,
                    TeraAdapter::new(),
                    DryRunCommandExecutor,
                    config_dir,
                );
                orchestrator.load_profile(&profile)?
                // Don't save state in dry-run mode
            } else {
                let orchestrator = Orchestrator::new(
                    RealFileSystem,
                    TeraAdapter::new(),
                    RealCommandExecutor,
                    config_dir,
                );
                let result = orchestrator.load_profile(&profile)?;

                // Save state after load (even with partial failures)
                let state = State::new(profile);
                state.save()?;

                result
            };

            // Exit with error code if any apps failed
            if !result.is_ok() {
                std::process::exit(1);
            }
        }
        Commands::Status => {
            themis::commands::status::run()?;
        }
        Commands::Init => {
            themis::commands::init::run(&config_dir)?;
        }
        Commands::Verify => {
            let system_dir = PathBuf::from(SYSTEM_DATA_DIR);
            let result = themis::commands::verify::run(&config_dir, &system_dir)?;
            if !result.is_ok() {
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            let result = themis::commands::doctor::run(&config_dir)?;
            if !result.is_healthy() {
                std::process::exit(1);
            }
        }
        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "themis", &mut io::stdout());
        }
    }

    Ok(())
}
