mod details;
mod traffic;

use std::cmp::Reverse;

use rosc_telemetry::{RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction};
use serde::Serialize;

pub use details::{
    ProxyOperatorDestinationDetail, ProxyOperatorDestinationDetailState, ProxyOperatorRouteDetail,
    ProxyOperatorRouteDetailState,
};
pub use traffic::{ProxyOperatorCounterEntry, ProxyOperatorTrafficSummary};

use crate::{
    ProxyOperatorSnapshot, ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot,
    proxy_operator_snapshot,
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
    pub route_details: Vec<ProxyOperatorRouteDetail>,
    pub destination_details: Vec<ProxyOperatorDestinationDetail>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorTimelineCategory {
    OperatorAction,
    ConfigEvent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTimelineEntry {
    pub category: ProxyOperatorTimelineCategory,
    pub label: String,
    pub recorded_at_unix_ms: u64,
    pub details: Vec<String>,
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
    let timeline = timeline_from_snapshot(&snapshot);
    let route_details = route_details_from_snapshot(&snapshot);
    let destination_details = destination_details_from_snapshot(&snapshot);

    ProxyOperatorDashboard {
        refresh_interval_ms: DASHBOARD_REFRESH_INTERVAL_MS,
        snapshot: Box::new(snapshot),
        traffic,
        timeline,
        route_details,
        destination_details,
    }
}

fn timeline_from_snapshot(snapshot: &ProxyOperatorSnapshot) -> Vec<ProxyOperatorTimelineEntry> {
    let mut timeline = Vec::new();

    timeline.extend(
        snapshot
            .diagnostics
            .recent_operator_actions
            .iter()
            .cloned()
            .map(timeline_entry_from_operator_action),
    );
    timeline.extend(
        snapshot
            .diagnostics
            .recent_config_events
            .iter()
            .cloned()
            .map(timeline_entry_from_config_event),
    );

    timeline.sort_by_key(|entry| (Reverse(entry.recorded_at_unix_ms), entry.label.clone()));
    timeline
}

fn timeline_entry_from_operator_action(action: RecentOperatorAction) -> ProxyOperatorTimelineEntry {
    ProxyOperatorTimelineEntry {
        category: ProxyOperatorTimelineCategory::OperatorAction,
        label: action.action,
        recorded_at_unix_ms: action.recorded_at_unix_ms,
        details: action.details,
    }
}

fn timeline_entry_from_config_event(event: RecentConfigEvent) -> ProxyOperatorTimelineEntry {
    let mut details = event.details;
    if let Some(revision) = event.revision {
        details.insert(0, format!("revision={revision}"));
    }
    if let Some(mode) = event.launch_profile_mode {
        details.push(format!("launch_profile={mode}"));
    }

    ProxyOperatorTimelineEntry {
        category: ProxyOperatorTimelineCategory::ConfigEvent,
        label: config_event_label(event.kind),
        recorded_at_unix_ms: event.recorded_at_unix_ms,
        details,
    }
}

fn config_event_label(kind: RecentConfigEventKind) -> String {
    match kind {
        RecentConfigEventKind::Applied => "config_applied",
        RecentConfigEventKind::Rejected => "config_rejected",
        RecentConfigEventKind::Blocked => "config_blocked",
        RecentConfigEventKind::ReloadFailed => "config_reload_failed",
        RecentConfigEventKind::LaunchProfileChanged => "launch_profile_changed",
    }
    .to_owned()
}
