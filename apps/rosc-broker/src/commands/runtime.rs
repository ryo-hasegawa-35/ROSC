use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;
use tokio::sync::Mutex;

use super::ProxyCommandOptions;
use super::common::{
    launch_profile_mode, load_config, print_applied_config, print_proxy_overview_summary,
    print_proxy_report, safety_policy,
};

pub(crate) async fn watch_udp_proxy(
    path: &Path,
    poll_ms: u64,
    ingress_queue_depth: usize,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let safety_policy = safety_policy(options);
    let telemetry = InMemoryTelemetry::default();
    let supervisor = Arc::new(Mutex::new(
        rosc_broker::ManagedProxyFileSupervisor::start(
            path,
            telemetry.clone(),
            ingress_queue_depth,
            safety_policy,
            launch_profile_mode(options),
            startup_options(options),
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
    {
        let supervisor = supervisor.lock().await;
        let initial_overview = supervisor.operator_overview();
        print_proxy_report(&initial_overview.status, safety_policy);
        print_proxy_overview_summary(&initial_overview);
        println!(
            "managed udp proxy loaded revision={}",
            supervisor.current_revision().unwrap_or_default()
        );
    }

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
                        let overview = supervisor.lock().await.operator_overview();
                        print_proxy_report(&overview.status, safety_policy);
                        print_proxy_overview_summary(&overview);
                    }
                    rosc_broker::ProxyReloadOutcome::Blocked(reasons) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "blocked proxy reload; keeping revision={} reasons={}",
                            revision,
                            reasons.join(" | ")
                        );
                        let overview = supervisor.lock().await.operator_overview();
                        print_proxy_report(&overview.status, safety_policy);
                        print_proxy_overview_summary(&overview);
                    }
                    rosc_broker::ProxyReloadOutcome::Rejected(error) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "rejected proxy reload; keeping revision={} reason={}",
                            revision,
                            error
                        );
                        let overview = supervisor.lock().await.operator_overview();
                        print_proxy_report(&overview.status, safety_policy);
                        print_proxy_overview_summary(&overview);
                    }
                    rosc_broker::ProxyReloadOutcome::ReloadFailed(error) => {
                        let revision = supervisor.lock().await.current_revision().unwrap_or_default();
                        println!(
                            "failed proxy reload; keeping revision={} reason={}",
                            revision,
                            error
                        );
                        let overview = supervisor.lock().await.operator_overview();
                        print_proxy_report(&overview.status, safety_policy);
                        print_proxy_overview_summary(&overview);
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

pub(crate) async fn run_udp_proxy(
    path: &Path,
    ingress_queue_depth: usize,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let safety_policy = safety_policy(options);
    let telemetry = InMemoryTelemetry::default();
    let proxy = Arc::new(Mutex::new(
        rosc_broker::ManagedUdpProxy::start(
            config,
            telemetry.clone(),
            ingress_queue_depth,
            safety_policy,
            launch_profile_mode(options),
            startup_options(options),
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
    {
        let proxy = proxy.lock().await;
        let overview = proxy.operator_overview();
        print_proxy_report(&overview.status, safety_policy);
        print_proxy_overview_summary(&overview);
    }
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

fn startup_options(options: ProxyCommandOptions) -> rosc_broker::ManagedProxyStartupOptions {
    rosc_broker::ManagedProxyStartupOptions {
        frozen_behavior: if options.start_frozen {
            rosc_broker::FrozenStartupBehavior::OperatorRequested
        } else {
            rosc_broker::FrozenStartupBehavior::Normal
        },
        ..rosc_broker::ManagedProxyStartupOptions::default()
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

async fn shutdown_optional_control_service(
    service: &mut Option<rosc_broker::ControlService>,
) -> Result<()> {
    if let Some(service) = service.as_mut() {
        service.shutdown().await?;
        println!("control endpoint stopped");
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::{start_managed_proxy_sidecars, start_supervisor_sidecars};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static UNIQUE_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_config_path() -> PathBuf {
        let nonce = UNIQUE_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("rosc-commands-{pid}-{nonce}.toml"))
    }

    fn proxy_config(ingress_bind: &str, destination_addr: &str) -> rosc_config::BrokerConfig {
        rosc_config::BrokerConfig::from_toml_str(&format!(
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
        let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
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
                .contains("control listener must bind to a loopback address")
        );

        let rebound = tokio::net::UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after control startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
    }

    #[tokio::test]
    async fn start_managed_proxy_sidecars_releases_ingress_port_when_health_startup_fails() {
        let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let occupied_health = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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
        assert!(error.to_string().contains("failed to bind health listener"));

        let rebound = tokio::net::UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after health startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
    }

    #[tokio::test]
    async fn start_supervisor_sidecars_releases_ingress_port_when_control_startup_fails() {
        let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
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
                .contains("control listener must bind to a loopback address")
        );

        let rebound = tokio::net::UdpSocket::bind(ingress_addr)
            .await
            .expect("ingress port should be released after supervisor control startup failure");
        assert_eq!(rebound.local_addr().unwrap(), ingress_addr);

        let _ = fs::remove_file(path);
    }
}
