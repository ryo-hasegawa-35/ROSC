use serde::Serialize;

use crate::{
    ProxyOperatorReport, ProxyOperatorRuntimeSummary, ProxyOperatorSignalScope,
    ProxyOperatorSignalsView, ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot,
    proxy_operator_report, proxy_operator_runtime_summary, proxy_operator_signals_view,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorOverview {
    pub status: UdpProxyStatusSnapshot,
    pub report: ProxyOperatorReport,
    pub runtime_summary: ProxyOperatorRuntimeSummary,
    pub problematic_signals: ProxyOperatorSignalsView,
}

pub fn proxy_operator_overview(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
) -> ProxyOperatorOverview {
    let report = proxy_operator_report(status, policy);
    let problematic_signals =
        proxy_operator_signals_view(&report, ProxyOperatorSignalScope::Problematic);
    let mut overview = ProxyOperatorOverview {
        status: status.clone(),
        report,
        runtime_summary: ProxyOperatorRuntimeSummary::default(),
        problematic_signals,
    };
    overview.runtime_summary = proxy_operator_runtime_summary(&overview);
    overview
}
