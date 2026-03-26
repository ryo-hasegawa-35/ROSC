use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;

use crate::cli::Command;

#[derive(Clone, Copy)]
struct ProxyCommandOptions {
    fail_on_warnings: bool,
    require_fallback_ready: bool,
    safe_mode: bool,
    start_frozen: bool,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::CheckConfig { path } => check_config(&path).await,
        Command::ProxyStatus {
            config,
            resolve_bindings,
            safe_mode,
        } => proxy_status(&config, resolve_bindings, safe_mode).await,
        Command::WatchConfig {
            path,
            poll_ms,
            fail_on_warnings,
            require_fallback_ready,
        } => watch_config(&path, poll_ms, fail_on_warnings, require_fallback_ready).await,
        Command::WatchUdpProxy {
            config,
            poll_ms,
            ingress_queue_depth,
            health_listen,
            fail_on_warnings,
            require_fallback_ready,
            safe_mode,
            start_frozen,
        } => {
            watch_udp_proxy(
                &config,
                poll_ms,
                ingress_queue_depth,
                health_listen.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen,
                },
            )
            .await
        }
        Command::DiffConfig { current, candidate } => diff_config(&current, &candidate).await,
        Command::ServeHealth { listen, config } => serve_health(&listen, config.as_deref()).await,
        Command::RunUdpProxy {
            config,
            ingress_queue_depth,
            health_listen,
            fail_on_warnings,
            require_fallback_ready,
            safe_mode,
            start_frozen,
        } => {
            run_udp_proxy(
                &config,
                ingress_queue_depth,
                health_listen.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen,
                },
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

async fn proxy_status(path: &Path, resolve_bindings: bool, safe_mode: bool) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
    let launch_profile_mode = if safe_mode {
        rosc_broker::ProxyLaunchProfileMode::SafeMode
    } else {
        rosc_broker::ProxyLaunchProfileMode::Normal
    };
    let prepared = rosc_broker::apply_launch_profile(&config, launch_profile_mode);
    let status = if resolve_bindings {
        let mut app =
            rosc_broker::UdpProxyApp::from_config(&prepared.config, InMemoryTelemetry::default())
                .await?;
        app.set_launch_profile(prepared.status);
        app.status_snapshot()
    } else {
        let mut status = rosc_broker::proxy_status_from_config(&prepared.config)?;
        status.launch_profile = prepared.status;
        status
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
                        print_applied_config(&applied);
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

async fn watch_udp_proxy(
    path: &Path,
    poll_ms: u64,
    ingress_queue_depth: usize,
    health_listen: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let safety_policy = rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings: options.fail_on_warnings,
        require_fallback_ready: options.require_fallback_ready,
    };
    let telemetry = InMemoryTelemetry::default();
    let launch_profile_mode = if options.safe_mode {
        rosc_broker::ProxyLaunchProfileMode::SafeMode
    } else {
        rosc_broker::ProxyLaunchProfileMode::Normal
    };
    let mut supervisor = rosc_broker::ManagedProxyFileSupervisor::start(
        path,
        telemetry.clone(),
        ingress_queue_depth,
        safety_policy,
        launch_profile_mode,
        rosc_broker::ManagedProxyStartupOptions {
            frozen_behavior: if options.start_frozen {
                rosc_broker::FrozenStartupBehavior::OperatorRequested
            } else {
                rosc_broker::FrozenStartupBehavior::Normal
            },
            ..rosc_broker::ManagedProxyStartupOptions::default()
        },
    )
    .await?;
    let mut health_service = spawn_optional_health_service(health_listen, telemetry).await?;
    print_proxy_report(&supervisor.status_snapshot());
    println!(
        "managed udp proxy loaded revision={}",
        supervisor.current_revision().unwrap_or_default()
    );

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(poll_ms));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match supervisor.poll_once().await? {
                    rosc_broker::ProxyReloadOutcome::Unchanged => {}
                    rosc_broker::ProxyReloadOutcome::Applied(applied) => {
                        print_applied_config(&applied);
                        print_proxy_report(&supervisor.status_snapshot());
                    }
                    rosc_broker::ProxyReloadOutcome::Blocked(reasons) => {
                        println!(
                            "blocked proxy reload; keeping revision={} reasons={}",
                            supervisor.current_revision().unwrap_or_default(),
                            reasons.join(" | ")
                        );
                        print_proxy_report(&supervisor.status_snapshot());
                    }
                    rosc_broker::ProxyReloadOutcome::Rejected(error) => {
                        println!(
                            "rejected proxy reload; keeping revision={} reason={}",
                            supervisor.current_revision().unwrap_or_default(),
                            error
                        );
                        print_proxy_report(&supervisor.status_snapshot());
                    }
                    rosc_broker::ProxyReloadOutcome::ReloadFailed(error) => {
                        println!(
                            "failed proxy reload; keeping revision={} reason={}",
                            supervisor.current_revision().unwrap_or_default(),
                            error
                        );
                        print_proxy_report(&supervisor.status_snapshot());
                    }
                }
            }
            result = tokio::signal::ctrl_c() => {
                result.context("failed to listen for ctrl-c")?;
                break;
            }
        }
    }

    shutdown_optional_health_service(&mut health_service).await?;
    supervisor.shutdown().await;
    println!("managed udp proxy stopped");
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

