use serde::Serialize;

use crate::operator_context::{
    dashboard_context, destination_incident_titles, destination_state, route_incident_titles,
    route_state, scoped_destination_blockers, scoped_route_blockers,
};
use crate::{
    ProxyOperatorBriefCatalog, ProxyOperatorDashboard, ProxyOperatorDestinationBrief,
    ProxyOperatorDestinationDossier, ProxyOperatorDestinationRunbook, ProxyOperatorRouteBrief,
    ProxyOperatorRouteDossier, ProxyOperatorRouteRunbook, ProxyOperatorState,
    ProxyOperatorSuggestedAction,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorGlobalMission {
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub gate_reasons: Vec<String>,
    pub blockers: Vec<String>,
    pub overrides: Vec<String>,
    pub board_highlights: Vec<String>,
    pub incident_titles: Vec<String>,
    pub recovery_summary: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteMission {
    pub route_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub board_highlights: Vec<String>,
    pub incident_titles: Vec<String>,
    pub trace_highlights: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_destination_ids: Vec<String>,
    pub recovery_surface: Vec<String>,
    pub brief: ProxyOperatorRouteBrief,
    pub dossier: ProxyOperatorRouteDossier,
    pub runbook: ProxyOperatorRouteRunbook,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationMission {
    pub destination_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub board_highlights: Vec<String>,
    pub incident_titles: Vec<String>,
    pub trace_highlights: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_route_ids: Vec<String>,
    pub recovery_surface: Vec<String>,
    pub brief: ProxyOperatorDestinationBrief,
    pub dossier: ProxyOperatorDestinationDossier,
    pub runbook: ProxyOperatorDestinationRunbook,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorMissionCatalog {
    pub state: String,
    pub global: ProxyOperatorGlobalMission,
    pub routes: Vec<ProxyOperatorRouteMission>,
    pub destinations: Vec<ProxyOperatorDestinationMission>,
}

pub fn proxy_operator_mission_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorMissionCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);
    let readiness_level = readiness_level_label(snapshot.readiness.level).to_owned();

    let routes = dashboard
        .runbook
        .routes
        .iter()
        .cloned()
        .filter_map(|runbook| {
            let brief = route_brief(&dashboard.brief, &runbook.route_id)?;
            let dossier = route_dossier(dashboard, &runbook.route_id)?;
            Some(ProxyOperatorRouteMission {
                route_id: runbook.route_id.clone(),
                state: state_label(route_state(snapshot, &runbook.route_id)).to_owned(),
                headline: route_headline(&brief, &runbook),
                readiness_level: readiness_level.clone(),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_route_blockers(snapshot, &runbook.route_id),
                global_overrides: context.global_overrides.clone(),
                board_highlights: runbook.board_highlights.clone(),
                incident_titles: route_incident_titles(snapshot, &runbook.route_id),
                trace_highlights: route_trace_highlights(dashboard, &runbook.route_id),
                next_steps: runbook.next_steps.clone(),
                recommended_actions: runbook.recommended_actions.clone(),
                linked_destination_ids: runbook.linked_destination_ids.clone(),
                recovery_surface: runbook.recovery_surface.clone(),
                brief,
                dossier,
                runbook,
            })
        })
        .collect();

    let destinations = dashboard
        .runbook
        .destinations
        .iter()
        .cloned()
        .filter_map(|runbook| {
            let brief = destination_brief(&dashboard.brief, &runbook.destination_id)?;
            let dossier = destination_dossier(dashboard, &runbook.destination_id)?;
            Some(ProxyOperatorDestinationMission {
                destination_id: runbook.destination_id.clone(),
                state: state_label(destination_state(snapshot, &runbook.destination_id)).to_owned(),
                headline: destination_headline(&brief, &runbook),
                readiness_level: readiness_level.clone(),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_destination_blockers(snapshot, &runbook.destination_id),
                global_overrides: context.global_overrides.clone(),
                board_highlights: runbook.board_highlights.clone(),
                incident_titles: destination_incident_titles(snapshot, &runbook.destination_id),
                trace_highlights: destination_trace_highlights(dashboard, &runbook.destination_id),
                next_steps: runbook.next_steps.clone(),
                recommended_actions: runbook.recommended_actions.clone(),
                linked_route_ids: runbook.linked_route_ids.clone(),
                recovery_surface: runbook.recovery_surface.clone(),
                brief,
                dossier,
                runbook,
            })
        })
        .collect();

    ProxyOperatorMissionCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global: ProxyOperatorGlobalMission {
            state: state_label(snapshot.overview.report.state.clone()).to_owned(),
            headline: global_headline(dashboard),
            readiness_level,
            gate_reasons: snapshot.readiness.reasons.clone(),
            blockers: context.global_blockers.clone(),
            overrides: context.global_overrides.clone(),
            board_highlights: global_board_highlights(dashboard),
            incident_titles: snapshot
                .incident_digest
                .clusters
                .iter()
                .map(|cluster| cluster.title.clone())
                .collect(),
            recovery_summary: global_recovery_summary(dashboard),
            next_steps: snapshot.triage.global.next_steps.clone(),
            recommended_actions: snapshot.triage.global.actions.clone(),
        },
        routes,
        destinations,
    }
}

fn route_brief(
    brief: &ProxyOperatorBriefCatalog,
    route_id: &str,
) -> Option<ProxyOperatorRouteBrief> {
    brief
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_brief(
    brief: &ProxyOperatorBriefCatalog,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationBrief> {
    brief
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn route_dossier(
    dashboard: &ProxyOperatorDashboard,
    route_id: &str,
) -> Option<ProxyOperatorRouteDossier> {
    dashboard
        .dossier
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_dossier(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationDossier> {
    dashboard
        .dossier
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn route_headline(brief: &ProxyOperatorRouteBrief, runbook: &ProxyOperatorRouteRunbook) -> String {
    if !runbook.scoped_blockers.is_empty() {
        runbook.headline.clone()
    } else {
        brief.summary.clone()
    }
}

fn destination_headline(
    brief: &ProxyOperatorDestinationBrief,
    runbook: &ProxyOperatorDestinationRunbook,
) -> String {
    if !runbook.scoped_blockers.is_empty() {
        runbook.headline.clone()
    } else {
        brief.summary.clone()
    }
}

fn route_trace_highlights(dashboard: &ProxyOperatorDashboard, route_id: &str) -> Vec<String> {
    dashboard
        .trace
        .routes
        .iter()
        .find(|trace| trace.route_id == route_id)
        .map(|trace| {
            trace
                .recent_events
                .iter()
                .take(4)
                .map(|event| event.title.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn destination_trace_highlights(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Vec<String> {
    dashboard
        .trace
        .destinations
        .iter()
        .find(|trace| trace.destination_id == destination_id)
        .map(|trace| {
            trace
                .recent_events
                .iter()
                .take(4)
                .map(|event| event.title.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn global_recovery_summary(dashboard: &ProxyOperatorDashboard) -> Vec<String> {
    let recovery = &dashboard.snapshot.recovery;
    vec![
        format!("cached_routes={}", recovery.cached_routes),
        format!("replayable_routes={}", recovery.replayable_routes),
        format!(
            "rehydrate_ready_destinations={}",
            recovery.rehydrate_ready_destinations
        ),
        format!("route_candidates={}", recovery.route_candidates.len()),
        format!(
            "destination_candidates={}",
            recovery.destination_candidates.len()
        ),
    ]
}

fn global_board_highlights(dashboard: &ProxyOperatorDashboard) -> Vec<String> {
    let mut highlights = Vec::new();
    for item in dashboard
        .snapshot
        .board
        .blocked_items
        .iter()
        .chain(dashboard.snapshot.board.degraded_items.iter())
        .chain(dashboard.snapshot.board.watch_items.iter())
    {
        if item.scope == crate::ProxyOperatorBoardScope::Global && !highlights.contains(&item.title)
        {
            highlights.push(item.title.clone());
        }
    }
    highlights
}

fn global_headline(dashboard: &ProxyOperatorDashboard) -> String {
    let snapshot = dashboard.snapshot.as_ref();
    if snapshot.attention.traffic_frozen {
        "Global override is freezing live traffic; thaw before trusting route-level recovery."
            .to_owned()
    } else if !snapshot.overview.report.blockers.is_empty() {
        format!(
            "{} config blocker(s) still gate a clean rollout.",
            snapshot.overview.report.blockers.len()
        )
    } else if snapshot.readiness.ready {
        "Broker is in a ready state with no active operator gate blockers.".to_owned()
    } else {
        snapshot
            .readiness
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| snapshot.triage.global.summary.clone())
    }
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}

fn readiness_level_label(level: crate::ProxyOperatorReadinessLevel) -> &'static str {
    match level {
        crate::ProxyOperatorReadinessLevel::Ready => "ready",
        crate::ProxyOperatorReadinessLevel::Degraded => "degraded",
        crate::ProxyOperatorReadinessLevel::Blocked => "blocked",
    }
}
