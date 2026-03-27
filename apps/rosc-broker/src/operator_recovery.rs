use serde::Serialize;

use crate::{
    ProxyOperatorSnapshot, ProxyOperatorSuggestedAction, ProxyOperatorSuggestedActionKind,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRecoveryRoute {
    pub route_id: String,
    pub cache_policy: String,
    pub capture_policy: String,
    pub rehydrate_on_connect: bool,
    pub replay_allowed: bool,
    pub isolated: bool,
    pub destination_ids: Vec<String>,
    pub fallback_ready: bool,
    pub action: ProxyOperatorSuggestedAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRecoveryDestination {
    pub destination_id: String,
    pub route_ids: Vec<String>,
    pub queue_depth: usize,
    pub send_failures_total: u64,
    pub drops_total: u64,
    pub breaker_state: String,
    pub action: ProxyOperatorSuggestedAction,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRecovery {
    pub cached_routes: usize,
    pub replayable_routes: usize,
    pub rehydrate_ready_destinations: usize,
    pub route_candidates: Vec<ProxyOperatorRecoveryRoute>,
    pub destination_candidates: Vec<ProxyOperatorRecoveryDestination>,
}

pub fn proxy_operator_recovery(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorRecovery {
    let runtime_destinations = snapshot
        .overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .destinations
                .iter()
                .map(|destination| (destination.destination_id.as_str(), destination))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let destination_signals = snapshot
        .overview
        .report
        .destination_signals
        .iter()
        .map(|destination| (destination.destination_id.as_str(), destination))
        .collect::<std::collections::BTreeMap<_, _>>();

    let route_candidates = snapshot
        .overview
        .status
        .routes
        .iter()
        .filter(|route| {
            route.cache_policy != rosc_route::CachePolicy::NoCache || route.replay_allowed
        })
        .map(|route| ProxyOperatorRecoveryRoute {
            route_id: route.id.clone(),
            cache_policy: format!("{:?}", route.cache_policy),
            capture_policy: format!("{:?}", route.capture_policy),
            rehydrate_on_connect: route.rehydrate_on_connect,
            replay_allowed: route.replay_allowed,
            isolated: snapshot
                .overview
                .report
                .overrides
                .isolated_route_ids
                .contains(&route.id),
            destination_ids: route.destination_ids.clone(),
            fallback_ready: snapshot
                .overview
                .status
                .route_assessments
                .iter()
                .find(|assessment| assessment.route_id == route.id)
                .map(|assessment| assessment.direct_udp_fallback_available)
                .unwrap_or(false),
            action: ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::FocusRoute,
                label: "Focus route".to_owned(),
                route_id: Some(route.id.clone()),
                destination_id: None,
            },
        })
        .collect::<Vec<_>>();

    let destination_candidates = snapshot
        .overview
        .status
        .destinations
        .iter()
        .filter_map(|destination| {
            let runtime = runtime_destinations.get(destination.id.as_str()).copied();
            let signal = destination_signals.get(destination.id.as_str()).copied();
            let queue_depth = runtime
                .map(|runtime| runtime.queue_depth)
                .unwrap_or(destination.queue_depth);
            let send_failures_total = signal
                .map(|signal| signal.send_failures_total)
                .unwrap_or_default();
            let drops_total = signal.map(|signal| signal.drops_total).unwrap_or_default();
            let breaker_state = runtime.and_then(|runtime| runtime.breaker_state.clone());
            if queue_depth == 0
                && send_failures_total == 0
                && drops_total == 0
                && breaker_state.is_none()
            {
                return None;
            }

            Some(ProxyOperatorRecoveryDestination {
                destination_id: destination.id.clone(),
                route_ids: destination.route_ids.clone(),
                queue_depth,
                send_failures_total,
                drops_total,
                breaker_state: breaker_state
                    .as_ref()
                    .map(|state| format!("{state:?}"))
                    .unwrap_or_else(|| "closed".to_owned()),
                action: ProxyOperatorSuggestedAction {
                    kind: ProxyOperatorSuggestedActionKind::RehydrateDestination,
                    label: "Rehydrate destination".to_owned(),
                    route_id: None,
                    destination_id: Some(destination.id.clone()),
                },
            })
        })
        .collect::<Vec<_>>();

    ProxyOperatorRecovery {
        cached_routes: route_candidates
            .iter()
            .filter(|route| route.cache_policy != "NoCache")
            .count(),
        replayable_routes: route_candidates
            .iter()
            .filter(|route| route.replay_allowed)
            .count(),
        rehydrate_ready_destinations: destination_candidates.len(),
        route_candidates,
        destination_candidates,
    }
}
