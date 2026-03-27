use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::{
    ProxyOperatorOverview, ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot,
    operator_history::bounded_recent_entries, proxy_operator_overview,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDiagnostics {
    pub overview: ProxyOperatorOverview,
    pub recent_operator_actions: Vec<RecentOperatorAction>,
    pub recent_config_events: Vec<RecentConfigEvent>,
}

pub fn proxy_operator_diagnostics(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
    history_limit: Option<usize>,
) -> ProxyOperatorDiagnostics {
    let overview = proxy_operator_overview(status, policy);
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
    proxy_operator_diagnostics_from_overview(
        overview,
        recent_operator_actions,
        recent_config_events,
        history_limit,
    )
}

pub fn proxy_operator_diagnostics_from_overview(
    overview: ProxyOperatorOverview,
    recent_operator_actions: Vec<RecentOperatorAction>,
    recent_config_events: Vec<RecentConfigEvent>,
    history_limit: Option<usize>,
) -> ProxyOperatorDiagnostics {
    ProxyOperatorDiagnostics {
        recent_operator_actions: bounded_recent_entries(recent_operator_actions, history_limit),
        recent_config_events: bounded_recent_entries(recent_config_events, history_limit),
        overview,
    }
}
