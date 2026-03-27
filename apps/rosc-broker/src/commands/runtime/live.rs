use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;
use tokio::sync::Mutex;

use super::super::ProxyCommandOptions;
use super::sidecars::{
    shutdown_optional_control_service, shutdown_optional_health_service,
    start_managed_proxy_sidecars, start_supervisor_sidecars,
};
use crate::commands::common::{
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

pub(super) fn startup_options(
    options: ProxyCommandOptions,
) -> rosc_broker::ManagedProxyStartupOptions {
    rosc_broker::ManagedProxyStartupOptions {
        frozen_behavior: if options.start_frozen {
            rosc_broker::FrozenStartupBehavior::OperatorRequested
        } else {
            rosc_broker::FrozenStartupBehavior::Normal
        },
        ..rosc_broker::ManagedProxyStartupOptions::default()
    }
}
