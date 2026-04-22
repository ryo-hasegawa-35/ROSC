use serde::Serialize;

use crate::operator_context::{
    dashboard_context, destination_incident_titles, destination_state, route_incident_titles,
    route_state, scoped_destination_blockers, scoped_route_blockers,
};
use crate::{
    ProxyOperatorDashboard, ProxyOperatorDestinationFocusPacket, ProxyOperatorDestinationMission,
    ProxyOperatorDestinationRunbook, ProxyOperatorDestinationWorkspace,
    ProxyOperatorRouteFocusPacket, ProxyOperatorRouteMission, ProxyOperatorRouteRunbook,
    ProxyOperatorRouteWorkspace, ProxyOperatorState, ProxyOperatorSuggestedAction,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorGlobalCockpit {
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub blockers: Vec<String>,
    pub overrides: Vec<String>,
    pub board_highlights: Vec<String>,
    pub work_item_titles: Vec<String>,
    pub incident_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub mission: crate::ProxyOperatorGlobalMission,
    pub workspace: crate::ProxyOperatorGlobalWorkspace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteCockpit {
    pub route_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub board_titles: Vec<String>,
    pub work_item_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_destination_ids: Vec<String>,
    pub focus: ProxyOperatorRouteFocusPacket,
    pub mission: ProxyOperatorRouteMission,
    pub workspace: ProxyOperatorRouteWorkspace,
    pub runbook: ProxyOperatorRouteRunbook,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationCockpit {
    pub destination_id: String,
    pub state: String,
    pub headline: String,
    pub readiness_level: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub board_titles: Vec<String>,
    pub work_item_titles: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_route_ids: Vec<String>,
    pub focus: ProxyOperatorDestinationFocusPacket,
    pub mission: ProxyOperatorDestinationMission,
    pub workspace: ProxyOperatorDestinationWorkspace,
    pub runbook: ProxyOperatorDestinationRunbook,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorCockpitCatalog {
    pub state: String,
    pub global: ProxyOperatorGlobalCockpit,
    pub routes: Vec<ProxyOperatorRouteCockpit>,
    pub destinations: Vec<ProxyOperatorDestinationCockpit>,
}

pub fn proxy_operator_cockpit_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorCockpitCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);
    let mission = &dashboard.mission;
    let workspace = &dashboard.workspace;

    let routes = workspace
        .routes
        .iter()
        .cloned()
        .filter_map(|workspace| {
            let focus = route_focus(dashboard, &workspace.route_id)?;
            let mission = route_mission(dashboard, &workspace.route_id)?;
            let runbook = route_runbook(dashboard, &workspace.route_id)?;
            Some(ProxyOperatorRouteCockpit {
                route_id: workspace.route_id.clone(),
                state: state_label(route_state(snapshot, &workspace.route_id)).to_owned(),
                headline: route_headline(&workspace, &runbook),
                readiness_level: workspace.readiness_level.clone(),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_route_blockers(snapshot, &workspace.route_id),
                global_overrides: context.global_overrides.clone(),
                incident_titles: route_incident_titles(snapshot, &workspace.route_id),
                board_titles: workspace
                    .board_items
                    .iter()
                    .map(|item| item.title.clone())
                    .collect(),
                work_item_titles: workspace
                    .work_items
                    .iter()
                    .map(|item| item.title.clone())
                    .collect(),
                next_steps: workspace.next_steps.clone(),
                recommended_actions: workspace.recommended_actions.clone(),
                linked_destination_ids: workspace.linked_destination_ids.clone(),
                focus,
                mission,
                workspace,
                runbook,
            })
        })
        .collect();

    let destinations = workspace
        .destinations
        .iter()
        .cloned()
        .filter_map(|workspace| {
            let focus = destination_focus(dashboard, &workspace.destination_id)?;
            let mission = destination_mission(dashboard, &workspace.destination_id)?;
            let runbook = destination_runbook(dashboard, &workspace.destination_id)?;
            Some(ProxyOperatorDestinationCockpit {
                destination_id: workspace.destination_id.clone(),
                state: state_label(destination_state(snapshot, &workspace.destination_id))
                    .to_owned(),
                headline: destination_headline(&workspace, &runbook),
                readiness_level: workspace.readiness_level.clone(),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_destination_blockers(snapshot, &workspace.destination_id),
                global_overrides: context.global_overrides.clone(),
                incident_titles: destination_incident_titles(snapshot, &workspace.destination_id),
                board_titles: workspace
                    .board_items
                    .iter()
                    .map(|item| item.title.clone())
                    .collect(),
                work_item_titles: workspace
                    .work_items
                    .iter()
                    .map(|item| item.title.clone())
                    .collect(),
                next_steps: workspace.next_steps.clone(),
                recommended_actions: workspace.recommended_actions.clone(),
                linked_route_ids: workspace.linked_route_ids.clone(),
                focus,
                mission,
                workspace,
                runbook,
            })
        })
        .collect();

    ProxyOperatorCockpitCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global: ProxyOperatorGlobalCockpit {
            state: state_label(snapshot.overview.report.state.clone()).to_owned(),
            headline: global_headline(mission, workspace),
            readiness_level: workspace.global.readiness_level.clone(),
            blockers: context.global_blockers.clone(),
            overrides: context.global_overrides.clone(),
            board_highlights: workspace.global.board_highlights.clone(),
            work_item_titles: snapshot
                .worklist
                .items
                .iter()
                .map(|item| item.title.clone())
                .collect(),
            incident_titles: mission.global.incident_titles.clone(),
            next_steps: workspace.global.next_steps.clone(),
            recommended_actions: workspace.global.recommended_actions.clone(),
            mission: mission.global.clone(),
            workspace: workspace.global.clone(),
        },
        routes,
        destinations,
    }
}

fn route_focus(
    dashboard: &ProxyOperatorDashboard,
    route_id: &str,
) -> Option<ProxyOperatorRouteFocusPacket> {
    dashboard
        .focus
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_focus(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationFocusPacket> {
    dashboard
        .focus
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn route_mission(
    dashboard: &ProxyOperatorDashboard,
    route_id: &str,
) -> Option<ProxyOperatorRouteMission> {
    dashboard
        .mission
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_mission(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationMission> {
    dashboard
        .mission
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn route_runbook(
    dashboard: &ProxyOperatorDashboard,
    route_id: &str,
) -> Option<ProxyOperatorRouteRunbook> {
    dashboard
        .runbook
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_runbook(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationRunbook> {
    dashboard
        .runbook
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn global_headline(
    mission: &crate::ProxyOperatorMissionCatalog,
    workspace: &crate::ProxyOperatorWorkspaceCatalog,
) -> String {
    if !workspace.global.work_items.is_empty() {
        format!(
            "{} active work item(s) are still gating a clean operator handoff.",
            workspace.global.work_items.len()
        )
    } else if !mission.global.gate_reasons.is_empty() {
        mission.global.gate_reasons[0].clone()
    } else {
        workspace.global.headline.clone()
    }
}

fn route_headline(
    workspace: &ProxyOperatorRouteWorkspace,
    runbook: &ProxyOperatorRouteRunbook,
) -> String {
    if !workspace.work_items.is_empty() {
        format!(
            "{} live work item(s) still target this route.",
            workspace.work_items.len()
        )
    } else if !runbook.next_steps.is_empty() {
        runbook.next_steps[0].clone()
    } else {
        workspace.headline.clone()
    }
}

fn destination_headline(
    workspace: &ProxyOperatorDestinationWorkspace,
    runbook: &ProxyOperatorDestinationRunbook,
) -> String {
    if !workspace.work_items.is_empty() {
        format!(
            "{} live work item(s) still target this destination.",
            workspace.work_items.len()
        )
    } else if !runbook.next_steps.is_empty() {
        runbook.next_steps[0].clone()
    } else {
        workspace.headline.clone()
    }
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
