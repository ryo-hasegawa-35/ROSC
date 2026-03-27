use std::cmp::Reverse;

use rosc_telemetry::{RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction};
use serde::Serialize;

use crate::ProxyOperatorSnapshot;

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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteTimeline {
    pub route_id: String,
    pub entries: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationTimeline {
    pub destination_id: String,
    pub entries: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTimelineCatalog {
    pub global: Vec<ProxyOperatorTimelineEntry>,
    pub routes: Vec<ProxyOperatorRouteTimeline>,
    pub destinations: Vec<ProxyOperatorDestinationTimeline>,
}

pub fn proxy_operator_timeline(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorTimelineCatalog {
    let global = global_timeline_entries(snapshot);
    let routes = snapshot
        .overview
        .status
        .routes
        .iter()
        .map(|route| ProxyOperatorRouteTimeline {
            route_id: route.id.clone(),
            entries: route_timeline_entries(snapshot, route.id.as_str(), &route.destination_ids),
        })
        .collect::<Vec<_>>();
    let destinations = snapshot
        .overview
        .status
        .destinations
        .iter()
        .map(|destination| ProxyOperatorDestinationTimeline {
            destination_id: destination.id.clone(),
            entries: destination_timeline_entries(
                snapshot,
                destination.id.as_str(),
                &destination.route_ids,
            ),
        })
        .collect::<Vec<_>>();

    ProxyOperatorTimelineCatalog {
        global,
        routes,
        destinations,
    }
}

pub fn global_timeline_entries(
    snapshot: &ProxyOperatorSnapshot,
) -> Vec<ProxyOperatorTimelineEntry> {
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
    sort_timeline_entries(timeline)
}

fn route_timeline_entries(
    snapshot: &ProxyOperatorSnapshot,
    route_id: &str,
    destination_ids: &[String],
) -> Vec<ProxyOperatorTimelineEntry> {
    let mut timeline = Vec::new();
    timeline.extend(
        snapshot
            .diagnostics
            .recent_operator_actions
            .iter()
            .filter(|action| action_targets_route(action, route_id, destination_ids))
            .cloned()
            .map(timeline_entry_from_operator_action),
    );
    timeline.extend(
        snapshot
            .diagnostics
            .recent_config_events
            .iter()
            .filter(|event| config_event_targets_route(event, route_id, destination_ids))
            .cloned()
            .map(timeline_entry_from_config_event),
    );
    sort_timeline_entries(timeline)
}

fn destination_timeline_entries(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
    route_ids: &[String],
) -> Vec<ProxyOperatorTimelineEntry> {
    let mut timeline = Vec::new();
    timeline.extend(
        snapshot
            .diagnostics
            .recent_operator_actions
            .iter()
            .filter(|action| action_targets_destination(action, destination_id, route_ids))
            .cloned()
            .map(timeline_entry_from_operator_action),
    );
    timeline.extend(
        snapshot
            .diagnostics
            .recent_config_events
            .iter()
            .filter(|event| config_event_targets_destination(event, destination_id, route_ids))
            .cloned()
            .map(timeline_entry_from_config_event),
    );
    sort_timeline_entries(timeline)
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

fn sort_timeline_entries(
    mut entries: Vec<ProxyOperatorTimelineEntry>,
) -> Vec<ProxyOperatorTimelineEntry> {
    entries.sort_by_key(|entry| (Reverse(entry.recorded_at_unix_ms), entry.label.clone()));
    entries
}

fn action_targets_route(
    action: &RecentOperatorAction,
    route_id: &str,
    destination_ids: &[String],
) -> bool {
    matches_global_action(action.action.as_str())
        || detail_matches_target(&action.details, "route_id", route_id)
        || destination_ids.iter().any(|destination_id| {
            detail_matches_target(&action.details, "destination_id", destination_id)
        })
}

fn action_targets_destination(
    action: &RecentOperatorAction,
    destination_id: &str,
    route_ids: &[String],
) -> bool {
    matches_global_action(action.action.as_str())
        || detail_matches_target(&action.details, "destination_id", destination_id)
        || route_ids
            .iter()
            .any(|route_id| detail_matches_target(&action.details, "route_id", route_id))
}

fn matches_global_action(action: &str) -> bool {
    matches!(
        action,
        "freeze_traffic" | "thaw_traffic" | "restore_all_routes"
    )
}

pub(crate) fn config_event_targets_route(
    event: &RecentConfigEvent,
    route_id: &str,
    destination_ids: &[String],
) -> bool {
    event.details.iter().any(|detail| {
        detail_mentions(detail, route_id)
            || detail_matches_target_line(detail, "route_id", route_id)
            || destination_ids.iter().any(|destination_id| {
                detail_mentions(detail, destination_id)
                    || detail_matches_target_line(detail, "destination_id", destination_id)
            })
    })
}

pub(crate) fn config_event_targets_destination(
    event: &RecentConfigEvent,
    destination_id: &str,
    route_ids: &[String],
) -> bool {
    event.details.iter().any(|detail| {
        detail_mentions(detail, destination_id)
            || detail_matches_target_line(detail, "destination_id", destination_id)
            || route_ids.iter().any(|route_id| {
                detail_mentions(detail, route_id)
                    || detail_matches_target_line(detail, "route_id", route_id)
            })
    })
}

fn detail_matches_target(details: &[String], key: &str, expected: &str) -> bool {
    details
        .iter()
        .any(|detail| detail_matches_target_line(detail, key, expected))
}

fn detail_matches_target_line(detail: &str, key: &str, expected: &str) -> bool {
    detail
        .strip_prefix(&format!("{key}="))
        .map(|value| value == expected)
        .unwrap_or(false)
}

fn detail_mentions(detail: &str, expected: &str) -> bool {
    detail.contains(expected)
}

pub(crate) fn config_event_label(kind: RecentConfigEventKind) -> String {
    match kind {
        RecentConfigEventKind::Applied => "config_applied",
        RecentConfigEventKind::Rejected => "config_rejected",
        RecentConfigEventKind::Blocked => "config_blocked",
        RecentConfigEventKind::ReloadFailed => "config_reload_failed",
        RecentConfigEventKind::LaunchProfileChanged => "launch_profile_changed",
    }
    .to_owned()
}
