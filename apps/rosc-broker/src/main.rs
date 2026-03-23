use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "ROSC broker bootstrap CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    CheckConfig {
        path: PathBuf,
    },
    DiffConfig {
        current: PathBuf,
        candidate: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::CheckConfig { path } => {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file {}", path.display()))?;
            let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
            println!(
                "valid config: schema_version={} route(s)={}",
                config.schema_version,
                config.routes.len()
            );
        }
        Command::DiffConfig { current, candidate } => {
            let current_content = fs::read_to_string(&current)
                .with_context(|| format!("failed to read config file {}", current.display()))?;
            let candidate_content = fs::read_to_string(&candidate)
                .with_context(|| format!("failed to read config file {}", candidate.display()))?;

            let mut manager = rosc_config::ConfigManager::default();
            let applied = manager.apply_toml_str(&current_content)?;
            let diff = manager.preview_toml_diff(&candidate_content)?;

            println!("current_revision={}", applied.revision);
            println!("added_routes={}", diff.added_routes.join(","));
            println!("removed_routes={}", diff.removed_routes.join(","));
            println!("changed_routes={}", diff.changed_routes.join(","));
        }
    }
    Ok(())
}