async fn run_udp_proxy(
    path: &Path,
    ingress_queue_depth: usize,
    health_listen: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = rosc_config::BrokerConfig::from_toml_str(&content)?;
    let launch_profile_mode = if options.safe_mode {
        rosc_broker::ProxyLaunchProfileMode::SafeMode
    } else {
        rosc_broker::ProxyLaunchProfileMode::Normal
    };

    let safety_policy = rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings: options.fail_on_warnings,
        require_fallback_ready: options.require_fallback_ready,
    };
    let telemetry = InMemoryTelemetry::default();
    let mut proxy = rosc_broker::ManagedUdpProxy::start(
        config,
        telemetry.clone(),
        ingress_queue_depth,
        safety_policy,
        launch_profile_mode,
        rosc_broker::ManagedProxyStartupOptions {
            frozen_behavior: if options.start_frozen {
                rosc_broker::FrozenStartupBehavior::OperatorRequested
            } else {
                rosc_broker::FrozenStartupBehavior::Normal
            },
            ..rosc_broker::ManagedProxyStartupOptions::default()
        },
    )
    .await?;
    let mut health_service = spawn_optional_health_service(health_listen, telemetry).await?;
    print_proxy_report(&proxy.app().status_snapshot());
    println!("udp proxy running; press Ctrl-C to stop");
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for ctrl-c")?;
    shutdown_optional_health_service(&mut health_service).await?;
    proxy.shutdown().await;
    println!("udp proxy stopped");
    Ok(())
}

fn print_applied_config(applied: &rosc_config::ConfigApplyResult) {
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

fn print_proxy_report(status: &rosc_broker::UdpProxyStatusSnapshot) {
    for line in rosc_broker::proxy_startup_report_lines(status) {
        println!("{line}");
    }
}

async fn spawn_optional_health_service(
    health_listen: Option<&str>,
    telemetry: InMemoryTelemetry,
) -> Result<Option<rosc_broker::HealthService>> {
    match health_listen {
        Some(listen) => {
            let service = rosc_broker::HealthService::spawn(listen, Arc::new(telemetry)).await?;
            println!("health endpoint listening on {}", service.listen_addr());
            Ok(Some(service))
        }
        None => Ok(None),
    }
}

async fn shutdown_optional_health_service(
    service: &mut Option<rosc_broker::HealthService>,
) -> Result<()> {
    if let Some(service) = service.as_mut() {
        service.shutdown().await?;
        println!("health endpoint stopped");
    }
    Ok(())
}
