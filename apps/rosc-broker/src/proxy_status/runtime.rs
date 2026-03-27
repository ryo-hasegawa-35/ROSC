use std::collections::{BTreeMap, BTreeSet};

use rosc_telemetry::HealthSnapshot;

use super::types::{
    UdpProxyDestinationRuntimeStatus, UdpProxyRuntimeStatus, UdpProxyStatusSnapshot,
};

pub fn attach_runtime_status(
    mut status: UdpProxyStatusSnapshot,
    snapshot: &HealthSnapshot,
) -> UdpProxyStatusSnapshot {
    status.runtime = Some(UdpProxyRuntimeStatus {
        traffic_frozen: snapshot.traffic_frozen,
        isolated_route_ids: snapshot
            .route_isolated
            .iter()
            .filter(|(_, isolated)| **isolated)
            .map(|(route_id, _)| route_id.clone())
            .collect(),
        operator_actions_total: snapshot.operator_actions_total.clone(),
        recent_operator_actions: snapshot.recent_operator_actions.clone(),
        recent_config_events: snapshot.recent_config_events.clone(),
        config_revision: snapshot.config_revision,
        config_rejections_total: snapshot.config_rejections_total,
        config_blocked_total: snapshot.config_blocked_total,
        config_reload_failures_total: snapshot.config_reload_failures_total,
        ingress_packets_total: snapshot.ingress_packets_total.clone(),
        ingress_drops_total: collapse_reason_counts(&snapshot.ingress_drops_total),
        dispatch_failures_total: collapse_dispatch_failures(&snapshot.dispatch_failures_total),
        route_matches_total: snapshot.route_matches_total.clone(),
        route_transform_failures_total: snapshot.route_transform_failures_total.clone(),
        destination_drops_total: collapse_reason_counts(&snapshot.destination_drops_total),
        destinations: destination_runtime(snapshot),
    });
    status
}

pub fn operator_warnings(status: &UdpProxyStatusSnapshot) -> Vec<String> {
    let mut warnings = status.warnings.clone();
    for route in &status.route_assessments {
        if !route.active {
            continue;
        }
        for warning in &route.warnings {
            warnings.push(format!("route `{}`: {}", route.route_id, warning));
        }
    }
    warnings
}

pub fn startup_blockers(
    status: &UdpProxyStatusSnapshot,
    fail_on_warnings: bool,
    require_fallback_ready: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if require_fallback_ready {
        for route in &status.route_assessments {
            if route.active && !route.direct_udp_fallback_available {
                blockers.push(format!(
                    "route `{}` does not have a direct UDP fallback target",
                    route.route_id
                ));
            }
        }
    }
    if fail_on_warnings {
        blockers.extend(operator_warnings(status));
    }
    blockers
}

fn collapse_reason_counts(counts: &BTreeMap<(String, String), u64>) -> BTreeMap<String, u64> {
    let mut collapsed = BTreeMap::new();
    for ((id, _reason), count) in counts {
        *collapsed.entry(id.clone()).or_default() += count;
    }
    collapsed
}

fn collapse_dispatch_failures(
    counts: &BTreeMap<(String, String, String), u64>,
) -> BTreeMap<String, u64> {
    let mut collapsed = BTreeMap::new();
    for ((route_id, _destination_id, _reason), count) in counts {
        *collapsed.entry(route_id.clone()).or_default() += count;
    }
    collapsed
}

fn destination_runtime(snapshot: &HealthSnapshot) -> Vec<UdpProxyDestinationRuntimeStatus> {
    let mut destination_ids = BTreeSet::new();
    destination_ids.extend(snapshot.queue_depth.keys().cloned());
    destination_ids.extend(snapshot.destination_sent_total.keys().cloned());
    destination_ids.extend(
        snapshot
            .destination_send_failures_total
            .keys()
            .map(|(destination_id, _reason)| destination_id.clone()),
    );
    destination_ids.extend(snapshot.destination_breaker_state.keys().cloned());

    destination_ids
        .into_iter()
        .map(|destination_id| UdpProxyDestinationRuntimeStatus {
            queue_depth: snapshot
                .queue_depth
                .get(&destination_id)
                .copied()
                .unwrap_or_default(),
            send_total: snapshot
                .destination_sent_total
                .get(&destination_id)
                .copied()
                .unwrap_or_default(),
            send_failures_total: snapshot
                .destination_send_failures_total
                .iter()
                .filter(|((id, _), _)| id == &destination_id)
                .map(|(_, count)| *count)
                .sum(),
            breaker_state: snapshot
                .destination_breaker_state
                .get(&destination_id)
                .cloned(),
            destination_id,
        })
        .collect()
}
