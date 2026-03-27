use serde::Serialize;

use crate::{
    ProxyOperatorIncidentCluster, ProxyOperatorIncidentScope, ProxyOperatorSnapshot,
    ProxyOperatorSuggestedAction, ProxyOperatorTimelineEntry, ProxyOperatorTraceEvent,
    ProxyOperatorTraceEventLevel, proxy_operator_handoff, proxy_operator_timeline,
    proxy_operator_trace, proxy_operator_triage,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteCasebook {
    pub route_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub linked_destination_ids: Vec<String>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub recovery_surface: Vec<String>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
    pub timeline: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationCasebook {
    pub destination_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub linked_route_ids: Vec<String>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub recovery_surface: Vec<String>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
    pub timeline: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorCasebookCatalog {
    pub state: String,
    pub route_casebooks: Vec<ProxyOperatorRouteCasebook>,
    pub destination_casebooks: Vec<ProxyOperatorDestinationCasebook>,
}

pub fn proxy_operator_casebook(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorCasebookCatalog {
    let trace = proxy_operator_trace(snapshot);
    let handoff = if snapshot.handoff.route_handoffs.is_empty()
        && snapshot.handoff.destination_handoffs.is_empty()
    {
        proxy_operator_handoff(snapshot)
    } else {
        snapshot.handoff.clone()
    };
    let triage = if snapshot.triage.route_triage.is_empty()
        && snapshot.triage.destination_triage.is_empty()
    {
        proxy_operator_triage(snapshot)
    } else {
        snapshot.triage.clone()
    };
    let timeline = proxy_operator_timeline(snapshot);

    let route_casebooks = trace
        .routes
        .iter()
        .map(|route_trace| {
            let handoff_entry = handoff
                .route_handoffs
                .iter()
                .find(|entry| entry.route_id == route_trace.route_id);
            let triage_entry = triage
                .route_triage
                .iter()
                .find(|entry| entry.route_id == route_trace.route_id);
            let timeline_entries = timeline
                .routes
                .iter()
                .find(|entry| entry.route_id == route_trace.route_id)
                .map(|entry| entry.entries.clone())
                .unwrap_or_default();
            let incident_titles = route_incident_titles(snapshot, &route_trace.route_id);
            let recovery_surface = route_recovery_surface(snapshot, &route_trace.route_id);

            ProxyOperatorRouteCasebook {
                route_id: route_trace.route_id.clone(),
                level: route_trace.level,
                summary: casebook_summary(
                    &route_trace.summary,
                    handoff_entry.map(|entry| entry.summary.as_str()),
                ),
                linked_destination_ids: route_trace.related_destination_ids.clone(),
                incident_titles,
                next_steps: merge_string_sections(
                    handoff_entry.map(|entry| entry.next_steps.as_slice()),
                    triage_entry.map(|entry| entry.next_steps.as_slice()),
                ),
                recommended_actions: merge_actions(
                    route_trace.actions.as_slice(),
                    handoff_entry.map(|entry| entry.actions.as_slice()),
                    triage_entry.map(|entry| entry.actions.as_slice()),
                ),
                recovery_surface,
                recent_events: merge_trace_events(
                    route_trace.recent_events.as_slice(),
                    handoff_entry.map(|entry| entry.recent_events.as_slice()),
                    triage_entry.map(|entry| entry.recent_events.as_slice()),
                ),
                timeline: timeline_entries,
            }
        })
        .collect::<Vec<_>>();

    let destination_casebooks = trace
        .destinations
        .iter()
        .map(|destination_trace| {
            let handoff_entry = handoff
                .destination_handoffs
                .iter()
                .find(|entry| entry.destination_id == destination_trace.destination_id);
            let triage_entry = triage
                .destination_triage
                .iter()
                .find(|entry| entry.destination_id == destination_trace.destination_id);
            let timeline_entries = timeline
                .destinations
                .iter()
                .find(|entry| entry.destination_id == destination_trace.destination_id)
                .map(|entry| entry.entries.clone())
                .unwrap_or_default();
            let incident_titles =
                destination_incident_titles(snapshot, &destination_trace.destination_id);
            let recovery_surface =
                destination_recovery_surface(snapshot, &destination_trace.destination_id);

            ProxyOperatorDestinationCasebook {
                destination_id: destination_trace.destination_id.clone(),
                level: destination_trace.level,
                summary: casebook_summary(
                    &destination_trace.summary,
                    handoff_entry.map(|entry| entry.summary.as_str()),
                ),
                linked_route_ids: destination_trace.route_ids.clone(),
                incident_titles,
                next_steps: merge_string_sections(
                    handoff_entry.map(|entry| entry.next_steps.as_slice()),
                    triage_entry.map(|entry| entry.next_steps.as_slice()),
                ),
                recommended_actions: merge_actions(
                    destination_trace.actions.as_slice(),
                    handoff_entry.map(|entry| entry.actions.as_slice()),
                    triage_entry.map(|entry| entry.actions.as_slice()),
                ),
                recovery_surface,
                recent_events: merge_trace_events(
                    destination_trace.recent_events.as_slice(),
                    handoff_entry.map(|entry| entry.recent_events.as_slice()),
                    triage_entry.map(|entry| entry.recent_events.as_slice()),
                ),
                timeline: timeline_entries,
            }
        })
        .collect::<Vec<_>>();

    ProxyOperatorCasebookCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        route_casebooks,
        destination_casebooks,
    }
}

fn casebook_summary(primary: &str, secondary: Option<&str>) -> String {
    match secondary {
        Some(secondary) if secondary != primary => format!("{primary} {secondary}"),
        _ => primary.to_owned(),
    }
}

fn route_incident_titles(snapshot: &ProxyOperatorSnapshot, route_id: &str) -> Vec<String> {
    incident_titles(snapshot, |cluster| {
        cluster.route_id.as_deref() == Some(route_id)
            || matches!(
                cluster.scope,
                ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
            )
    })
}

fn destination_incident_titles(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<String> {
    incident_titles(snapshot, |cluster| {
        cluster.destination_id.as_deref() == Some(destination_id)
            || matches!(
                cluster.scope,
                ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
            )
    })
}

fn incident_titles<F>(snapshot: &ProxyOperatorSnapshot, include: F) -> Vec<String>
where
    F: Fn(&ProxyOperatorIncidentCluster) -> bool,
{
    let mut titles = snapshot
        .incident_digest
        .clusters
        .iter()
        .filter(|cluster| include(cluster))
        .map(|cluster| cluster.title.clone())
        .collect::<Vec<_>>();
    titles.sort();
    titles.dedup();
    titles
}

fn route_recovery_surface(snapshot: &ProxyOperatorSnapshot, route_id: &str) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(candidate) = snapshot
        .recovery
        .route_candidates
        .iter()
        .find(|candidate| candidate.route_id == route_id)
    {
        items.push(format!("cache_policy={}", candidate.cache_policy));
        items.push(format!("capture_policy={}", candidate.capture_policy));
        items.push(format!(
            "rehydrate_on_connect={}",
            candidate.rehydrate_on_connect
        ));
        items.push(format!("replay_allowed={}", candidate.replay_allowed));
        items.push(format!("fallback_ready={}", candidate.fallback_ready));
    }
    items
}

fn destination_recovery_surface(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(candidate) = snapshot
        .recovery
        .destination_candidates
        .iter()
        .find(|candidate| candidate.destination_id == destination_id)
    {
        items.push(format!("queue_depth={}", candidate.queue_depth));
        items.push(format!(
            "send_failures_total={}",
            candidate.send_failures_total
        ));
        items.push(format!("drops_total={}", candidate.drops_total));
        items.push(format!("breaker_state={}", candidate.breaker_state));
    }
    items
}

fn merge_string_sections(left: Option<&[String]>, right: Option<&[String]>) -> Vec<String> {
    let mut merged = Vec::new();
    if let Some(left) = left {
        merged.extend(left.iter().cloned());
    }
    if let Some(right) = right {
        for item in right {
            if !merged.contains(item) {
                merged.push(item.clone());
            }
        }
    }
    merged
}

fn merge_actions(
    base: &[ProxyOperatorSuggestedAction],
    left: Option<&[ProxyOperatorSuggestedAction]>,
    right: Option<&[ProxyOperatorSuggestedAction]>,
) -> Vec<ProxyOperatorSuggestedAction> {
    let mut merged = base.to_vec();
    for actions in [left, right].into_iter().flatten() {
        for action in actions {
            if !merged.contains(action) {
                merged.push(action.clone());
            }
        }
    }
    merged
}

fn merge_trace_events(
    base: &[ProxyOperatorTraceEvent],
    left: Option<&[ProxyOperatorTraceEvent]>,
    right: Option<&[ProxyOperatorTraceEvent]>,
) -> Vec<ProxyOperatorTraceEvent> {
    let mut merged = base.to_vec();
    for events in [left, right].into_iter().flatten() {
        for event in events {
            if !merged.contains(event) {
                merged.push(event.clone());
            }
        }
    }
    merged
}

fn state_label(state: crate::ProxyOperatorState) -> &'static str {
    match state {
        crate::ProxyOperatorState::Healthy => "healthy",
        crate::ProxyOperatorState::Warning => "warning",
        crate::ProxyOperatorState::Blocked => "blocked",
    }
}
