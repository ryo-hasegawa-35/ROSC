use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;
use tokio::sync::Mutex;

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
            control_listen,
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
                control_listen.as_deref(),
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
            control_listen,
            fail_on_warnings,
            require_fallback_ready,
            safe_mode,
            start_frozen,
        } => {
            run_udp_proxy(
                &config,
                ingress_queue_depth,
                health_listen.as_deref(),
                control_listen.as_deref(),
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
        app.apply_launch_profile(prepared.status);
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
    control_listen: Option<&str>,
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
    let supervisor = Arc::new(Mutex::new(
        rosc_broker::ManagedProxyFileSupervisor::start(
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
        .await?,
    ));
    let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
        rosc_broker::ManagedProxyFileSupervisorController::new(Arc::clone(&supervisor)),
    );
    let (mut health_service, mut control_service) = start_supervisor_sidecars(
        &supervisor,
        telemetry,
        health_listen,
        control_listen,
        control_plane,
    )
    .await?;
    let initial_status = supervisor.lock().await.status_snapshot();
    print_proxy_report(&initial_status, safety_policy);
    println!(
        "managed udp proxy loaded revision={}",
        supervisor
            .lock()
            .await
            .current_revision()
            .unwrap_or_default()
    );

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(poll_ms));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let outcome = {
                    let mut supervisor = supervisor.lock().await;
                    supervisor.poll_once().await?
                };
                match outcome {
                    rosc_broker::ProxyReloadOutcome::Unchanged => {}
                    rosc_broker::ProxyReloadOutcome::Applied(applied) => {
                        print_applied_config(&applied);
                        let status = supervisor.lock().await.status_snapshot();
                        print_proxy_report(&status, safety_policy);
                    }
                    rosc_broker::ProxyReloadOutcome::Blocked(reasons) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "blocked proxy reload; keeping revision={} reasons={}",
                            revision,
                            reasons.join(" | ")
                        );
                        let status = supervisor.lock().await.status_snapshot();
                        print_proxy_report(&status, safety_policy);
                    }
                    rosc_broker::ProxyReloadOutcome::Rejected(error) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "rejected proxy reload; keeping revision={} reason={}",
                            revision,
                            error
                        );
                        let status = supervisor.lock().await.status_snapshot();
                        print_proxy_report(&status, safety_policy);
                    }
                    rosc_broker::ProxyReloadOutcome::ReloadFailed(error) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "failed proxy reload; keeping revision={} reason={}",
                            revision,
                            error
                        );
                        let status = supervisor.lock().await.status_snapshot();
                        print_proxy_report(&status, safety_policy);
                    }
                }
            }
            result = tokio::signal::ctrl_c() => {
                result.context("failed to listen for ctrl-c")?;
                break;
            }
        }
    }

    shutdown_optional_control_service(&mut control_service).await?;
    shutdown_optional_health_service(&mut health_service).await?;
    supervisor.lock().await.shutdown().await;
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
    control_listen: Option<&str>,
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
    let proxy = Arc::new(Mutex::new(
        rosc_broker::ManagedUdpProxy::start(
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
        .await?,
    ));
    let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
        rosc_broker::ManagedUdpProxyController::new(Arc::clone(&proxy)),
    );
    let (mut health_service, mut control_service) = start_managed_proxy_sidecars(
        &proxy,
        telemetry,
        health_listen,
        control_listen,
        control_plane,
    )
    .await?;
    let status = proxy.lock().await.status_snapshot();
    print_proxy_report(&status, safety_policy);
    println!("udp proxy running; press Ctrl-C to stop");
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for ctrl-c")?;
    shutdown_optional_control_service(&mut control_service).await?;
    shutdown_optional_health_service(&mut health_service).await?;
    proxy.lock().await.shutdown().await;
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

