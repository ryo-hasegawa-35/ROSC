use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;

pub(crate) async fn serve_health(listen: &str, config: Option<&Path>) -> Result<()> {
    let telemetry = InMemoryTelemetry::default();
    let mut manager = rosc_config::ConfigManager::default();

    if let Some(path) = config {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let applied = manager.apply_toml_str(&content)?;
        rosc_broker::emit_applied_config(&telemetry, &applied);
    }

    let mut health_service = rosc_broker::HealthService::spawn(listen, Arc::new(telemetry)).await?;
    println!(
        "health endpoint listening on {}",
        health_service.listen_addr()
    );
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for ctrl-c")?;
    health_service.shutdown().await?;
    println!("health endpoint stopped");
    Ok(())
}
