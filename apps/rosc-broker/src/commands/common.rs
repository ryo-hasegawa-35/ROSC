use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;

use super::ProxyCommandOptions;

pub(crate) fn load_config(path: &Path) -> Result<rosc_config::BrokerConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    Ok(rosc_config::BrokerConfig::from_toml_str(&content)?)
}

pub(crate) fn launch_profile_mode(
    options: ProxyCommandOptions,
) -> rosc_broker::ProxyLaunchProfileMode {
    if options.safe_mode {
        rosc_broker::ProxyLaunchProfileMode::SafeMode
    } else {
        rosc_broker::ProxyLaunchProfileMode::Normal
    }
}

pub(crate) fn safety_policy(options: ProxyCommandOptions) -> rosc_broker::ProxyRuntimeSafetyPolicy {
    rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings: options.fail_on_warnings,
        require_fallback_ready: options.require_fallback_ready,
    }
}

pub(crate) async fn resolved_status_from_config(
    config: &rosc_config::BrokerConfig,
    launch_profile_mode: rosc_broker::ProxyLaunchProfileMode,
) -> Result<rosc_broker::UdpProxyStatusSnapshot> {
    let prepared = rosc_broker::apply_launch_profile(config, launch_profile_mode);
    let mut app =
        rosc_broker::UdpProxyApp::from_config(&prepared.config, InMemoryTelemetry::default())
            .await?;
    app.apply_launch_profile(prepared.status);
    Ok(app.status_snapshot())
}

pub(crate) fn unresolved_status_from_config(
    config: &rosc_config::BrokerConfig,
    launch_profile_mode: rosc_broker::ProxyLaunchProfileMode,
) -> Result<rosc_broker::UdpProxyStatusSnapshot> {
    let prepared = rosc_broker::apply_launch_profile(config, launch_profile_mode);
    let mut status = rosc_broker::proxy_status_from_config(&prepared.config)?;
    status.launch_profile = prepared.status;
    Ok(status)
}

pub(crate) async fn status_from_config(
    config: &rosc_config::BrokerConfig,
    resolve_bindings: bool,
    launch_profile_mode: rosc_broker::ProxyLaunchProfileMode,
) -> Result<rosc_broker::UdpProxyStatusSnapshot> {
    if resolve_bindings {
        resolved_status_from_config(config, launch_profile_mode).await
    } else {
        unresolved_status_from_config(config, launch_profile_mode)
    }
}

pub(crate) fn print_applied_config(applied: &rosc_config::ConfigApplyResult) {
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

pub(crate) fn print_proxy_report(
    status: &rosc_broker::UdpProxyStatusSnapshot,
    safety_policy: rosc_broker::ProxyRuntimeSafetyPolicy,
) {
    let report = rosc_broker::proxy_operator_report(status, safety_policy);
    for line in report.report_lines {
        println!("{line}");
    }
}

pub(crate) fn print_proxy_overview_summary(overview: &rosc_broker::ProxyOperatorOverview) {
    println!(
        "proxy overview: state={} blockers={} warnings={} problematic_routes={} problematic_destinations={} recent_operator_actions={} recent_config_events={}",
        match overview.report.state {
            rosc_broker::ProxyOperatorState::Healthy => "healthy",
            rosc_broker::ProxyOperatorState::Warning => "warning",
            rosc_broker::ProxyOperatorState::Blocked => "blocked",
        },
        overview.report.blockers.len(),
        overview.report.warnings.len(),
        overview.problematic_signals.route_signals.len(),
        overview.problematic_signals.destination_signals.len(),
        overview.runtime_summary.recent_operator_action_count,
        overview.runtime_summary.recent_config_event_count,
    );
}

pub(crate) fn print_proxy_diagnostics_summary(diagnostics: &rosc_broker::ProxyOperatorDiagnostics) {
    let overview = &diagnostics.overview;
    println!(
        "proxy diagnostics: state={} blockers={} warnings={} problematic_routes={} problematic_destinations={} recent_operator_actions={} recent_config_events={} traffic_frozen={} isolated_routes={} backlog_destinations={} open_breakers={}",
        match overview.report.state {
            rosc_broker::ProxyOperatorState::Healthy => "healthy",
            rosc_broker::ProxyOperatorState::Warning => "warning",
            rosc_broker::ProxyOperatorState::Blocked => "blocked",
        },
        overview.report.blockers.len(),
        overview.report.warnings.len(),
        overview.problematic_signals.route_signals.len(),
        overview.problematic_signals.destination_signals.len(),
        diagnostics.recent_operator_actions.len(),
        diagnostics.recent_config_events.len(),
        overview.runtime_summary.traffic_frozen,
        overview.runtime_summary.isolated_route_count,
        overview.runtime_summary.destinations_with_backlog,
        overview.runtime_summary.destinations_with_open_breakers,
    );
}
