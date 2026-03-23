use std::sync::Arc;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rosc_telemetry::{BrokerEvent, HealthReporter, InMemoryTelemetry, TelemetrySink};
use tokio::net::TcpListener;

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
    ProxyStatus {
        config: PathBuf,
        #[arg(long)]
        resolve_bindings: bool,
    },
    WatchConfig {
        path: PathBuf,
        #[arg(long, default_value_t = 1000)]
        poll_ms: u64,
    },
    DiffConfig {
        current: PathBuf,
        candidate: PathBuf,
    },
    ServeHealth {
        listen: String,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    RunUdpProxy {
        config: PathBuf,
        #[arg(long, default_value_t = 1024)]
        ingress_queue_depth: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
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
        Command::ProxyStatus {
            config,
            resolve_bindings,
        } => {
            let content = fs::read_to_string(&config)
                .with_context(|| format!("failed to read config file {}", config.display()))?;
            let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
            let status = if resolve_bindings {
                let app =
                    rosc_broker::UdpProxyApp::from_config(&config, InMemoryTelemetry::default())
                        .await?;
                app.status_snapshot()
            } else {
                rosc_broker::proxy_status_from_config(&config)?
            };
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Command::WatchConfig { path, poll_ms } => {
            let telemetry = InMemoryTelemetry::default();
            let mut supervisor = rosc_broker::ConfigFileSupervisor::new(&path, telemetry);
            let applied = supervisor.load_initial()?;
            println!(
                "loaded initial config: revision={} added_ingresses={} added_destinations={} added_routes={}",
                applied.revision,
                applied.diff.added_ingresses.join(","),
                applied.diff.added_destinations.join(","),
                applied.diff.added_routes.join(",")
            );

            let mut interval = tokio::time::interval(std::time::Duration::from_millis(poll_ms));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match supervisor.poll_once()? {
                            rosc_broker::ConfigReloadOutcome::Unchanged => {}
                            rosc_broker::ConfigReloadOutcome::Applied(applied) => {
                                println!(
                                    "applied config revision={} added_ingresses={} removed_ingresses={} changed_ingresses={} added_destinations={} removed_destinations={} changed_destinations={} added_routes={} removed_routes={} changed_routes={}",
                                    applied.revision,
                                    applied.diff.added_ingresses.join(","),
                                    applied.diff.removed_ingresses.join(","),
                                    applied.diff.changed_ingresses.join(","),
                                    applied.diff.added_destinations.join(","),
                                    applied.diff.removed_destinations.join(","),
                                    applied.diff.changed_destinations.join(","),
                                    applied.diff.added_routes.join(","),
                                    applied.diff.removed_routes.join(","),
                                    applied.diff.changed_routes.join(","),
                                );
                            }
                            rosc_broker::ConfigReloadOutcome::Rejected(error) => {
                                let revision = supervisor.current_revision().unwrap_or_default();
                                println!(
                                    "rejected config change; keeping revision={} reason={}",
                                    revision,
                                    error
                                );
                            }
                        }
                    }
                    result = tokio::signal::ctrl_c() => {
                        result.context("failed to listen for ctrl-c")?;
                        break;
                    }
                }
            }
            println!("config watcher stopped");
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
            println!("added_ingresses={}", diff.added_ingresses.join(","));
            println!("removed_ingresses={}", diff.removed_ingresses.join(","));
            println!("changed_ingresses={}", diff.changed_ingresses.join(","));
            println!("added_destinations={}", diff.added_destinations.join(","));
            println!(
                "removed_destinations={}",
                diff.removed_destinations.join(",")
            );
            println!(
                "changed_destinations={}",
                diff.changed_destinations.join(",")
            );
            println!("added_routes={}", diff.added_routes.join(","));
            println!("removed_routes={}", diff.removed_routes.join(","));
            println!("changed_routes={}", diff.changed_routes.join(","));
        }
        Command::ServeHealth { listen, config } => {
            let telemetry = InMemoryTelemetry::default();
            let mut manager = rosc_config::ConfigManager::default();

            if let Some(path) = config {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("failed to read config file {}", path.display()))?;
                let applied = manager.apply_toml_str(&content)?;
                telemetry.emit(BrokerEvent::ConfigApplied {
                    revision: applied.revision,
                    added_ingresses: applied.diff.added_ingresses.len(),
                    removed_ingresses: applied.diff.removed_ingresses.len(),
                    changed_ingresses: applied.diff.changed_ingresses.len(),
                    added_destinations: applied.diff.added_destinations.len(),
                    removed_destinations: applied.diff.removed_destinations.len(),
                    changed_destinations: applied.diff.changed_destinations.len(),
                    added_routes: applied.diff.added_routes.len(),
                    removed_routes: applied.diff.removed_routes.len(),
                    changed_routes: applied.diff.changed_routes.len(),
                });
            }

            let listener = TcpListener::bind(&listen)
                .await
                .with_context(|| format!("failed to bind health listener on {listen}"))?;
            println!("health endpoint listening on {}", listener.local_addr()?);

            let reporter: Arc<dyn HealthReporter> = Arc::new(telemetry);
            loop {
                tokio::select! {
                    result = rosc_runtime::serve_health_http_once(&listener, Arc::clone(&reporter)) => {
                        result?;
                    }
                    result = tokio::signal::ctrl_c() => {
                        result.context("failed to listen for ctrl-c")?;
                        break;
                    }
                }
            }
            println!("health endpoint stopped");
        }
        Command::RunUdpProxy {
            config,
            ingress_queue_depth,
        } => {
            let content = fs::read_to_string(&config)
                .with_context(|| format!("failed to read config file {}", config.display()))?;
            let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
            let telemetry = InMemoryTelemetry::default();
            let mut app = rosc_broker::UdpProxyApp::from_config(&config, telemetry).await?;
            app.spawn_ingress_tasks(ingress_queue_depth).await;
            println!("udp proxy running; press Ctrl-C to stop");
            tokio::signal::ctrl_c()
                .await
                .context("failed to listen for ctrl-c")?;
            println!("udp proxy stopped");
        }
    }
    Ok(())
}