fn print_proxy_report(
    status: &rosc_broker::UdpProxyStatusSnapshot,
    safety_policy: rosc_broker::ProxyRuntimeSafetyPolicy,
) {
    let report = rosc_broker::proxy_operator_report(status, safety_policy);
    for line in report.report_lines {
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

async fn spawn_optional_control_service(
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<Option<rosc_broker::ControlService>> {
    match control_listen {
        Some(listen) => {
            let service = rosc_broker::ControlService::spawn(listen, control_plane).await?;
            println!("control endpoint listening on {}", service.listen_addr());
            Ok(Some(service))
        }
        None => Ok(None),
    }
}

async fn start_managed_proxy_sidecars(
    proxy: &Arc<Mutex<rosc_broker::ManagedUdpProxy>>,
    telemetry: InMemoryTelemetry,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<(
    Option<rosc_broker::HealthService>,
    Option<rosc_broker::ControlService>,
)> {
    let mut health_service = match spawn_optional_health_service(health_listen, telemetry).await {
        Ok(service) => service,
        Err(error) => {
            proxy.lock().await.shutdown().await;
            return Err(error);
        }
    };

    let control_service = match spawn_optional_control_service(control_listen, control_plane).await
    {
        Ok(service) => service,
        Err(error) => {
            shutdown_optional_health_service(&mut health_service).await?;
            proxy.lock().await.shutdown().await;
            return Err(error);
        }
    };

    Ok((health_service, control_service))
}

async fn start_supervisor_sidecars(
    supervisor: &Arc<Mutex<rosc_broker::ManagedProxyFileSupervisor>>,
    telemetry: InMemoryTelemetry,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<(
    Option<rosc_broker::HealthService>,
    Option<rosc_broker::ControlService>,
)> {
    let mut health_service = match spawn_optional_health_service(health_listen, telemetry).await {
        Ok(service) => service,
        Err(error) => {
            supervisor.lock().await.shutdown().await;
            return Err(error);
        }
    };

    let control_service = match spawn_optional_control_service(control_listen, control_plane).await
    {
        Ok(service) => service,
        Err(error) => {
            shutdown_optional_health_service(&mut health_service).await?;
            supervisor.lock().await.shutdown().await;
            return Err(error);
        }
    };

    Ok((health_service, control_service))
}

async fn shutdown_optional_control_service(
    service: &mut Option<rosc_broker::ControlService>,
) -> Result<()> {
    if let Some(service) = service.as_mut() {
        service.shutdown().await?;
        println!("control endpoint stopped");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use rosc_config::BrokerConfig;
    use rosc_telemetry::InMemoryTelemetry;
    use tokio::net::TcpListener;
    use tokio::net::UdpSocket;
    use tokio::sync::Mutex;

    use super::{start_managed_proxy_sidecars, start_supervisor_sidecars};

    static UNIQUE_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_config_path() -> PathBuf {
        let nonce = UNIQUE_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("rosc-commands-{pid}-{nonce}.toml"))
    }

    fn proxy_config(ingress_bind: &str, destination_addr: &str) -> BrokerConfig {
        BrokerConfig::from_toml_str(&format!(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "{ingress_bind}"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "udp_renderer"
            bind = "127.0.0.1:0"
            target = "{destination_addr}"

            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["udp_localhost_in"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
        ))
        .unwrap()
    }

    fn proxy_config_toml(ingress_bind: &str, destination_addr: &str) -> String {
        format!(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "{ingress_bind}"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "udp_renderer"
            bind = "127.0.0.1:0"
            target = "{destination_addr}"

            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["udp_localhost_in"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
        )
    }

    #[tokio::test]
    async fn start_managed_proxy_sidecars_releases_ingress_port_when_control_startup_fails() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let telemetry = InMemoryTelemetry::default();
        let proxy = Arc::new(Mutex::new(
            rosc_broker::ManagedUdpProxy::start(
                proxy_config(
                    "127.0.0.1:0",
                    &destination_listener.local_addr().unwrap().to_string(),
                ),
                telemetry.clone(),
                32,
                rosc_broker::ProxyRuntimeSafetyPolicy::default(),
                rosc_broker::ProxyLaunchProfileMode::Normal,
                rosc_broker::ManagedProxyStartupOptions::default(),
            )
            .await
            .unwrap(),
        ));
        let ingress_addr = proxy
            .lock()
            .await
            .app()
            .ingress_local_addr("udp_localhost_in")
            .unwrap();
        let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
            rosc_broker::ManagedUdpProxyController::new(Arc::clone(&proxy)),
        );

        let error = match start_managed_proxy_sidecars(
            &proxy,
            telemetry,
            None,
            Some("0.0.0.0:0"),
            control_plane,
        )
        .await
        {
            Ok(_) => panic!("non-loopback control listener should fail"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("control listener must bind to a loopback address"),
            "unexpected error: {error:#}"
        );

        let rebound = UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after control startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
    }

    #[tokio::test]
    async fn start_managed_proxy_sidecars_releases_ingress_port_when_health_startup_fails() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let occupied_health = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let telemetry = InMemoryTelemetry::default();
        let proxy = Arc::new(Mutex::new(
            rosc_broker::ManagedUdpProxy::start(
                proxy_config(
                    "127.0.0.1:0",
                    &destination_listener.local_addr().unwrap().to_string(),
                ),
                telemetry.clone(),
                32,
                rosc_broker::ProxyRuntimeSafetyPolicy::default(),
                rosc_broker::ProxyLaunchProfileMode::Normal,
                rosc_broker::ManagedProxyStartupOptions::default(),
            )
            .await
            .unwrap(),
        ));
        let ingress_addr = proxy
            .lock()
            .await
            .app()
            .ingress_local_addr("udp_localhost_in")
            .unwrap();
        let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
            rosc_broker::ManagedUdpProxyController::new(Arc::clone(&proxy)),
        );

        let error = match start_managed_proxy_sidecars(
            &proxy,
            telemetry,
            Some(&occupied_health.local_addr().unwrap().to_string()),
            None,
            control_plane,
        )
        .await
        {
            Ok(_) => panic!("occupied health listener should fail"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("failed to bind health listener"),
            "unexpected error: {error:#}"
        );

        let rebound = UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after health startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
    }

    #[tokio::test]
    async fn start_supervisor_sidecars_releases_ingress_port_when_control_startup_fails() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let path = unique_config_path();
        fs::write(
            &path,
            proxy_config_toml(
                "127.0.0.1:0",
                &destination_listener.local_addr().unwrap().to_string(),
            ),
        )
        .unwrap();

        let telemetry = InMemoryTelemetry::default();
        let supervisor = Arc::new(Mutex::new(
            rosc_broker::ManagedProxyFileSupervisor::start(
                &path,
                telemetry.clone(),
                32,
                rosc_broker::ProxyRuntimeSafetyPolicy::default(),
                rosc_broker::ProxyLaunchProfileMode::Normal,
                rosc_broker::ManagedProxyStartupOptions::default(),
            )
            .await
            .unwrap(),
        ));
        let ingress_addr = supervisor
            .lock()
            .await
            .proxy()
            .app()
            .ingress_local_addr("udp_localhost_in")
            .unwrap();
        let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
            rosc_broker::ManagedProxyFileSupervisorController::new(Arc::clone(&supervisor)),
        );

        let error = match start_supervisor_sidecars(
            &supervisor,
            telemetry,
            None,
            Some("0.0.0.0:0"),
            control_plane,
        )
        .await
        {
            Ok(_) => panic!("non-loopback control listener should fail"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("control listener must bind to a loopback address"),
            "unexpected error: {error:#}"
        );

        let rebound = UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after supervisor control startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);

        let _ = fs::remove_file(path);
    }
}
