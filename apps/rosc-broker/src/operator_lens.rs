use serde::Serialize;

use crate::{
    ProxyOperatorBoardItem, ProxyOperatorBoardScope, ProxyOperatorDashboard,
    ProxyOperatorDestinationFocusPacket, ProxyOperatorIncidentScope, ProxyOperatorRouteFocusPacket,
    ProxyOperatorState, ProxyOperatorWorkItem,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteLens {
    pub route_id: String,
    pub focus: ProxyOperatorRouteFocusPacket,
    pub global_blockers: Vec<String>,
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
    let global_blockers = if snapshot.readiness.ready {
        Vec::new()
    } else {
        snapshot.readiness.reasons.clone()
    };
    let global_overrides = global_overrides(snapshot);

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
            global_blockers: global_blockers.clone(),
            global_overrides: global_overrides.clone(),
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
            global_blockers: global_blockers.clone(),
            global_overrides: global_overrides.clone(),
            focus,
        })
        .collect();

    ProxyOperatorLensCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global_blockers,
        global_overrides,
        routes,
        destinations,
    }
}

fn global_overrides(snapshot: &crate::ProxyOperatorSnapshot) -> Vec<String> {
    let mut overrides = Vec::new();
    if snapshot.attention.traffic_frozen {
        overrides.push("traffic_frozen".to_owned());
    }
    if snapshot.overview.report.overrides.launch_profile_mode != "normal" {
        overrides.push(format!(
            "launch_profile={}",
            snapshot.overview.report.overrides.launch_profile_mode
        ));
    }
    overrides
}

fn route_incident_titles(snapshot: &crate::ProxyOperatorSnapshot, route_id: &str) -> Vec<String> {
    incident_titles(snapshot, |cluster| {
        cluster.route_id.as_deref() == Some(route_id)
            || matches!(
                cluster.scope,
                ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
            )
    })
}

fn destination_incident_titles(
    snapshot: &crate::ProxyOperatorSnapshot,
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

fn incident_titles<F>(snapshot: &crate::ProxyOperatorSnapshot, include: F) -> Vec<String>
where
    F: Fn(&crate::ProxyOperatorIncidentCluster) -> bool,
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

fn route_work_items(
    snapshot: &crate::ProxyOperatorSnapshot,
    route_id: &str,
) -> Vec<ProxyOperatorWorkItem> {
    snapshot
        .worklist
        .items
        .iter()
        .filter(|item| {
            is_global_work_item(item)
                || item
                    .action
                    .as_ref()
                    .and_then(|action| action.route_id.as_deref())
                    == Some(route_id)
        })
        .cloned()
        .collect()
}

fn destination_work_items(
    snapshot: &crate::ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<ProxyOperatorWorkItem> {
    snapshot
        .worklist
        .items
        .iter()
        .filter(|item| {
            is_global_work_item(item)
                || item
                    .action
                    .as_ref()
                    .and_then(|action| action.destination_id.as_deref())
                    == Some(destination_id)
        })
        .cloned()
        .collect()
}

fn is_global_work_item(item: &ProxyOperatorWorkItem) -> bool {
    item.action
        .as_ref()
        .map(|action| action.route_id.is_none() && action.destination_id.is_none())
        .unwrap_or(true)
}

fn route_board_items(
    snapshot: &crate::ProxyOperatorSnapshot,
    route_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    scoped_board_items(snapshot, |item| {
        item.scope == ProxyOperatorBoardScope::Global || item.route_id.as_deref() == Some(route_id)
    })
}

fn destination_board_items(
    snapshot: &crate::ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    scoped_board_items(snapshot, |item| {
        item.scope == ProxyOperatorBoardScope::Global
            || item.destination_id.as_deref() == Some(destination_id)
    })
}

fn scoped_board_items<F>(
    snapshot: &crate::ProxyOperatorSnapshot,
    include: F,
) -> Vec<ProxyOperatorBoardItem>
where
    F: Fn(&ProxyOperatorBoardItem) -> bool,
{
    snapshot
        .board
        .blocked_items
        .iter()
        .chain(snapshot.board.degraded_items.iter())
        .chain(snapshot.board.watch_items.iter())
        .filter(|item| include(item))
        .cloned()
        .collect()
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
