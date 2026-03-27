use anyhow::Result;
use rosc_config::BrokerConfig;
use rosc_telemetry::BreakerStateSnapshot;
use rosc_telemetry::{RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction};
use serde::Serialize;

use crate::{
    UdpProxyStatusSnapshot, operator_warnings, proxy_status_from_config, startup_blockers,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyRuntimeSafetyPolicy {
    pub fail_on_warnings: bool,
    pub require_fallback_ready: bool,
}

impl ProxyRuntimeSafetyPolicy {
    pub fn blockers(self, status: &UdpProxyStatusSnapshot) -> Vec<String> {
        startup_blockers(status, self.fail_on_warnings, self.require_fallback_ready)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorReport {
    pub state: ProxyOperatorState,
    pub policy: ProxyRuntimeSafetyPolicy,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub overrides: ProxyOperatorOverrides,
    pub runtime_signals: ProxyOperatorRuntimeSignals,
    pub route_signals: Vec<ProxyOperatorRouteSignal>,
    pub destination_signals: Vec<ProxyOperatorDestinationSignal>,
    pub highlights: ProxyOperatorHighlights,
    pub report_lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorSignalScope {
    #[default]
    All,
    Problematic,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorState {
    Healthy,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorHighlights {
    pub latest_operator_action: Option<RecentOperatorAction>,
    pub latest_config_issue: Option<RecentConfigEvent>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorOverrides {
    pub launch_profile_mode: String,
    pub traffic_frozen: bool,
    pub isolated_route_ids: Vec<String>,
    pub disabled_capture_routes: Vec<String>,
    pub disabled_replay_routes: Vec<String>,
    pub disabled_restart_rehydrate_routes: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRuntimeSignals {
    pub ingresses_with_drops: Vec<String>,
    pub routes_with_dispatch_failures: Vec<String>,
    pub routes_with_transform_failures: Vec<String>,
    pub destinations_with_drops: Vec<String>,
    pub destinations_with_send_failures: Vec<String>,
    pub destinations_with_open_breakers: Vec<String>,
    pub destinations_with_half_open_breakers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteSignal {
    pub route_id: String,
    pub active: bool,
    pub isolated: bool,
    pub direct_udp_fallback_available: bool,
    pub config_warnings: Vec<String>,
    pub dispatch_failures_total: u64,
    pub transform_failures_total: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationSignal {
    pub destination_id: String,
    pub queue_depth: usize,
    pub send_total: u64,
    pub send_failures_total: u64,
    pub drops_total: u64,
    pub breaker_state: Option<BreakerStateSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorSignalsView {
    pub scope: ProxyOperatorSignalScope,
    pub runtime_signals: ProxyOperatorRuntimeSignals,
    pub route_signals: Vec<ProxyOperatorRouteSignal>,
    pub destination_signals: Vec<ProxyOperatorDestinationSignal>,
}

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

impl ProxyOperatorSignalScope {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "problematic" => Some(Self::Problematic),
            _ => None,
        }
    }
}

impl ProxyOperatorRouteSignal {
    pub fn is_problematic(&self) -> bool {
        !self.active
            || self.isolated
            || !self.direct_udp_fallback_available
            || !self.config_warnings.is_empty()
            || self.dispatch_failures_total > 0
            || self.transform_failures_total > 0
    }
}

impl ProxyOperatorDestinationSignal {
    pub fn is_problematic(&self) -> bool {
        self.queue_depth > 0
            || self.send_failures_total > 0
            || self.drops_total > 0
            || matches!(
                self.breaker_state,
                Some(BreakerStateSnapshot::Open | BreakerStateSnapshot::HalfOpen)
            )
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

fn operator_state(
    status: &UdpProxyStatusSnapshot,
    warnings: &[String],
    blockers: &[String],
) -> ProxyOperatorState {
    if !blockers.is_empty() {
        return ProxyOperatorState::Blocked;
    }

    let has_runtime_override = status
        .runtime
        .as_ref()
        .is_some_and(|runtime| runtime.traffic_frozen || !runtime.isolated_route_ids.is_empty());
    if !warnings.is_empty() || has_runtime_override {
        ProxyOperatorState::Warning
    } else {
        ProxyOperatorState::Healthy
    }
}

fn operator_highlights(status: &UdpProxyStatusSnapshot) -> ProxyOperatorHighlights {
    let Some(runtime) = &status.runtime else {
        return ProxyOperatorHighlights::default();
    };

    ProxyOperatorHighlights {
        latest_operator_action: runtime.recent_operator_actions.last().cloned(),
        latest_config_issue: runtime
            .recent_config_events
            .iter()
            .rev()
            .find(|event| {
                !matches!(
                    event.kind,
                    RecentConfigEventKind::Applied | RecentConfigEventKind::LaunchProfileChanged
                )
            })
            .cloned(),
    }
}

fn operator_overrides(status: &UdpProxyStatusSnapshot) -> ProxyOperatorOverrides {
    let traffic_frozen = status
        .runtime
        .as_ref()
        .map(|runtime| runtime.traffic_frozen)
        .unwrap_or(false);
    let isolated_route_ids = status
        .runtime
        .as_ref()
        .map(|runtime| runtime.isolated_route_ids.clone())
        .unwrap_or_default();
    ProxyOperatorOverrides {
        launch_profile_mode: status.launch_profile.mode.as_str().to_owned(),
        traffic_frozen,
        isolated_route_ids,
        disabled_capture_routes: status.launch_profile.disabled_capture_routes.clone(),
        disabled_replay_routes: status.launch_profile.disabled_replay_routes.clone(),
        disabled_restart_rehydrate_routes: status
            .launch_profile
            .disabled_restart_rehydrate_routes
            .clone(),
    }
}

fn operator_runtime_signals(
    status: &UdpProxyStatusSnapshot,
    route_signals: &[ProxyOperatorRouteSignal],
    destination_signals: &[ProxyOperatorDestinationSignal],
) -> ProxyOperatorRuntimeSignals {
    let ingresses_with_drops = status
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .ingress_drops_total
                .iter()
                .filter(|(_, total)| **total > 0)
                .map(|(ingress_id, _)| ingress_id.clone())
                .collect()
        })
        .unwrap_or_default();

    ProxyOperatorRuntimeSignals {
        ingresses_with_drops,
        routes_with_dispatch_failures: route_signals
            .iter()
            .filter(|signal| signal.dispatch_failures_total > 0)
            .map(|signal| signal.route_id.clone())
            .collect(),
        routes_with_transform_failures: route_signals
            .iter()
            .filter(|signal| signal.transform_failures_total > 0)
            .map(|signal| signal.route_id.clone())
            .collect(),
        destinations_with_drops: destination_signals
            .iter()
            .filter(|signal| signal.drops_total > 0)
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_send_failures: destination_signals
            .iter()
            .filter(|signal| signal.send_failures_total > 0)
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_open_breakers: destination_signals
            .iter()
            .filter(|signal| signal.breaker_state == Some(BreakerStateSnapshot::Open))
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_half_open_breakers: destination_signals
            .iter()
            .filter(|signal| signal.breaker_state == Some(BreakerStateSnapshot::HalfOpen))
            .map(|signal| signal.destination_id.clone())
            .collect(),
    }
}

fn operator_route_signals(status: &UdpProxyStatusSnapshot) -> Vec<ProxyOperatorRouteSignal> {
    let runtime = status.runtime.as_ref();
    status
        .route_assessments
        .iter()
        .map(|assessment| ProxyOperatorRouteSignal {
            route_id: assessment.route_id.clone(),
            active: assessment.active,
            isolated: runtime
                .is_some_and(|runtime| runtime.isolated_route_ids.contains(&assessment.route_id)),
            direct_udp_fallback_available: assessment.direct_udp_fallback_available,
            config_warnings: assessment.warnings.clone(),
            dispatch_failures_total: runtime
                .and_then(|runtime| runtime.dispatch_failures_total.get(&assessment.route_id))
                .copied()
                .unwrap_or_default(),
            transform_failures_total: runtime
                .and_then(|runtime| {
                    runtime
                        .route_transform_failures_total
                        .get(&assessment.route_id)
                })
                .copied()
                .unwrap_or_default(),
        })
        .collect()
}

fn operator_destination_signals(
    status: &UdpProxyStatusSnapshot,
) -> Vec<ProxyOperatorDestinationSignal> {
    status
        .destinations
        .iter()
        .map(|destination| {
            let runtime = status.runtime.as_ref().and_then(|runtime| {
                runtime.destinations.iter().find(|runtime_destination| {
                    runtime_destination.destination_id == destination.id
                })
            });
            let drops_total = status
                .runtime
                .as_ref()
                .and_then(|runtime| runtime.destination_drops_total.get(&destination.id))
                .copied()
                .unwrap_or_default();
            ProxyOperatorDestinationSignal {
                destination_id: destination.id.clone(),
                queue_depth: runtime
                    .map(|runtime| runtime.queue_depth)
                    .unwrap_or_default(),
                send_total: runtime
                    .map(|runtime| runtime.send_total)
                    .unwrap_or_default(),
                send_failures_total: runtime
                    .map(|runtime| runtime.send_failures_total)
                    .unwrap_or_default(),
                drops_total,
                breaker_state: runtime.and_then(|runtime| runtime.breaker_state.clone()),
            }
        })
        .collect()
}

impl ProxyOperatorState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}

fn recent_config_event_kind_label(kind: &RecentConfigEventKind) -> &'static str {
    match kind {
        RecentConfigEventKind::Applied => "applied",
        RecentConfigEventKind::Rejected => "rejected",
        RecentConfigEventKind::Blocked => "blocked",
        RecentConfigEventKind::ReloadFailed => "reload_failed",
        RecentConfigEventKind::LaunchProfileChanged => "launch_profile_changed",
    }
}
