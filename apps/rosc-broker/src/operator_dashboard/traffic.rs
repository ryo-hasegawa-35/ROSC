use std::cmp::Reverse;
use std::collections::BTreeMap;

use serde::Serialize;

use crate::UdpProxyRuntimeStatus;

const HOTSPOT_LIMIT: usize = 3;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTrafficSummary {
    pub has_runtime_status: bool,
    pub ingress_packets_total: u64,
    pub ingress_drops_total: u64,
    pub route_matches_total: u64,
    pub route_dispatch_failures_total: u64,
    pub route_transform_failures_total: u64,
    pub destination_send_total: u64,
    pub destination_send_failures_total: u64,
    pub destination_drops_total: u64,
    pub destination_queue_depth_total: usize,
    pub destinations_with_backlog: usize,
    pub destinations_with_open_breakers: usize,
    pub destinations_with_half_open_breakers: usize,
    pub busiest_ingresses: Vec<ProxyOperatorCounterEntry>,
    pub busiest_routes: Vec<ProxyOperatorCounterEntry>,
    pub busiest_destinations: Vec<ProxyOperatorCounterEntry>,
    pub noisiest_destinations: Vec<ProxyOperatorCounterEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorCounterEntry {
    pub id: String,
    pub total: u64,
}

pub fn traffic_summary_from_runtime(
    runtime: &UdpProxyRuntimeStatus,
) -> ProxyOperatorTrafficSummary {
    let destination_send_total = runtime
        .destinations
        .iter()
        .map(|entry| entry.send_total)
        .sum();
    let destination_send_failures_total = runtime
        .destinations
        .iter()
        .map(|entry| entry.send_failures_total)
        .sum();
    let destination_queue_depth_total = runtime
        .destinations
        .iter()
        .map(|entry| entry.queue_depth)
        .sum();
    let destinations_with_backlog = runtime
        .destinations
        .iter()
        .filter(|entry| entry.queue_depth > 0)
        .count();
    let destinations_with_open_breakers = runtime
        .destinations
        .iter()
        .filter(|entry| {
            matches!(
                entry.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::Open)
            )
        })
        .count();
    let destinations_with_half_open_breakers = runtime
        .destinations
        .iter()
        .filter(|entry| {
            matches!(
                entry.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::HalfOpen)
            )
        })
        .count();

    let busiest_destinations = top_entries_from_iter(
        runtime
            .destinations
            .iter()
            .map(|entry| (entry.destination_id.clone(), entry.send_total)),
        HOTSPOT_LIMIT,
    );
    let noisiest_destinations = top_entries_from_iter(
        runtime.destinations.iter().map(|entry| {
            (
                entry.destination_id.clone(),
                entry.send_failures_total
                    + runtime
                        .destination_drops_total
                        .get(&entry.destination_id)
                        .copied()
                        .unwrap_or_default(),
            )
        }),
        HOTSPOT_LIMIT,
    );

    ProxyOperatorTrafficSummary {
        has_runtime_status: true,
        ingress_packets_total: sum_map(&runtime.ingress_packets_total),
        ingress_drops_total: sum_map(&runtime.ingress_drops_total),
        route_matches_total: sum_map(&runtime.route_matches_total),
        route_dispatch_failures_total: sum_map(&runtime.dispatch_failures_total),
        route_transform_failures_total: sum_map(&runtime.route_transform_failures_total),
        destination_send_total,
        destination_send_failures_total,
        destination_drops_total: sum_map(&runtime.destination_drops_total),
        destination_queue_depth_total,
        destinations_with_backlog,
        destinations_with_open_breakers,
        destinations_with_half_open_breakers,
        busiest_ingresses: top_entries(&runtime.ingress_packets_total, HOTSPOT_LIMIT),
        busiest_routes: top_entries(&runtime.route_matches_total, HOTSPOT_LIMIT),
        busiest_destinations,
        noisiest_destinations,
    }
}

fn sum_map(map: &BTreeMap<String, u64>) -> u64 {
    map.values().copied().sum()
}

fn top_entries(map: &BTreeMap<String, u64>, limit: usize) -> Vec<ProxyOperatorCounterEntry> {
    top_entries_from_iter(map.iter().map(|(id, total)| (id.clone(), *total)), limit)
}

fn top_entries_from_iter(
    entries: impl IntoIterator<Item = (String, u64)>,
    limit: usize,
) -> Vec<ProxyOperatorCounterEntry> {
    let mut ranked = entries
        .into_iter()
        .filter(|(_, total)| *total > 0)
        .map(|(id, total)| ProxyOperatorCounterEntry { id, total })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|entry| (Reverse(entry.total), entry.id.clone()));
    ranked.truncate(limit);
    ranked
}
