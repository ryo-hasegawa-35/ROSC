use serde::Serialize;

use crate::{
    ProxyOperatorBoardItem, ProxyOperatorDashboard, ProxyOperatorDestinationFocusPacket,
    ProxyOperatorRouteFocusPacket, ProxyOperatorState, ProxyOperatorWorkItem,
    operator_context::{
        dashboard_context, destination_board_items, destination_incident_titles,
        destination_work_items, route_board_items, route_incident_titles, route_work_items,
        scoped_destination_blockers, scoped_route_blockers,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteLens {
    pub route_id: String,
    pub focus: ProxyOperatorRouteFocusPacket,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationLens {
    pub destination_id: String,
    pub focus: ProxyOperatorDestinationFocusPacket,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub incident_titles: Vec<String>,
    pub work_items: Vec<ProxyOperatorWorkItem>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorLensCatalog {
    pub state: String,
    pub global_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub routes: Vec<ProxyOperatorRouteLens>,
    pub destinations: Vec<ProxyOperatorDestinationLens>,
}

pub fn proxy_operator_lens_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorLensCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);

    let routes = dashboard
        .focus
        .routes
        .iter()
        .cloned()
        .map(|focus| ProxyOperatorRouteLens {
            route_id: focus.route_id.clone(),
            incident_titles: route_incident_titles(snapshot, &focus.route_id),
            work_items: route_work_items(snapshot, &focus.route_id),
            board_items: route_board_items(snapshot, &focus.route_id),
            global_blockers: context.global_blockers.clone(),
            scoped_blockers: scoped_route_blockers(snapshot, &focus.route_id),
            global_overrides: context.global_overrides.clone(),
            focus,
        })
        .collect();

    let destinations = dashboard
        .focus
        .destinations
        .iter()
        .cloned()
        .map(|focus| ProxyOperatorDestinationLens {
            destination_id: focus.destination_id.clone(),
            incident_titles: destination_incident_titles(snapshot, &focus.destination_id),
            work_items: destination_work_items(snapshot, &focus.destination_id),
            board_items: destination_board_items(snapshot, &focus.destination_id),
            global_blockers: context.global_blockers.clone(),
            scoped_blockers: scoped_destination_blockers(snapshot, &focus.destination_id),
            global_overrides: context.global_overrides.clone(),
            focus,
        })
        .collect();

    ProxyOperatorLensCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global_blockers: context.global_blockers,
        global_overrides: context.global_overrides,
        routes,
        destinations,
    }
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
