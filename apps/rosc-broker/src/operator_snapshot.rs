use serde::Serialize;

use crate::{
    ProxyOperatorAttention, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverview, ProxyOperatorReadiness, ProxyRuntimeSafetyPolicy,
    UdpProxyStatusSnapshot, proxy_operator_attention, proxy_operator_diagnostics_from_overview,
    proxy_operator_incidents_from_histories, proxy_operator_overview,
    proxy_operator_readiness_from_overview,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorSnapshot {
    pub overview: ProxyOperatorOverview,
    pub readiness: ProxyOperatorReadiness,
    pub diagnostics: ProxyOperatorDiagnostics,
    pub attention: ProxyOperatorAttention,
    pub incidents: ProxyOperatorIncidents,
}

pub fn proxy_operator_snapshot(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
    history_limit: Option<usize>,
) -> ProxyOperatorSnapshot {
    proxy_operator_snapshot_from_overview(proxy_operator_overview(status, policy), history_limit)
}

pub fn proxy_operator_snapshot_from_overview(
    overview: ProxyOperatorOverview,
    history_limit: Option<usize>,
) -> ProxyOperatorSnapshot {
    let report = overview.report.clone();
    let recent_operator_actions = overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| runtime.recent_operator_actions.clone())
        .unwrap_or_default();
    let recent_config_events = overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| runtime.recent_config_events.clone())
        .unwrap_or_default();

    let readiness = proxy_operator_readiness_from_overview(overview.clone());
    let diagnostics = proxy_operator_diagnostics_from_overview(
        overview.clone(),
        recent_operator_actions.clone(),
        recent_config_events.clone(),
        history_limit,
    );
    let attention = proxy_operator_attention(&report);
    let incidents = proxy_operator_incidents_from_histories(
        &report,
        recent_operator_actions,
        recent_config_events,
        history_limit,
    );

    ProxyOperatorSnapshot {
        overview,
        readiness,
        diagnostics,
        attention,
        incidents,
    }
}
