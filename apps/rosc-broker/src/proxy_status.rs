use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use rosc_config::{BrokerConfig, DropPolicyConfig};
use rosc_route::{CachePolicy, CapturePolicy, TrafficClass, TransportSelector};
use rosc_telemetry::{BreakerStateSnapshot, HealthSnapshot};
use serde::Serialize;

use crate::ProxyLaunchProfileStatus;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyStatusSnapshot {
    pub launch_profile: ProxyLaunchProfileStatus,
    pub summary: UdpProxySummary,
    pub runtime: Option<UdpProxyRuntimeStatus>,
    pub ingresses: Vec<UdpProxyIngressStatus>,
    pub destinations: Vec<UdpProxyDestinationStatus>,
    pub routes: Vec<UdpProxyRouteStatus>,
    pub fallback_routes: Vec<UdpProxyFallbackStatus>,
    pub route_assessments: Vec<UdpProxyRouteAssessment>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxySummary {
    pub total_routes: usize,
    pub active_routes: usize,
    pub disabled_routes: usize,
    pub active_ingresses: usize,
    pub active_destinations: usize,
    pub fallback_ready_routes: usize,
    pub fallback_missing_routes: usize,
    pub warning_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRuntimeStatus {
    pub traffic_frozen: bool,
    pub config_revision: u64,
    pub config_rejections_total: u64,
    pub config_blocked_total: u64,
    pub config_reload_failures_total: u64,
    pub ingress_packets_total: BTreeMap<String, u64>,
    pub ingress_drops_total: BTreeMap<String, u64>,
    pub dispatch_failures_total: BTreeMap<String, u64>,
    pub route_matches_total: BTreeMap<String, u64>,
    pub route_transform_failures_total: BTreeMap<String, u64>,
    pub destination_drops_total: BTreeMap<String, u64>,
    pub destinations: Vec<UdpProxyDestinationRuntimeStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyDestinationRuntimeStatus {
    pub destination_id: String,
    pub queue_depth: usize,
    pub send_total: u64,
    pub send_failures_total: u64,
    pub breaker_state: Option<BreakerStateSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyIngressStatus {
    pub id: String,
    pub configured_bind: String,
    pub bound_local_addr: Option<String>,
    pub route_ids: Vec<String>,
    pub max_packet_size: usize,
    pub mode: rosc_osc::CompatibilityMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyDestinationStatus {
    pub id: String,
    pub bind: String,
    pub target: String,
    pub route_ids: Vec<String>,
    pub queue_depth: usize,
    pub drop_policy: DropPolicyConfig,
    pub breaker_open_after_consecutive_failures: u32,
    pub breaker_open_after_consecutive_queue_overflows: u32,
    pub breaker_cooldown_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRouteStatus {
    pub id: String,
    pub enabled: bool,
    pub mode: rosc_osc::CompatibilityMode,
    pub traffic_class: TrafficClass,
    pub ingress_ids: Vec<String>,
    pub address_patterns: Vec<String>,
    pub destination_ids: Vec<String>,
    pub rename_address: Option<String>,
    pub cache_policy: CachePolicy,
    pub capture_policy: CapturePolicy,
    pub rehydrate_on_connect: bool,
    pub replay_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyFallbackStatus {
    pub route_id: String,
    pub direct_udp_targets: Vec<String>,
    pub available: bool,
    pub note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRouteAssessment {
    pub route_id: String,
    pub active: bool,
    pub direct_udp_fallback_available: bool,
    pub warning_count: usize,
    pub warnings: Vec<String>,
}

pub fn proxy_status_from_config(config: &BrokerConfig) -> Result<UdpProxyStatusSnapshot> {
    config.validate_runtime_references()?;

    let mut route_ids_by_ingress = config
        .udp_ingresses
        .iter()
        .map(|ingress| (ingress.id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut route_ids_by_destination = config
        .udp_destinations
        .iter()
        .map(|destination| (destination.id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut warnings = Vec::new();

    for route in &config.routes {
        if !route.enabled {
            continue;
        }

        if route.match_spec.ingress_ids.is_empty() {
            for route_ids in route_ids_by_ingress.values_mut() {
                route_ids.insert(route.id.clone());
            }
            warnings.push(format!(
                "route `{}` matches all ingresses; proxy sidecar scope is broad",
                route.id
            ));
        } else {
            for ingress_id in &route.match_spec.ingress_ids {
                if let Some(route_ids) = route_ids_by_ingress.get_mut(ingress_id) {
                    route_ids.insert(route.id.clone());
                }
            }
        }

        if route.match_spec.address_patterns.is_empty() {
            warnings.push(format!(
                "route `{}` matches all addresses; fallback analysis is broad",
                route.id
            ));
        }

        for destination in &route.destinations {
            if destination.transport != TransportSelector::OscUdp {
                continue;
            }
            if let Some(route_ids) = route_ids_by_destination.get_mut(&destination.target) {
                route_ids.insert(route.id.clone());
            }
        }
    }

    let ingresses = config
        .udp_ingresses
        .iter()
        .map(|ingress| UdpProxyIngressStatus {
            id: ingress.id.clone(),
            configured_bind: ingress.bind.clone(),
            bound_local_addr: None,
            route_ids: route_ids_by_ingress
                .get(&ingress.id)
                .map(|route_ids| route_ids.iter().cloned().collect())
                .unwrap_or_default(),
            max_packet_size: ingress.max_packet_size,
            mode: ingress.mode,
        })
        .collect::<Vec<_>>();

    let destinations = config
        .udp_destinations
        .iter()
        .map(|destination| UdpProxyDestinationStatus {
            id: destination.id.clone(),
            bind: destination.bind.clone(),
            target: destination.target.clone(),
            route_ids: route_ids_by_destination
                .get(&destination.id)
                .map(|route_ids| route_ids.iter().cloned().collect())
                .unwrap_or_default(),
            queue_depth: destination.policy.queue_depth,
            drop_policy: destination.policy.drop_policy,
            breaker_open_after_consecutive_failures: destination
                .policy
                .breaker
                .open_after_consecutive_failures,
            breaker_open_after_consecutive_queue_overflows: destination
                .policy
                .breaker
                .open_after_consecutive_queue_overflows,
            breaker_cooldown_ms: destination.policy.breaker.cooldown_ms,
        })
        .collect::<Vec<_>>();

    let destination_targets = config
        .udp_destinations
        .iter()
        .map(|destination| (destination.id.as_str(), destination.target.as_str()))
        .collect::<BTreeMap<_, _>>();

    let mut route_assessments = Vec::new();
    let routes = config
        .routes
        .iter()
        .map(|route| {
            let assessment = assess_route(route, &destination_targets);
            route_assessments.push(assessment);

            UdpProxyRouteStatus {
                id: route.id.clone(),
                enabled: route.enabled,
                mode: route.mode,
                traffic_class: route.class.clone(),
                ingress_ids: route.match_spec.ingress_ids.clone(),
                address_patterns: route.match_spec.address_patterns.clone(),
                destination_ids: route
                    .destinations
                    .iter()
                    .map(|destination| destination.target.clone())
                    .collect(),
                rename_address: route.transform.rename_address.clone(),
                cache_policy: route.cache.policy,
                capture_policy: route.observability.capture,
                rehydrate_on_connect: route.recovery.rehydrate_on_connect,
                replay_allowed: route.recovery.replay_allowed,
            }
        })
        .collect::<Vec<_>>();

    let fallback_routes = config
        .routes
        .iter()
        .filter(|route| route.enabled)
        .map(|route| {
            let direct_udp_targets = route
                .destinations
                .iter()
                .filter(|destination| destination.transport == TransportSelector::OscUdp)
                .filter_map(|destination| destination_targets.get(destination.target.as_str()))
                .map(|target| (*target).to_owned())
                .collect::<Vec<_>>();
            let available = !direct_udp_targets.is_empty();
            let note = if available {
                "If ROSC sidecar mode is unavailable, point the sender directly at these UDP targets.".to_owned()
            } else {
                "No direct UDP fallback target is available for this route yet.".to_owned()
            };

            UdpProxyFallbackStatus {
                route_id: route.id.clone(),
                direct_udp_targets,
                available,
                note,
            }
        })
        .collect::<Vec<_>>();

    let summary = UdpProxySummary {
        total_routes: routes.len(),
        active_routes: route_assessments
            .iter()
            .filter(|route| route.active)
            .count(),
        disabled_routes: route_assessments
            .iter()
            .filter(|route| !route.active)
            .count(),
        active_ingresses: ingresses
            .iter()
            .filter(|ingress| !ingress.route_ids.is_empty())
            .count(),
        active_destinations: destinations
            .iter()
            .filter(|destination| !destination.route_ids.is_empty())
            .count(),
        fallback_ready_routes: route_assessments
            .iter()
            .filter(|route| route.active && route.direct_udp_fallback_available)
            .count(),
        fallback_missing_routes: route_assessments
            .iter()
            .filter(|route| route.active && !route.direct_udp_fallback_available)
            .count(),
        warning_count: warnings.len()
            + route_assessments
                .iter()
                .map(|route| route.warning_count)
                .sum::<usize>(),
    };

    Ok(UdpProxyStatusSnapshot {
        launch_profile: ProxyLaunchProfileStatus::default(),
        summary,
        runtime: None,
        ingresses,
        destinations,
        routes,
        fallback_routes,
        route_assessments,
        warnings,
    })
}

pub fn attach_runtime_status(
    mut status: UdpProxyStatusSnapshot,
    snapshot: &HealthSnapshot,
) -> UdpProxyStatusSnapshot {
    status.runtime = Some(UdpProxyRuntimeStatus {
        traffic_frozen: snapshot.traffic_frozen,
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

fn assess_route(
    route: &rosc_route::RouteSpec,
    destination_targets: &BTreeMap<&str, &str>,
) -> UdpProxyRouteAssessment {
    let direct_udp_targets = route
        .destinations
        .iter()
        .filter(|destination| destination.transport == TransportSelector::OscUdp)
        .filter_map(|destination| destination_targets.get(destination.target.as_str()))
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if route.enabled && route.match_spec.ingress_ids.is_empty() {
        warnings.push("matches all ingresses".to_owned());
    }
    if route.enabled && route.match_spec.address_patterns.is_empty() {
        warnings.push("matches all addresses".to_owned());
    }
    if route.enabled && direct_udp_targets.is_empty() {
        warnings.push("no direct udp fallback target".to_owned());
    }
    if route.enabled
        && route.recovery.replay_allowed
        && route.observability.capture == CapturePolicy::Off
    {
        warnings.push("replay configured without capture visibility".to_owned());
    }

    UdpProxyRouteAssessment {
        route_id: route.id.clone(),
        active: route.enabled,
        direct_udp_fallback_available: route.enabled && !direct_udp_targets.is_empty(),
        warning_count: warnings.len(),
        warnings,
    }
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
