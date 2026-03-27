use serde::Serialize;

use crate::{
    ProxyOperatorBoardItem, ProxyOperatorDashboard, ProxyOperatorDestinationCasebook,
    ProxyOperatorDestinationDetail, ProxyOperatorDestinationHandoff,
    ProxyOperatorDestinationTimeline, ProxyOperatorDestinationTrace,
    ProxyOperatorDestinationTriage, ProxyOperatorRouteCasebook, ProxyOperatorRouteDetail,
    ProxyOperatorRouteHandoff, ProxyOperatorRouteTimeline, ProxyOperatorRouteTrace,
    ProxyOperatorRouteTriage,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteFocusPacket {
    pub route_id: String,
    pub detail: ProxyOperatorRouteDetail,
    pub trace: Option<ProxyOperatorRouteTrace>,
    pub timeline: Option<ProxyOperatorRouteTimeline>,
    pub handoff: Option<ProxyOperatorRouteHandoff>,
    pub triage: Option<ProxyOperatorRouteTriage>,
    pub casebook: Option<ProxyOperatorRouteCasebook>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationFocusPacket {
    pub destination_id: String,
    pub detail: ProxyOperatorDestinationDetail,
    pub trace: Option<ProxyOperatorDestinationTrace>,
    pub timeline: Option<ProxyOperatorDestinationTimeline>,
    pub handoff: Option<ProxyOperatorDestinationHandoff>,
    pub triage: Option<ProxyOperatorDestinationTriage>,
    pub casebook: Option<ProxyOperatorDestinationCasebook>,
    pub board_items: Vec<ProxyOperatorBoardItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorFocusCatalog {
    pub state: String,
    pub routes: Vec<ProxyOperatorRouteFocusPacket>,
    pub destinations: Vec<ProxyOperatorDestinationFocusPacket>,
}

pub fn proxy_operator_focus_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorFocusCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let casebook = &snapshot.casebook;
    let board = &snapshot.board;

    let routes = dashboard
        .route_details
        .iter()
        .cloned()
        .map(|detail| ProxyOperatorRouteFocusPacket {
            route_id: detail.route_id.clone(),
            trace: dashboard
                .trace
                .routes
                .iter()
                .find(|entry| entry.route_id == detail.route_id)
                .cloned(),
            timeline: dashboard
                .timeline_catalog
                .routes
                .iter()
                .find(|entry| entry.route_id == detail.route_id)
                .cloned(),
            handoff: snapshot
                .handoff
                .route_handoffs
                .iter()
                .find(|entry| entry.route_id == detail.route_id)
                .cloned(),
            triage: snapshot
                .triage
                .route_triage
                .iter()
                .find(|entry| entry.route_id == detail.route_id)
                .cloned(),
            casebook: casebook
                .route_casebooks
                .iter()
                .find(|entry| entry.route_id == detail.route_id)
                .cloned(),
            board_items: route_board_items(board, &detail.route_id),
            detail,
        })
        .collect::<Vec<_>>();

    let destinations = dashboard
        .destination_details
        .iter()
        .cloned()
        .map(|detail| ProxyOperatorDestinationFocusPacket {
            destination_id: detail.destination_id.clone(),
            trace: dashboard
                .trace
                .destinations
                .iter()
                .find(|entry| entry.destination_id == detail.destination_id)
                .cloned(),
            timeline: dashboard
                .timeline_catalog
                .destinations
                .iter()
                .find(|entry| entry.destination_id == detail.destination_id)
                .cloned(),
            handoff: snapshot
                .handoff
                .destination_handoffs
                .iter()
                .find(|entry| entry.destination_id == detail.destination_id)
                .cloned(),
            triage: snapshot
                .triage
                .destination_triage
                .iter()
                .find(|entry| entry.destination_id == detail.destination_id)
                .cloned(),
            casebook: casebook
                .destination_casebooks
                .iter()
                .find(|entry| entry.destination_id == detail.destination_id)
                .cloned(),
            board_items: destination_board_items(board, &detail.destination_id),
            detail,
        })
        .collect::<Vec<_>>();

    ProxyOperatorFocusCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        routes,
        destinations,
    }
}

fn route_board_items(
    board: &crate::ProxyOperatorBoard,
    route_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    board
        .blocked_items
        .iter()
        .chain(board.degraded_items.iter())
        .chain(board.watch_items.iter())
        .filter(|item| item.route_id.as_deref() == Some(route_id))
        .cloned()
        .collect()
}

fn destination_board_items(
    board: &crate::ProxyOperatorBoard,
    destination_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    board
        .blocked_items
        .iter()
        .chain(board.degraded_items.iter())
        .chain(board.watch_items.iter())
        .filter(|item| item.destination_id.as_deref() == Some(destination_id))
        .cloned()
        .collect()
}

fn state_label(state: crate::ProxyOperatorState) -> &'static str {
    match state {
        crate::ProxyOperatorState::Healthy => "healthy",
        crate::ProxyOperatorState::Warning => "warning",
        crate::ProxyOperatorState::Blocked => "blocked",
    }
}
