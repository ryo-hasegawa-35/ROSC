use serde::Serialize;

use crate::{
    ProxyOperatorReport, ProxyOperatorSignalScope, ProxyOperatorSignalsView,
    ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot, proxy_operator_report,
    proxy_operator_signals_view,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorOverview {
    pub status: UdpProxyStatusSnapshot,
    pub report: ProxyOperatorReport,
    pub problematic_signals: ProxyOperatorSignalsView,
}

pub fn proxy_operator_overview(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
) -> ProxyOperatorOverview {
    let report = proxy_operator_report(status, policy);
    let problematic_signals =
        proxy_operator_signals_view(&report, ProxyOperatorSignalScope::Problematic);

    ProxyOperatorOverview {
        status: status.clone(),
        report,
        problematic_signals,
    }
}
