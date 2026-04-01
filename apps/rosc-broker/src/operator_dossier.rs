use serde::Serialize;

use crate::{
    ProxyOperatorBoardItem, ProxyOperatorBriefCatalog, ProxyOperatorDashboard,
    ProxyOperatorDestinationBrief, ProxyOperatorDestinationFocusPacket,
    ProxyOperatorDestinationLens, ProxyOperatorRouteBrief, ProxyOperatorRouteFocusPacket,
    ProxyOperatorRouteLens, ProxyOperatorState, ProxyOperatorSuggestedAction,
    ProxyOperatorWorkItem,
    operator_context::{
        dashboard_context, destination_board_items, destination_incident_titles,
        destination_work_items, route_board_items, route_incident_titles, route_work_items,
        scoped_destination_blockers, scoped_route_blockers,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteDossier {
    pub route_id: String,
    pub state: String,
    pub summary: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub headline_timeline: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
    pub focus: ProxyOperatorRouteFocusPacket,
    pub brief: Option<ProxyOperatorRouteBrief>,
    pub lens: Option<ProxyOperatorRouteLens>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationDossier {
    pub destination_id: String,
    pub state: String,
    pub summary: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub headline_timeline: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
    pub focus: ProxyOperatorDestinationFocusPacket,
    pub brief: Option<ProxyOperatorDestinationBrief>,
    pub lens: Option<ProxyOperatorDestinationLens>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDossierCatalog {
    pub state: String,
    pub global_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub routes: Vec<ProxyOperatorRouteDossier>,
    pub destinations: Vec<ProxyOperatorDestinationDossier>,
}

pub fn proxy_operator_dossier_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorDossierCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);

    let routes = dashboard
        .focus
        .routes
        .iter()
        .cloned()
        .map(|focus| {
            let brief = route_brief(&dashboard.brief, &focus.route_id);
            let lens = route_lens(dashboard, &focus.route_id);
            ProxyOperatorRouteDossier {
                route_id: focus.route_id.clone(),
                state: state_label(snapshot.overview.report.state.clone()).to_owned(),
                summary: brief
                    .as_ref()
                    .map(|entry| entry.summary.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{} destination(s), {} warning(s), fallback_ready={}",
                            focus.detail.destination_ids.len(),
                            focus.detail.warnings.len(),
                            focus.detail.direct_udp_fallback_available
                        )
                    }),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_route_blockers(snapshot, &focus.route_id),
                global_overrides: context.global_overrides.clone(),
                incident_titles: route_incident_titles(snapshot, &focus.route_id),
                headline_timeline: brief
                    .as_ref()
                    .map(|entry| entry.headline_timeline.clone())
                    .unwrap_or_default(),
                next_steps: brief
                    .as_ref()
                    .map(|entry| entry.next_steps.clone())
                    .unwrap_or_default(),
                recommended_actions: brief
                    .as_ref()
                    .map(|entry| entry.recommended_actions.clone())
                    .unwrap_or_default(),
                work_items: route_work_items(snapshot, &focus.route_id),
                board_items: route_board_items(snapshot, &focus.route_id),
                focus,
                brief,
                lens,
            }
        })
        .collect();

    let destinations = dashboard
        .focus
        .destinations
        .iter()
        .cloned()
        .map(|focus| {
            let brief = destination_brief(&dashboard.brief, &focus.destination_id);
            let lens = destination_lens(dashboard, &focus.destination_id);
            ProxyOperatorDestinationDossier {
                destination_id: focus.destination_id.clone(),
                state: state_label(snapshot.overview.report.state.clone()).to_owned(),
                summary: brief
                    .as_ref()
                    .map(|entry| entry.summary.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "queue_depth={}, send_failures={}, drops={}, linked_routes={}",
                            focus.detail.live_queue_depth,
                            focus.detail.send_failures_total,
                            focus.detail.drops_total,
                            focus.detail.route_ids.len()
                        )
                    }),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_destination_blockers(snapshot, &focus.destination_id),
                global_overrides: context.global_overrides.clone(),
                incident_titles: destination_incident_titles(snapshot, &focus.destination_id),
                headline_timeline: brief
                    .as_ref()
                    .map(|entry| entry.headline_timeline.clone())
                    .unwrap_or_default(),
                next_steps: brief
                    .as_ref()
                    .map(|entry| entry.next_steps.clone())
                    .unwrap_or_default(),
                recommended_actions: brief
                    .as_ref()
                    .map(|entry| entry.recommended_actions.clone())
                    .unwrap_or_default(),
                work_items: destination_work_items(snapshot, &focus.destination_id),
                board_items: destination_board_items(snapshot, &focus.destination_id),
                focus,
                brief,
                lens,
            }
        })
        .collect();

    ProxyOperatorDossierCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global_blockers: context.global_blockers,
        global_overrides: context.global_overrides,
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

fn route_lens(
    dashboard: &ProxyOperatorDashboard,
    route_id: &str,
) -> Option<ProxyOperatorRouteLens> {
    dashboard
        .lens
        .routes
        .iter()
        .find(|entry| entry.route_id == route_id)
        .cloned()
}

fn destination_lens(
    dashboard: &ProxyOperatorDashboard,
    destination_id: &str,
) -> Option<ProxyOperatorDestinationLens> {
    dashboard
        .lens
        .destinations
        .iter()
        .find(|entry| entry.destination_id == destination_id)
        .cloned()
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
