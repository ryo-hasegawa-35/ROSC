mod cli;
mod commands;

use anyhow::{Result, anyhow};
use clap::Parser;

use crate::cli::Cli;

fn main() -> Result<()> {
    std::thread::Builder::new()
        .name("rosc-broker-main".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| -> Result<()> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async_main())
        })
        .map_err(|error| anyhow!("failed to spawn broker main thread: {error}"))?
        .join()
        .map_err(|payload| {
            if let Some(message) = payload.downcast_ref::<&str>() {
                anyhow!("broker main thread panicked: {message}")
            } else if let Some(message) = payload.downcast_ref::<String>() {
                anyhow!("broker main thread panicked: {message}")
            } else {
                anyhow!("broker main thread panicked")
            }
        })?
}

async fn async_main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(Box::new(cli.command)).await
}
