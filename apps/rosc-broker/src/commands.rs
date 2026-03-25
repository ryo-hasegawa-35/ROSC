use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::{BrokerEvent, HealthReporter, InMemoryTelemetry, TelemetrySink};
use tokio::net::TcpListener;

use crate::cli::Command;

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::CheckConfig { path } => check_config(&path).await,
        Command::ProxyStatus {
            config,
            resolve_bindings,
        } => proxy_status(&config, resolve_bindings).await,
        Command::WatchConfig {
            path,
            poll_ms,
            fail_on_warnings,
            require_fallback_ready,
        } => watch_config(&path, poll_ms, fail_on_warnings, require_fallback_ready).await,
        Command::DiffConfig { current, candidate } => diff_config(&current, &candidate).await,
        Command::ServeHealth { listen, config } => serve_health(&listen, config.as_deref()).await,
        Command::RunUdpProxy {
            config,
            ingress_queue_depth,
            fail_on_warnings,
            require_fallback_ready,
        } => {
            run_udp_proxy(
                &config,
                ingress_queue_depth,
                fail_on_warnings,
                require_fallback_ready,
            )
            .await
        }
    }
}

async fn check_config(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
    println!(
        "valid config: schema_version={} route(s)={}",
        config.schema_version,
        config.routes.len()
    );
    Ok(())
}

async fn proxy_status(path: &Path, resolve_bindings: bool) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
    let status = if resolve_bindings {
        let app =
            rosc_broker::UdpProxyApp::from_config(&config, InMemoryTelemetry::default()).await?;
        app.status_snapshot()
    } else {
        rosc_broker::proxy_status_from_config(&config)?
    };
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn watch_config(
    path: &Path,
    poll_ms: u64,
    fail_on_warnings: bool,
    require_fallback_ready: bool,
) -> Result<()> {
    let telemetry = InMemoryTelemetry::default();
    let mut supervisor = rosc_broker::ConfigFileSupervisor::new(path, telemetry);
    let safety_policy = rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings,
        require_fallback_ready,
    };
    let applied = supervisor.load_initial_with_guard(|config| {
        rosc_broker::evaluate_proxy_runtime_policy(config, safety_policy)
    })?;
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
                match supervisor.poll_once_with_guard(|config| {
                    rosc_broker::evaluate_proxy_runtime_policy(config, safety_policy)
                })? {
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
                    rosc_broker::ConfigReloadOutcome::Blocked(reasons) => {
                        let revision = supervisor.current_revision().unwrap_or_default();
                        println!(
                            "blocked config change; keeping revision={} reasons={}",
                            revision,
                            reasons.join(" | ")
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
    Ok(())
}

async fn diff_config(current: &Path, candidate: &Path) -> Result<()> {
    let current_content = fs::read_to_string(current)
        .with_context(|| format!("failed to read config file {}", current.display()))?;
    let candidate_content = fs::read_to_string(candidate)
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
    Ok(())
}

async fn serve_health(listen: &str, config: Option<&Path>) -> Result<()> {
    let telemetry = InMemoryTelemetry::default();
    let mut manager = rosc_config::ConfigManager::default();

    if let Some(path) = config {
        let content = fs::read_to_string(path)
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

    let listener = TcpListener::bind(listen)
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
    Ok(())
}

async fn run_udp_proxy(
    path: &Path,
    ingress_queue_depth: usize,
    fail_on_warnings: bool,
    require_fallback_ready: bool,
) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
    let status = rosc_broker::proxy_status_from_config(&config)?;
    for line in rosc_broker::proxy_startup_report_lines(&status) {
        println!("{line}");
    }

    let safety_policy = rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings,
        require_fallback_ready,
    };
    let telemetry = InMemoryTelemetry::default();
    let mut proxy =
        rosc_broker::ManagedUdpProxy::start(config, telemetry, ingress_queue_depth, safety_policy)
            .await?;
    println!("udp proxy running; press Ctrl-C to stop");
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for ctrl-c")?;
    proxy.shutdown().await;
    println!("udp proxy stopped");
    Ok(())
}
