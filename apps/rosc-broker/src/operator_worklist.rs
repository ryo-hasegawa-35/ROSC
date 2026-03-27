use std::cmp::Reverse;
use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    ProxyOperatorDestinationSignal, ProxyOperatorRouteSignal, ProxyOperatorSnapshot,
    ProxyOperatorState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorWorkItemLevel {
    Blocked,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorSuggestedActionKind {
    ThawTraffic,
    RestoreRoute,
    RehydrateDestination,
    FocusRoute,
    FocusDestination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorSuggestedAction {
    pub kind: ProxyOperatorSuggestedActionKind,
    pub label: String,
    pub route_id: Option<String>,
    pub destination_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorWorkItem {
    pub id: String,
    pub level: ProxyOperatorWorkItemLevel,
    pub title: String,
    pub summary: String,
    pub reasons: Vec<String>,
    pub action: Option<ProxyOperatorSuggestedAction>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorWorklist {
    pub state: String,
    pub immediate_actions: usize,
    pub recovery_actions: usize,
    pub items: Vec<ProxyOperatorWorkItem>,
}

pub fn proxy_operator_worklist(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorWorklist {
    let route_signals = snapshot
        .overview
        .report
        .route_signals
        .iter()
        .map(|signal| (signal.route_id.as_str(), signal))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();

    if !snapshot.overview.report.blockers.is_empty() {
        items.push(ProxyOperatorWorkItem {
            id: "config-blockers".to_owned(),
            level: ProxyOperatorWorkItemLevel::Blocked,
            title: "Resolve config blockers".to_owned(),
            summary: format!(
                "{} blocker(s) currently prevent a clean ready state.",
                snapshot.overview.report.blockers.len()
            ),
            reasons: snapshot.overview.report.blockers.clone(),
            action: None,
        });
    }

    if snapshot.overview.report.overrides.traffic_frozen {
        items.push(ProxyOperatorWorkItem {
            id: "traffic-frozen".to_owned(),
            level: ProxyOperatorWorkItemLevel::Degraded,
            title: "Traffic is currently frozen".to_owned(),
            summary: "Live dispatch is paused until the operator thaws traffic.".to_owned(),
            reasons: vec![
                "traffic is currently frozen by operator override".to_owned(),
                "queued packets will remain held until thaw".to_owned(),
            ],
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::ThawTraffic,
                label: "Thaw traffic".to_owned(),
                route_id: None,
                destination_id: None,
            }),
        });
    }

    for route_id in &snapshot.overview.report.overrides.isolated_route_ids {
        let signal = route_signals.get(route_id.as_str()).copied();
        let reasons = route_reasons(signal);
        items.push(ProxyOperatorWorkItem {
            id: format!("route:{route_id}:restore"),
            level: ProxyOperatorWorkItemLevel::Degraded,
            title: format!("Restore isolated route `{route_id}`"),
            summary: "This route is isolated and not forwarding live traffic.".to_owned(),
            reasons,
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::RestoreRoute,
                label: "Restore route".to_owned(),
                route_id: Some(route_id.clone()),
                destination_id: None,
            }),
        });
    }

    for signal in snapshot
        .incidents
        .problematic_routes
        .iter()
        .filter(|signal| !signal.isolated)
    {
        items.push(ProxyOperatorWorkItem {
            id: format!("route:{}:focus", signal.route_id),
            level: if signal.direct_udp_fallback_available {
                ProxyOperatorWorkItemLevel::Degraded
            } else {
                ProxyOperatorWorkItemLevel::Blocked
            },
            title: format!("Inspect route `{}`", signal.route_id),
            summary: "This route is reporting operator-visible issues and should be reviewed."
                .to_owned(),
            reasons: route_reasons(Some(signal)),
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::FocusRoute,
                label: "Focus route".to_owned(),
                route_id: Some(signal.route_id.clone()),
                destination_id: None,
            }),
        });
    }

    for signal in &snapshot.incidents.problematic_destinations {
        items.push(ProxyOperatorWorkItem {
            id: format!("destination:{}:rehydrate", signal.destination_id),
            level: if matches!(
                signal.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::Open)
            ) {
                ProxyOperatorWorkItemLevel::Blocked
            } else {
                ProxyOperatorWorkItemLevel::Degraded
            },
            title: format!("Recover destination `{}`", signal.destination_id),
            summary: "This destination has queue, breaker, or send-failure pressure.".to_owned(),
            reasons: destination_reasons(signal),
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::RehydrateDestination,
                label: "Rehydrate destination".to_owned(),
                route_id: None,
                destination_id: Some(signal.destination_id.clone()),
            }),
        });
    }

    if items.is_empty() {
        for route in snapshot
            .overview
            .report
            .route_signals
            .iter()
            .filter(|signal| signal.is_problematic())
        {
            items.push(ProxyOperatorWorkItem {
                id: format!("route:{}:focus", route.route_id),
                level: ProxyOperatorWorkItemLevel::Degraded,
                title: format!("Inspect route `{}`", route.route_id),
                summary: "This route still deserves a closer look.".to_owned(),
                reasons: route_reasons(Some(route)),
                action: Some(ProxyOperatorSuggestedAction {
                    kind: ProxyOperatorSuggestedActionKind::FocusRoute,
                    label: "Focus route".to_owned(),
                    route_id: Some(route.route_id.clone()),
                    destination_id: None,
                }),
            });
        }
        if items.is_empty() {
            for destination in snapshot
                .overview
                .report
                .destination_signals
                .iter()
                .filter(|signal| signal.is_problematic())
            {
                items.push(ProxyOperatorWorkItem {
                    id: format!("destination:{}:focus", destination.destination_id),
                    level: ProxyOperatorWorkItemLevel::Degraded,
                    title: format!("Inspect destination `{}`", destination.destination_id),
                    summary: "This destination still deserves a closer look.".to_owned(),
                    reasons: destination_reasons(destination),
                    action: Some(ProxyOperatorSuggestedAction {
                        kind: ProxyOperatorSuggestedActionKind::FocusDestination,
                        label: "Focus destination".to_owned(),
                        route_id: None,
                        destination_id: Some(destination.destination_id.clone()),
                    }),
                });
            }
        }
    }

    dedupe_items(&mut items);
    items.sort_by_key(|item| (item.level, Reverse(item.reasons.len()), item.id.clone()));

    ProxyOperatorWorklist {
        state: worklist_state(snapshot.overview.report.state.clone()).to_owned(),
        immediate_actions: items
            .iter()
            .filter(|item| {
                matches!(
                    item.action.as_ref().map(|action| &action.kind),
                    Some(
                        ProxyOperatorSuggestedActionKind::ThawTraffic
                            | ProxyOperatorSuggestedActionKind::RestoreRoute
                            | ProxyOperatorSuggestedActionKind::RehydrateDestination
                    )
                )
            })
            .count(),
        recovery_actions: items.len(),
        items,
    }
}

