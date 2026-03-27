use anyhow::Result;
use rosc_config::BrokerConfig;

use crate::{UdpProxyStatusSnapshot, operator_warnings, proxy_status_from_config};

use super::collect::{
    operator_destination_signals, operator_highlights, operator_overrides, operator_route_signals,
    operator_runtime_signals, operator_state, recent_config_event_kind_label,
};
use super::types::{
    ProxyOperatorReport, ProxyOperatorSignalScope, ProxyOperatorSignalsView,
    ProxyRuntimeSafetyPolicy,
};

pub fn evaluate_proxy_runtime_policy(
    config: &BrokerConfig,
    policy: ProxyRuntimeSafetyPolicy,
) -> Result<(), Vec<String>> {
    let status = match proxy_status_from_config(config) {
        Ok(status) => status,
        Err(error) => return Err(vec![error.to_string()]),
    };
    let blockers = policy.blockers(&status);
    if blockers.is_empty() {
        Ok(())
    } else {
        Err(blockers)
    }
}

pub fn proxy_operator_report(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
) -> ProxyOperatorReport {
    let warnings = operator_warnings(status);
    let blockers = policy.blockers(status);
    let overrides = operator_overrides(status);
    let route_signals = operator_route_signals(status);
    let destination_signals = operator_destination_signals(status);
    let runtime_signals = operator_runtime_signals(status, &route_signals, &destination_signals);
    let highlights = operator_highlights(status);
    let state = operator_state(status, &warnings, &blockers);
    let mut report_lines = proxy_startup_report_lines(status);
    report_lines.push(format!(
        "proxy operator state: state={} warnings={} blockers={} latest_operator_action={} latest_config_issue={}",
        state.as_str(),
        warnings.len(),
        blockers.len(),
        highlights
            .latest_operator_action
            .as_ref()
            .map(|action| action.action.as_str())
            .unwrap_or("none"),
        highlights
            .latest_config_issue
            .as_ref()
            .map(|event| recent_config_event_kind_label(&event.kind))
            .unwrap_or("none")
    ));
    report_lines.push(format!(
        "proxy overrides: launch_profile_mode={} traffic_frozen={} isolated_routes={} disabled_capture_routes={} disabled_replay_routes={} disabled_restart_rehydrate_routes={}",
        overrides.launch_profile_mode,
        overrides.traffic_frozen,
        overrides.isolated_route_ids.len(),
        overrides.disabled_capture_routes.len(),
        overrides.disabled_replay_routes.len(),
        overrides.disabled_restart_rehydrate_routes.len()
    ));
    report_lines.push(format!(
        "proxy runtime signals: ingresses_with_drops={} routes_with_dispatch_failures={} routes_with_transform_failures={} destinations_with_drops={} destinations_with_send_failures={} destinations_with_open_breakers={} destinations_with_half_open_breakers={}",
        runtime_signals.ingresses_with_drops.len(),
        runtime_signals.routes_with_dispatch_failures.len(),
        runtime_signals.routes_with_transform_failures.len(),
        runtime_signals.destinations_with_drops.len(),
        runtime_signals.destinations_with_send_failures.len(),
        runtime_signals.destinations_with_open_breakers.len(),
        runtime_signals.destinations_with_half_open_breakers.len()
    ));
    report_lines.push(format!(
        "proxy safety policy: fail_on_warnings={} require_fallback_ready={}",
        policy.fail_on_warnings, policy.require_fallback_ready
    ));
    report_lines.extend(
        blockers
            .iter()
            .map(|blocker| format!("proxy blocker: {blocker}")),
    );

    ProxyOperatorReport {
        state,
        policy,
        warnings,
        blockers,
        overrides,
        runtime_signals,
        route_signals,
        destination_signals,
        highlights,
        report_lines,
    }
}

pub fn proxy_operator_signals_view(
    report: &ProxyOperatorReport,
    scope: ProxyOperatorSignalScope,
) -> ProxyOperatorSignalsView {
    let route_signals = match scope {
        ProxyOperatorSignalScope::All => report.route_signals.clone(),
        ProxyOperatorSignalScope::Problematic => report
            .route_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .cloned()
            .collect(),
    };
    let destination_signals = match scope {
        ProxyOperatorSignalScope::All => report.destination_signals.clone(),
        ProxyOperatorSignalScope::Problematic => report
            .destination_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .cloned()
            .collect(),
    };

    ProxyOperatorSignalsView {
        scope,
        runtime_signals: report.runtime_signals.clone(),
        route_signals,
        destination_signals,
    }
}

pub fn proxy_startup_report_lines(status: &UdpProxyStatusSnapshot) -> Vec<String> {
    let mut lines = vec![format!(
        "proxy summary: active_routes={} disabled_routes={} active_ingresses={} active_destinations={} fallback_ready_routes={} fallback_missing_routes={} warnings={}",
        status.summary.active_routes,
        status.summary.disabled_routes,
        status.summary.active_ingresses,
        status.summary.active_destinations,
        status.summary.fallback_ready_routes,
        status.summary.fallback_missing_routes,
        status.summary.warning_count
    )];

    lines.push(format!(
        "proxy launch profile: mode={} disabled_capture_routes={} disabled_replay_routes={} disabled_restart_rehydrate_routes={}",
        status.launch_profile.mode.as_str(),
        status.launch_profile.disabled_capture_routes.len(),
        status.launch_profile.disabled_replay_routes.len(),
        status
            .launch_profile
            .disabled_restart_rehydrate_routes
            .len()
    ));

    if let Some(runtime) = &status.runtime {
        lines.push(format!(
            "proxy runtime: traffic_frozen={} isolated_routes={} config_revision={} config_rejections_total={} config_blocked_total={} config_reload_failures_total={}",
            runtime.traffic_frozen,
            runtime.isolated_route_ids.len(),
            runtime.config_revision,
            runtime.config_rejections_total,
            runtime.config_blocked_total,
            runtime.config_reload_failures_total
        ));
    }

    lines.extend(
        operator_warnings(status)
            .into_iter()
            .map(|warning| format!("proxy warning: {warning}")),
    );
    lines
}
