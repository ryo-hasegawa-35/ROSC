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
    CheckConfig { path: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::CheckConfig { path } => {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file {}", path.display()))?;
            let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
            println!("valid config: {} route(s)", config.routes.len());
        }
    }
    Ok(())
}
