use std::path::Path;

use anyhow::Result;

use super::super::ProxyCommandOptions;
use super::super::common::{
    launch_profile_mode, load_config, print_json_pretty, safety_policy, status_from_config,
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

pub(crate) async fn proxy_snapshot(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    print_json_pretty(&snapshot)?;
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
