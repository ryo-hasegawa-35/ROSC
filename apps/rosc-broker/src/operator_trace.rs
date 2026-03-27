use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use rosc_telemetry::{
    BreakerStateSnapshot, RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction,
};
use serde::Serialize;

use crate::{
    ProxyOperatorSnapshot, ProxyOperatorSuggestedAction, ProxyOperatorSuggestedActionKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorTraceEventKind {
    RuntimeSignal,
    OperatorAction,
    ConfigEvent,
    Override,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorTraceEventLevel {
    Info,
    Degraded,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTraceEvent {
    pub kind: ProxyOperatorTraceEventKind,
    pub level: ProxyOperatorTraceEventLevel,
    pub title: String,
    pub summary: String,
    pub recorded_at_unix_ms: Option<u64>,
    pub details: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteTrace {
    pub route_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub related_destination_ids: Vec<String>,
    pub direct_udp_targets: Vec<String>,
    pub open_reasons: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationTrace {
    pub destination_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub route_ids: Vec<String>,
    pub target: String,
    pub open_reasons: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTraceCatalog {
    pub routes: Vec<ProxyOperatorRouteTrace>,
    pub destinations: Vec<ProxyOperatorDestinationTrace>,
}

pub fn proxy_operator_trace(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorTraceCatalog {
    let route_statuses = snapshot
        .overview
        .status
        .routes
        .iter()
        .map(|route| (route.id.as_str(), route))
        .collect::<BTreeMap<_, _>>();
    let route_assessments = snapshot
        .overview
        .status
        .route_assessments
        .iter()
        .map(|assessment| (assessment.route_id.as_str(), assessment))
        .collect::<BTreeMap<_, _>>();
    let route_fallbacks = snapshot
        .overview
        .status
        .fallback_routes
        .iter()
        .map(|fallback| (fallback.route_id.as_str(), fallback))
        .collect::<BTreeMap<_, _>>();
    let route_signals = snapshot
        .overview
        .report
        .route_signals
        .iter()
        .map(|signal| (signal.route_id.as_str(), signal))
        .collect::<BTreeMap<_, _>>();
    let destination_statuses = snapshot
        .overview
        .status
        .destinations
        .iter()
        .map(|destination| (destination.id.as_str(), destination))
        .collect::<BTreeMap<_, _>>();
    let destination_runtime = snapshot
        .overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .destinations
                .iter()
                .map(|destination| (destination.destination_id.as_str(), destination))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let destination_signals = snapshot
        .overview
        .report
        .destination_signals
        .iter()
        .map(|signal| (signal.destination_id.as_str(), signal))
        .collect::<BTreeMap<_, _>>();

    let routes = snapshot
        .overview
        .status
        .routes
        .iter()
        .map(|route| {
            let signal = route_signals.get(route.id.as_str()).copied();
            let assessment = route_assessments.get(route.id.as_str()).copied();
            let fallback = route_fallbacks.get(route.id.as_str()).copied();
            let isolated = snapshot
                .overview
                .report
                .overrides
                .isolated_route_ids
                .contains(&route.id);
            let open_reasons =
                route_open_reasons(route.id.as_str(), isolated, signal, assessment, fallback);
            let level = route_level(route.enabled, isolated, signal, fallback);
            let actions = route_actions(route.id.as_str(), isolated);
            let recent_events = route_events(
                snapshot,
                route.id.as_str(),
                &route.destination_ids,
                isolated,
                &open_reasons,
            );

            ProxyOperatorRouteTrace {
                route_id: route.id.clone(),
                level,
                summary: route_summary(route.id.as_str(), level, isolated),
                related_destination_ids: route.destination_ids.clone(),
                direct_udp_targets: fallback
                    .map(|entry| entry.direct_udp_targets.clone())
                    .unwrap_or_default(),
                open_reasons,
                actions,
                recent_events,
            }
        })
        .collect::<Vec<_>>();

    let destinations = snapshot
        .overview
        .status
        .destinations
        .iter()
        .map(|destination| {
            let runtime = destination_runtime.get(destination.id.as_str()).copied();
            let signal = destination_signals.get(destination.id.as_str()).copied();
            let open_reasons = destination_open_reasons(runtime, signal);
            let level = destination_level(runtime, signal);
            let actions = destination_actions(destination.id.as_str(), level);
            let recent_events = destination_events(
                snapshot,
                destination.id.as_str(),
                &destination.route_ids,
                &open_reasons,
            );

            ProxyOperatorDestinationTrace {
                destination_id: destination.id.clone(),
                level,
                summary: destination_summary(destination.id.as_str(), level),
                route_ids: destination.route_ids.clone(),
                target: destination.target.clone(),
                open_reasons,
                actions,
                recent_events,
            }
        })
        .collect::<Vec<_>>();

    let _ = route_statuses;
    let _ = destination_statuses;

    ProxyOperatorTraceCatalog {
        routes,
        destinations,
    }
}

fn route_summary(route_id: &str, level: ProxyOperatorTraceEventLevel, isolated: bool) -> String {
    if isolated {
        return format!("Route `{route_id}` is isolated and live forwarding is paused.");
    }
    match level {
        ProxyOperatorTraceEventLevel::Blocked => {
            format!("Route `{route_id}` is blocked and needs recovery coverage.")
        }
        ProxyOperatorTraceEventLevel::Degraded => {
            format!("Route `{route_id}` is degraded and needs operator attention.")
        }
        ProxyOperatorTraceEventLevel::Info => {
            format!("Route `{route_id}` is currently stable.")
        }
    }
}

fn destination_summary(destination_id: &str, level: ProxyOperatorTraceEventLevel) -> String {
    match level {
        ProxyOperatorTraceEventLevel::Blocked => {
            format!(
                "Destination `{destination_id}` is blocked by breaker or repeated send failure."
            )
        }
        ProxyOperatorTraceEventLevel::Degraded => {
            format!("Destination `{destination_id}` is under queue or send pressure.")
        }
        ProxyOperatorTraceEventLevel::Info => {
            format!("Destination `{destination_id}` is currently stable.")
        }
    }
}

fn route_level(
    enabled: bool,
    isolated: bool,
    signal: Option<&crate::ProxyOperatorRouteSignal>,
    fallback: Option<&crate::UdpProxyFallbackStatus>,
) -> ProxyOperatorTraceEventLevel {
    if !enabled {
        return ProxyOperatorTraceEventLevel::Blocked;
    }
    if isolated {
        return ProxyOperatorTraceEventLevel::Degraded;
    }
    if !fallback.map(|entry| entry.available).unwrap_or(false) {
        return ProxyOperatorTraceEventLevel::Blocked;
    }
    if let Some(signal) = signal
        && (signal.dispatch_failures_total > 0
            || signal.transform_failures_total > 0
            || !signal.config_warnings.is_empty())
    {
        return ProxyOperatorTraceEventLevel::Degraded;
    }
    ProxyOperatorTraceEventLevel::Info
}

fn destination_level(
    runtime: Option<&crate::UdpProxyDestinationRuntimeStatus>,
    signal: Option<&crate::ProxyOperatorDestinationSignal>,
) -> ProxyOperatorTraceEventLevel {
    if matches!(
        runtime.and_then(|entry| entry.breaker_state.as_ref()),
        Some(BreakerStateSnapshot::Open)
    ) {
        return ProxyOperatorTraceEventLevel::Blocked;
    }
    if runtime.map(|entry| entry.queue_depth > 0).unwrap_or(false)
        || signal
            .map(|entry| entry.send_failures_total > 0)
            .unwrap_or(false)
        || signal.map(|entry| entry.drops_total > 0).unwrap_or(false)
        || matches!(
            runtime.and_then(|entry| entry.breaker_state.as_ref()),
            Some(BreakerStateSnapshot::HalfOpen)
        )
    {
        return ProxyOperatorTraceEventLevel::Degraded;
    }
    ProxyOperatorTraceEventLevel::Info
}

fn route_actions(route_id: &str, isolated: bool) -> Vec<ProxyOperatorSuggestedAction> {
    if !isolated {
        return Vec::new();
    }
    vec![ProxyOperatorSuggestedAction {
        kind: ProxyOperatorSuggestedActionKind::RestoreRoute,
        label: "Restore route".to_owned(),
        route_id: Some(route_id.to_owned()),
        destination_id: None,
    }]
}

fn destination_actions(
    destination_id: &str,
    level: ProxyOperatorTraceEventLevel,
) -> Vec<ProxyOperatorSuggestedAction> {
    if level == ProxyOperatorTraceEventLevel::Info {
        return Vec::new();
    }
    vec![ProxyOperatorSuggestedAction {
        kind: ProxyOperatorSuggestedActionKind::RehydrateDestination,
        label: "Rehydrate destination".to_owned(),
        route_id: None,
        destination_id: Some(destination_id.to_owned()),
    }]
}

fn route_open_reasons(
    route_id: &str,
    isolated: bool,
    signal: Option<&crate::ProxyOperatorRouteSignal>,
    assessment: Option<&crate::UdpProxyRouteAssessment>,
    fallback: Option<&crate::UdpProxyFallbackStatus>,
) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    if isolated {
        reasons.insert("operator isolation active".to_owned());
    }
    if !fallback.map(|entry| entry.available).unwrap_or(false) {
        reasons.insert("direct UDP fallback is missing".to_owned());
    }
    if let Some(assessment) = assessment {
        reasons.extend(assessment.warnings.iter().cloned());
    }
    if let Some(signal) = signal {
        reasons.extend(signal.config_warnings.iter().cloned());
        if signal.dispatch_failures_total > 0 {
            reasons.insert(format!(
                "dispatch failures observed ({})",
                signal.dispatch_failures_total
            ));
        }
        if signal.transform_failures_total > 0 {
            reasons.insert(format!(
                "transform failures observed ({})",
                signal.transform_failures_total
            ));
        }
    }
    let mut reasons = reasons.into_iter().collect::<Vec<_>>();
    if reasons.is_empty() {
        reasons.push(format!("route `{route_id}` has no active trace blockers"));
    }
    reasons
}

fn destination_open_reasons(
    runtime: Option<&crate::UdpProxyDestinationRuntimeStatus>,
    signal: Option<&crate::ProxyOperatorDestinationSignal>,
) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    if let Some(runtime) = runtime {
        if runtime.queue_depth > 0 {
            reasons.insert(format!("queue backlog observed ({})", runtime.queue_depth));
        }
        if let Some(state) = runtime.breaker_state.as_ref() {
            match state {
                BreakerStateSnapshot::Open => {
                    reasons.insert("breaker is open".to_owned());
                }
                BreakerStateSnapshot::HalfOpen => {
                    reasons.insert("breaker is half-open".to_owned());
                }
                BreakerStateSnapshot::Closed => {}
            }
        }
    }
    if let Some(signal) = signal {
        if signal.send_failures_total > 0 {
            reasons.insert(format!(
                "send failures observed ({})",
                signal.send_failures_total
            ));
        }
        if signal.drops_total > 0 {
            reasons.insert(format!("drops observed ({})", signal.drops_total));
        }
    }
    let mut reasons = reasons.into_iter().collect::<Vec<_>>();
    if reasons.is_empty() {
        reasons.push("destination is currently stable".to_owned());
    }
    reasons
}

fn route_events(
    snapshot: &ProxyOperatorSnapshot,
    route_id: &str,
    destination_ids: &[String],
    isolated: bool,
    open_reasons: &[String],
) -> Vec<ProxyOperatorTraceEvent> {
    let mut events = Vec::new();
    if isolated
        || open_reasons
            .iter()
            .any(|reason| !reason.contains("no active trace blockers"))
    {
        events.push(ProxyOperatorTraceEvent {
            kind: if isolated {
                ProxyOperatorTraceEventKind::Override
            } else {
                ProxyOperatorTraceEventKind::RuntimeSignal
            },
            level: if open_reasons
                .iter()
                .any(|reason| reason.contains("fallback is missing"))
            {
                ProxyOperatorTraceEventLevel::Blocked
            } else {
                ProxyOperatorTraceEventLevel::Degraded
            },
            title: if isolated {
                "Operator override".to_owned()
            } else {
                "Runtime route signal".to_owned()
            },
            summary: open_reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "route requires inspection".to_owned()),
            recorded_at_unix_ms: None,
            details: open_reasons.to_vec(),
        });
    }
    if snapshot.overview.report.overrides.traffic_frozen {
        events.push(ProxyOperatorTraceEvent {
            kind: ProxyOperatorTraceEventKind::Override,
            level: ProxyOperatorTraceEventLevel::Degraded,
            title: "Traffic override".to_owned(),
            summary: "Traffic is frozen, so live dispatch is paused across all routes.".to_owned(),
            recorded_at_unix_ms: None,
            details: vec!["traffic_frozen=true".to_owned()],
        });
    }
    events.extend(
        snapshot
            .diagnostics
            .recent_operator_actions
            .iter()
            .filter(|action| action_targets_route(action, route_id, destination_ids))
            .cloned()
            .map(operator_action_event),
    );
    events.extend(
        snapshot
            .diagnostics
            .recent_config_events
            .iter()
            .filter(|event| config_event_targets_route(event, route_id))
            .cloned()
            .map(config_event_trace),
    );
    sort_trace_events(events)
}

fn destination_events(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
    route_ids: &[String],
    open_reasons: &[String],
) -> Vec<ProxyOperatorTraceEvent> {
    let mut events = Vec::new();
    if open_reasons
        .iter()
        .any(|reason| !reason.contains("currently stable"))
    {
        events.push(ProxyOperatorTraceEvent {
            kind: ProxyOperatorTraceEventKind::RuntimeSignal,
            level: if open_reasons
                .iter()
                .any(|reason| reason.contains("breaker is open"))
            {
                ProxyOperatorTraceEventLevel::Blocked
            } else {
                ProxyOperatorTraceEventLevel::Degraded
            },
            title: "Runtime destination signal".to_owned(),
            summary: open_reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "destination requires inspection".to_owned()),
            recorded_at_unix_ms: None,
            details: open_reasons.to_vec(),
        });
    }
    if snapshot.overview.report.overrides.traffic_frozen {
        events.push(ProxyOperatorTraceEvent {
            kind: ProxyOperatorTraceEventKind::Override,
            level: ProxyOperatorTraceEventLevel::Degraded,
            title: "Traffic override".to_owned(),
            summary:
                "Traffic is frozen, so destination send pressure may remain static until thaw."
                    .to_owned(),
            recorded_at_unix_ms: None,
            details: vec!["traffic_frozen=true".to_owned()],
        });
    }
    events.extend(
        snapshot
            .diagnostics
            .recent_operator_actions
            .iter()
            .filter(|action| action_targets_destination(action, destination_id, route_ids))
            .cloned()
            .map(operator_action_event),
    );
    events.extend(
        snapshot
            .diagnostics
            .recent_config_events
            .iter()
            .filter(|event| config_event_targets_destination(event, destination_id))
            .cloned()
            .map(config_event_trace),
    );
    sort_trace_events(events)
}

fn sort_trace_events(mut events: Vec<ProxyOperatorTraceEvent>) -> Vec<ProxyOperatorTraceEvent> {
    let mut live = Vec::new();
    let mut historical = Vec::new();
    for event in events.drain(..) {
        if event.recorded_at_unix_ms.is_some() {
            historical.push(event);
        } else {
            live.push(event);
        }
    }
    historical.sort_by_key(|event| {
        (
            Reverse(event.recorded_at_unix_ms.unwrap_or_default()),
            event.title.clone(),
        )
    });
    live.extend(historical);
    live
}

fn operator_action_event(action: RecentOperatorAction) -> ProxyOperatorTraceEvent {
    ProxyOperatorTraceEvent {
        kind: ProxyOperatorTraceEventKind::OperatorAction,
        level: ProxyOperatorTraceEventLevel::Info,
        title: action.action.replace('_', " "),
        summary: "Recent operator intervention affecting this entity.".to_owned(),
        recorded_at_unix_ms: Some(action.recorded_at_unix_ms),
        details: action.details,
    }
}

fn config_event_trace(event: RecentConfigEvent) -> ProxyOperatorTraceEvent {
    let mut details = event.details;
    if let Some(revision) = event.revision {
        details.insert(0, format!("revision={revision}"));
    }
    if let Some(mode) = event.launch_profile_mode {
        details.push(format!("launch_profile={mode}"));
    }
    ProxyOperatorTraceEvent {
        kind: ProxyOperatorTraceEventKind::ConfigEvent,
        level: match event.kind {
            RecentConfigEventKind::Blocked | RecentConfigEventKind::ReloadFailed => {
                ProxyOperatorTraceEventLevel::Blocked
            }
            RecentConfigEventKind::Rejected | RecentConfigEventKind::LaunchProfileChanged => {
                ProxyOperatorTraceEventLevel::Degraded
            }
            RecentConfigEventKind::Applied => ProxyOperatorTraceEventLevel::Info,
        },
        title: format!("config {}", config_event_label(event.kind)),
        summary: "Recent config transition related to this entity.".to_owned(),
        recorded_at_unix_ms: Some(event.recorded_at_unix_ms),
        details,
    }
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

fn config_event_targets_route(event: &RecentConfigEvent, route_id: &str) -> bool {
    event.details.iter().any(|detail| {
        detail_mentions(detail, route_id)
            || detail_matches_target_line(detail, "route_id", route_id)
    }) || event.changed_routes > 0
        && matches!(
            event.kind,
            RecentConfigEventKind::Blocked
                | RecentConfigEventKind::Rejected
                | RecentConfigEventKind::ReloadFailed
                | RecentConfigEventKind::LaunchProfileChanged
        )
}

fn config_event_targets_destination(event: &RecentConfigEvent, destination_id: &str) -> bool {
    event.details.iter().any(|detail| {
        detail_mentions(detail, destination_id)
            || detail_matches_target_line(detail, "destination_id", destination_id)
    }) || event.changed_destinations > 0
        && matches!(
            event.kind,
            RecentConfigEventKind::Blocked
                | RecentConfigEventKind::Rejected
                | RecentConfigEventKind::ReloadFailed
                | RecentConfigEventKind::LaunchProfileChanged
        )
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

fn config_event_label(kind: RecentConfigEventKind) -> &'static str {
    match kind {
        RecentConfigEventKind::Applied => "applied",
        RecentConfigEventKind::Rejected => "rejected",
        RecentConfigEventKind::Blocked => "blocked",
        RecentConfigEventKind::ReloadFailed => "reload_failed",
        RecentConfigEventKind::LaunchProfileChanged => "launch_profile_changed",
    }
}
