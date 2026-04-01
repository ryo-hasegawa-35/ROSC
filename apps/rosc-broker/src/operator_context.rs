use crate::{
    ProxyOperatorBoardItem, ProxyOperatorBoardScope, ProxyOperatorDashboard,
    ProxyOperatorIncidentScope, ProxyOperatorSnapshot, ProxyOperatorState,
    ProxyOperatorTraceEventLevel, ProxyOperatorWorkItem,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProxyOperatorContext {
    pub global_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
}

pub fn dashboard_context(dashboard: &ProxyOperatorDashboard) -> ProxyOperatorContext {
    snapshot_context(dashboard.snapshot.as_ref())
}

pub fn snapshot_context(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorContext {
    ProxyOperatorContext {
        global_blockers: global_blockers(snapshot),
        global_overrides: global_overrides(snapshot),
    }
}

pub fn scoped_route_blockers(snapshot: &ProxyOperatorSnapshot, route_id: &str) -> Vec<String> {
    scoped_blockers(
        snapshot,
        |item| item.route_id.as_deref() == Some(route_id),
        |cluster| {
            cluster.route_id.as_deref() == Some(route_id)
                || matches!(
                    cluster.scope,
                    ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
                )
        },
    )
}

pub fn scoped_destination_blockers(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<String> {
    scoped_blockers(
        snapshot,
        |item| item.destination_id.as_deref() == Some(destination_id),
        |cluster| {
            cluster.destination_id.as_deref() == Some(destination_id)
                || matches!(
                    cluster.scope,
                    ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
                )
        },
    )
}

pub fn route_work_items(
    snapshot: &ProxyOperatorSnapshot,
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

pub fn destination_work_items(
    snapshot: &ProxyOperatorSnapshot,
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

pub fn route_board_items(
    snapshot: &ProxyOperatorSnapshot,
    route_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    scoped_board_items(snapshot, |item| {
        item.scope == ProxyOperatorBoardScope::Global || item.route_id.as_deref() == Some(route_id)
    })
}

pub fn destination_board_items(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
) -> Vec<ProxyOperatorBoardItem> {
    scoped_board_items(snapshot, |item| {
        item.scope == ProxyOperatorBoardScope::Global
            || item.destination_id.as_deref() == Some(destination_id)
    })
}

pub fn route_incident_titles(snapshot: &ProxyOperatorSnapshot, route_id: &str) -> Vec<String> {
    incident_titles(snapshot, |cluster| {
        cluster.route_id.as_deref() == Some(route_id)
            || matches!(
                cluster.scope,
                ProxyOperatorIncidentScope::Global | ProxyOperatorIncidentScope::Config
            )
    })
}

pub fn destination_incident_titles(
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

pub fn route_state(snapshot: &ProxyOperatorSnapshot, route_id: &str) -> ProxyOperatorState {
    scoped_state(scoped_board_items(snapshot, |item| {
        item.route_id.as_deref() == Some(route_id)
    }))
}

pub fn destination_state(
    snapshot: &ProxyOperatorSnapshot,
    destination_id: &str,
) -> ProxyOperatorState {
    scoped_state(scoped_board_items(snapshot, |item| {
        item.destination_id.as_deref() == Some(destination_id)
    }))
}

fn global_blockers(snapshot: &ProxyOperatorSnapshot) -> Vec<String> {
    let mut blockers = snapshot.overview.report.blockers.clone();
    for item in snapshot
        .board
        .blocked_items
        .iter()
        .chain(snapshot.board.degraded_items.iter())
    {
        if item.scope == ProxyOperatorBoardScope::Global {
            push_unique(&mut blockers, item.title.clone());
            for reason in &item.reasons {
                push_unique(&mut blockers, reason.clone());
            }
        }
    }
    if blockers.is_empty() && snapshot.overview.report.overrides.traffic_frozen {
        blockers.push("traffic is currently frozen by operator override".to_owned());
    }
    blockers
}

fn global_overrides(snapshot: &ProxyOperatorSnapshot) -> Vec<String> {
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

fn scoped_blockers<BoardFilter, ClusterFilter>(
    snapshot: &ProxyOperatorSnapshot,
    board_filter: BoardFilter,
    cluster_filter: ClusterFilter,
) -> Vec<String>
where
    BoardFilter: Fn(&ProxyOperatorBoardItem) -> bool,
    ClusterFilter: Fn(&crate::ProxyOperatorIncidentCluster) -> bool,
{
    let mut blockers = Vec::new();
    for item in snapshot
        .board
        .blocked_items
        .iter()
        .chain(snapshot.board.degraded_items.iter())
        .filter(|item| board_filter(item))
    {
        push_unique(&mut blockers, item.title.clone());
        for reason in &item.reasons {
            push_unique(&mut blockers, reason.clone());
        }
    }

    for cluster in snapshot
        .incident_digest
        .clusters
        .iter()
        .filter(|cluster| cluster_filter(cluster))
    {
        push_unique(&mut blockers, cluster.title.clone());
        for reason in &cluster.reasons {
            push_unique(&mut blockers, reason.clone());
        }
    }

    blockers
}

fn incident_titles<F>(snapshot: &ProxyOperatorSnapshot, include: F) -> Vec<String>
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

fn scoped_board_items<F>(
    snapshot: &ProxyOperatorSnapshot,
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

fn is_global_work_item(item: &ProxyOperatorWorkItem) -> bool {
    item.action
        .as_ref()
        .map(|action| action.route_id.is_none() && action.destination_id.is_none())
        .unwrap_or(true)
}

fn push_unique(target: &mut Vec<String>, value: String) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn scoped_state(items: Vec<ProxyOperatorBoardItem>) -> ProxyOperatorState {
    if items
        .iter()
        .any(|item| item.level == ProxyOperatorTraceEventLevel::Blocked)
    {
        ProxyOperatorState::Blocked
    } else if !items.is_empty() {
        ProxyOperatorState::Warning
    } else {
        ProxyOperatorState::Healthy
    }
}
