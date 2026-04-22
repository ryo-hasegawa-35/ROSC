use serde::Serialize;

use crate::{
    ProxyOperatorSnapshot, ProxyOperatorSuggestedAction, ProxyOperatorTraceEventLevel,
    proxy_operator_casebook,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorBoardScope {
    Global,
    Route,
    Destination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorBoardItem {
    pub id: String,
    pub scope: ProxyOperatorBoardScope,
    pub level: ProxyOperatorTraceEventLevel,
    pub title: String,
    pub summary: String,
    pub reasons: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub route_id: Option<String>,
    pub destination_id: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorBoard {
    pub state: String,
    pub blocked_items: Vec<ProxyOperatorBoardItem>,
    pub degraded_items: Vec<ProxyOperatorBoardItem>,
    pub watch_items: Vec<ProxyOperatorBoardItem>,
}

pub fn proxy_operator_board(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorBoard {
    let casebook = if snapshot.casebook.route_casebooks.is_empty()
        && snapshot.casebook.destination_casebooks.is_empty()
    {
        proxy_operator_casebook(snapshot)
    } else {
        snapshot.casebook.clone()
    };

    let mut blocked_items = Vec::new();
    let mut degraded_items = Vec::new();
    let mut watch_items = Vec::new();

    for cluster in &snapshot.incident_digest.clusters {
        let item = ProxyOperatorBoardItem {
            id: format!("incident:{}", cluster.id),
            scope: match (cluster.route_id.as_ref(), cluster.destination_id.as_ref()) {
                (Some(_), _) => ProxyOperatorBoardScope::Route,
                (_, Some(_)) => ProxyOperatorBoardScope::Destination,
                _ => ProxyOperatorBoardScope::Global,
            },
            level: match cluster.level {
                crate::ProxyOperatorIncidentLevel::Blocked => ProxyOperatorTraceEventLevel::Blocked,
                crate::ProxyOperatorIncidentLevel::Degraded => {
                    ProxyOperatorTraceEventLevel::Degraded
                }
            },
            title: cluster.title.clone(),
            summary: cluster.summary.clone(),
            reasons: cluster.reasons.clone(),
            actions: cluster.action.clone().into_iter().collect(),
            route_id: cluster.route_id.clone(),
            destination_id: cluster.destination_id.clone(),
        };
        push_item(
            &mut blocked_items,
            &mut degraded_items,
            &mut watch_items,
            item,
        );
    }

    for route in &casebook.route_casebooks {
        let item = ProxyOperatorBoardItem {
            id: format!("route:{}", route.route_id),
            scope: ProxyOperatorBoardScope::Route,
            level: route.level,
            title: format!("Route `{}`", route.route_id),
            summary: route.summary.clone(),
            reasons: merge_reasons(
                route.incident_titles.clone(),
                route.recovery_surface.clone(),
            ),
            actions: route.recommended_actions.clone(),
            route_id: Some(route.route_id.clone()),
            destination_id: None,
        };
        push_item(
            &mut blocked_items,
            &mut degraded_items,
            &mut watch_items,
            item,
        );
    }

    for destination in &casebook.destination_casebooks {
        let item = ProxyOperatorBoardItem {
            id: format!("destination:{}", destination.destination_id),
            scope: ProxyOperatorBoardScope::Destination,
            level: destination.level,
            title: format!("Destination `{}`", destination.destination_id),
            summary: destination.summary.clone(),
            reasons: merge_reasons(
                destination.incident_titles.clone(),
                destination.recovery_surface.clone(),
            ),
            actions: destination.recommended_actions.clone(),
            route_id: None,
            destination_id: Some(destination.destination_id.clone()),
        };
        push_item(
            &mut blocked_items,
            &mut degraded_items,
            &mut watch_items,
            item,
        );
    }

    dedupe_items(&mut blocked_items);
    dedupe_items(&mut degraded_items);
    dedupe_items(&mut watch_items);

    ProxyOperatorBoard {
        state: casebook.state,
        blocked_items,
        degraded_items,
        watch_items,
    }
}

fn push_item(
    blocked_items: &mut Vec<ProxyOperatorBoardItem>,
    degraded_items: &mut Vec<ProxyOperatorBoardItem>,
    watch_items: &mut Vec<ProxyOperatorBoardItem>,
    item: ProxyOperatorBoardItem,
) {
    match item.level {
        ProxyOperatorTraceEventLevel::Blocked => blocked_items.push(item),
        ProxyOperatorTraceEventLevel::Degraded => degraded_items.push(item),
        ProxyOperatorTraceEventLevel::Info => {
            if !item.reasons.is_empty() || !item.actions.is_empty() {
                watch_items.push(item);
            }
        }
    }
}

fn dedupe_items(items: &mut Vec<ProxyOperatorBoardItem>) {
    items.sort_by_key(|item| item.id.clone());
    items.dedup_by(|left, right| left.id == right.id);
}

fn merge_reasons(left: Vec<String>, right: Vec<String>) -> Vec<String> {
    let mut merged = left;
    for item in right {
        if !merged.contains(&item) {
            merged.push(item);
        }
    }
    merged
}