fn dedupe_items(items: &mut Vec<ProxyOperatorWorkItem>) {
    let mut by_id = BTreeMap::new();
    for item in items.drain(..) {
        by_id.entry(item.id.clone()).or_insert(item);
    }
    items.extend(by_id.into_values());
}

fn worklist_state(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}

fn route_reasons(signal: Option<&ProxyOperatorRouteSignal>) -> Vec<String> {
    let Some(signal) = signal else {
        return vec!["No route-specific signal details are available.".to_owned()];
    };
    let mut reasons = Vec::new();
    if signal.isolated {
        reasons.push("route is currently isolated".to_owned());
    }
    if !signal.direct_udp_fallback_available {
        reasons.push("route is missing direct UDP fallback coverage".to_owned());
    }
    reasons.extend(signal.config_warnings.iter().cloned());
    if signal.dispatch_failures_total > 0 {
        reasons.push(format!(
            "dispatch failures observed ({})",
            signal.dispatch_failures_total
        ));
    }
    if signal.transform_failures_total > 0 {
        reasons.push(format!(
            "transform failures observed ({})",
            signal.transform_failures_total
        ));
    }
    if reasons.is_empty() {
        reasons.push("route is flagged for review by operator policy".to_owned());
    }
    reasons
}

fn destination_reasons(signal: &ProxyOperatorDestinationSignal) -> Vec<String> {
    let mut reasons = Vec::new();
    if signal.queue_depth > 0 {
        reasons.push(format!("queue backlog observed ({})", signal.queue_depth));
    }
    if signal.send_failures_total > 0 {
        reasons.push(format!(
            "send failures observed ({})",
            signal.send_failures_total
        ));
    }
    if signal.drops_total > 0 {
        reasons.push(format!("drops observed ({})", signal.drops_total));
    }
    match signal.breaker_state {
        Some(rosc_telemetry::BreakerStateSnapshot::Open) => {
            reasons.push("destination breaker is open".to_owned())
        }
        Some(rosc_telemetry::BreakerStateSnapshot::HalfOpen) => {
            reasons.push("destination breaker is half-open".to_owned())
        }
        Some(rosc_telemetry::BreakerStateSnapshot::Closed) => {}
        None => {}
    }
    if reasons.is_empty() {
        reasons.push("destination is flagged for review by operator policy".to_owned());
    }
    reasons
}
