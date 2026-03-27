mod details;
mod traffic;

use serde::Serialize;

pub use details::{
    ProxyOperatorDestinationDetail, ProxyOperatorDestinationDetailState, ProxyOperatorRouteDetail,
    ProxyOperatorRouteDetailState,
};
pub use traffic::{ProxyOperatorCounterEntry, ProxyOperatorTrafficSummary};

use crate::{
    ProxyOperatorFocusCatalog, ProxyOperatorSnapshot, ProxyOperatorTimelineCatalog,
    ProxyOperatorTimelineEntry, ProxyOperatorTraceCatalog, ProxyRuntimeSafetyPolicy,
    UdpProxyStatusSnapshot, proxy_operator_focus_from_dashboard, proxy_operator_snapshot,
    proxy_operator_timeline, proxy_operator_trace,
};

use self::details::{destination_details_from_snapshot, route_details_from_snapshot};
use self::traffic::traffic_summary_from_runtime;

pub const DASHBOARD_REFRESH_INTERVAL_MS: u64 = 2_500;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDashboard {
    pub refresh_interval_ms: u64,
    pub snapshot: Box<ProxyOperatorSnapshot>,
    pub traffic: ProxyOperatorTrafficSummary,
    pub timeline: Vec<ProxyOperatorTimelineEntry>,
    pub timeline_catalog: ProxyOperatorTimelineCatalog,
    pub route_details: Vec<ProxyOperatorRouteDetail>,
    pub destination_details: Vec<ProxyOperatorDestinationDetail>,
    pub trace: ProxyOperatorTraceCatalog,
    pub focus: ProxyOperatorFocusCatalog,
}

pub fn proxy_operator_dashboard(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
    history_limit: Option<usize>,
) -> ProxyOperatorDashboard {
    let snapshot = proxy_operator_snapshot(status, policy, history_limit);
    proxy_operator_dashboard_from_snapshot(snapshot)
}

pub fn proxy_operator_dashboard_from_snapshot(
    snapshot: ProxyOperatorSnapshot,
) -> ProxyOperatorDashboard {
    let traffic = snapshot
        .overview
        .status
        .runtime
        .as_ref()
        .map(traffic_summary_from_runtime)
        .unwrap_or_default();
    let timeline_catalog = proxy_operator_timeline(&snapshot);
    let timeline = timeline_catalog.global.clone();
    let route_details = route_details_from_snapshot(&snapshot);
    let destination_details = destination_details_from_snapshot(&snapshot);
    let trace = proxy_operator_trace(&snapshot);

    let mut dashboard = ProxyOperatorDashboard {
        refresh_interval_ms: DASHBOARD_REFRESH_INTERVAL_MS,
        snapshot: Box::new(snapshot),
        traffic,
        timeline,
        timeline_catalog,
        route_details,
        destination_details,
        trace,
        focus: ProxyOperatorFocusCatalog::default(),
    };
    dashboard.focus = proxy_operator_focus_from_dashboard(&dashboard);
    dashboard
}
