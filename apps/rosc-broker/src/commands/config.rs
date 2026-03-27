use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;

use super::ProxyCommandOptions;
use super::common::{
    launch_profile_mode, load_config, print_applied_config, print_json_pretty, safety_policy,
    status_from_config,
};

pub(crate) async fn check_config(path: &Path) -> Result<()> {
    let config = load_config(path)?;
    println!(
        "valid config: schema_version={} route(s)={}",
        config.schema_version,
        config.routes.len()
    );
    Ok(())
}

pub(crate) async fn proxy_status(
    path: &Path,
    resolve_bindings: bool,
    safe_mode: bool,
) -> Result<()> {
    let config = load_config(path)?;
    let status = status_from_config(
        &config,
        resolve_bindings,
        launch_profile_mode(ProxyCommandOptions {
            fail_on_warnings: false,
            require_fallback_ready: false,
            safe_mode,
            start_frozen: false,
        }),
    )
    .await?;
    print_json_pretty(&status)?;
    Ok(())
}

pub(crate) async fn proxy_overview(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let overview = rosc_broker::proxy_operator_overview(&status, safety_policy(options));
    print_json_pretty(&overview)?;
    Ok(())
}

pub(crate) async fn proxy_readiness(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let readiness = rosc_broker::proxy_operator_readiness(&status, safety_policy(options));
    print_json_pretty(&readiness)?;
    Ok(())
}

pub(crate) async fn proxy_diagnostics(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let diagnostics =
        rosc_broker::proxy_operator_diagnostics(&status, safety_policy(options), history_limit);
    print_json_pretty(&diagnostics)?;
    Ok(())
}

pub(crate) async fn proxy_attention(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let report = rosc_broker::proxy_operator_report(&status, safety_policy(options));
    let attention = rosc_broker::proxy_operator_attention(&report);
    print_json_pretty(&attention)?;
    Ok(())
}

pub(crate) async fn proxy_incidents(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let report = rosc_broker::proxy_operator_report(&status, safety_policy(options));
    let (recent_operator_actions, recent_config_events) = status
        .runtime
        .as_ref()
        .map(|runtime| {
            (
                runtime.recent_operator_actions.clone(),
                runtime.recent_config_events.clone(),
            )
        })
        .unwrap_or_default();
    let incidents = rosc_broker::proxy_operator_incidents_from_histories(
        &report,
        recent_operator_actions,
        recent_config_events,
        history_limit,
    );
    print_json_pretty(&incidents)?;
    Ok(())
}

pub(crate) async fn watch_config(
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

pub(crate) async fn diff_config(current: &Path, candidate: &Path) -> Result<()> {
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
