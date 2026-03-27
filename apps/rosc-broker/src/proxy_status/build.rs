use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use rosc_config::BrokerConfig;
use rosc_route::{CapturePolicy, TransportSelector};

use crate::ProxyLaunchProfileStatus;

use super::types::{
    UdpProxyDestinationStatus, UdpProxyFallbackStatus, UdpProxyIngressStatus,
    UdpProxyRouteAssessment, UdpProxyRouteStatus, UdpProxyStatusSnapshot, UdpProxySummary,
};

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
