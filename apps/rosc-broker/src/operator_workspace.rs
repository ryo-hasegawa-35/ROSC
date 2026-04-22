use serde::Serialize;

use crate::operator_context::{
    dashboard_context, destination_board_items, destination_incident_titles, destination_state,
    destination_work_items, route_board_items, route_incident_titles, route_state,
    route_work_items, scoped_destination_blockers, scoped_route_blockers,
};
use crate::{
    ProxyOperatorBoardItem, ProxyOperatorDashboard, ProxyOperatorDestinationMission,
    ProxyOperatorMissionCatalog, ProxyOperatorRouteMission, ProxyOperatorState,
    ProxyOperatorSuggestedAction, ProxyOperatorWorkItem,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorGlobalWorkspace {
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub blockers: Vec<String>,
    pub overrides: Vec<String>,
    pub board_highlights: Vec<String>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub mission: crate::ProxyOperatorGlobalMission,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteWorkspace {
    pub route_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_destination_ids: Vec<String>,
    pub mission: ProxyOperatorRouteMission,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationWorkspace {
    pub destination_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_route_ids: Vec<String>,
    pub mission: ProxyOperatorDestinationMission,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorWorkspaceCatalog {
    pub state: String,
    pub global: ProxyOperatorGlobalWorkspace,
    pub routes: Vec<ProxyOperatorRouteWorkspace>,
    pub destinations: Vec<ProxyOperatorDestinationWorkspace>,
}

pub fn proxy_operator_workspace_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorWorkspaceCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);
    let mission = &dashboard.mission;

    let routes = mission
        .routes
        .iter()
        .cloned()
        .map(|mission| ProxyOperatorRouteWorkspace {
            route_id: mission.route_id.clone(),
            state: state_label(route_state(snapshot, &mission.route_id)).to_owned(),
            headline: mission.headline.clone(),
            readiness_level: mission.readiness_level.clone(),
            global_blockers: context.global_blockers.clone(),
            scoped_blockers: scoped_route_blockers(snapshot, &mission.route_id),
            global_overrides: context.global_overrides.clone(),
            board_items: route_board_items(snapshot, &mission.route_id),
            work_items: route_work_items(snapshot, &mission.route_id),
            incident_titles: route_incident_titles(snapshot, &mission.route_id),
            next_steps: mission.next_steps.clone(),
            recommended_actions: mission.recommended_actions.clone(),
            linked_destination_ids: mission.linked_destination_ids.clone(),
            mission,
        })
        .collect();

    let destinations = mission
        .destinations
        .iter()
        .cloned()
        .map(|mission| ProxyOperatorDestinationWorkspace {
            destination_id: mission.destination_id.clone(),
            state: state_label(destination_state(snapshot, &mission.destination_id)).to_owned(),
            headline: mission.headline.clone(),
            readiness_level: mission.readiness_level.clone(),
            global_blockers: context.global_blockers.clone(),
            scoped_blockers: scoped_destination_blockers(snapshot, &mission.destination_id),
            global_overrides: context.global_overrides.clone(),
            board_items: destination_board_items(snapshot, &mission.destination_id),
            work_items: destination_work_items(snapshot, &mission.destination_id),
            incident_titles: destination_incident_titles(snapshot, &mission.destination_id),
            next_steps: mission.next_steps.clone(),
            recommended_actions: mission.recommended_actions.clone(),
            linked_route_ids: mission.linked_route_ids.clone(),
            mission,
        })
        .collect();

    ProxyOperatorWorkspaceCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global: ProxyOperatorGlobalWorkspace {
            state: state_label(snapshot.overview.report.state.clone()).to_owned(),
            headline: global_headline(mission),
            readiness_level: mission.global.readiness_level.clone(),
            blockers: context.global_blockers.clone(),
            overrides: context.global_overrides.clone(),
            board_highlights: mission.global.board_highlights.clone(),
            work_items: snapshot.worklist.items.clone(),
            incident_titles: mission.global.incident_titles.clone(),
            next_steps: mission.global.next_steps.clone(),
            recommended_actions: mission.global.recommended_actions.clone(),
            mission: mission.global.clone(),
        },
        routes,
        destinations,
    }
}

fn global_headline(mission: &ProxyOperatorMissionCatalog) -> String {
    if !mission.global.gate_reasons.is_empty() {
        mission.global.gate_reasons[0].clone()
    } else {
        mission.global.headline.clone()
    }
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
