use std::collections::{BTreeMap, BTreeSet};

use rosc_config::DropPolicyConfig;
use rosc_route::{CachePolicy, CapturePolicy, TrafficClass};
use rosc_telemetry::BreakerStateSnapshot;
use serde::Serialize;

use crate::{
    ProxyOperatorRouteSignal, ProxyOperatorSnapshot, UdpProxyRouteAssessment, UdpProxyRouteStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorRouteDetailState {
    Healthy,
    Warning,
    Isolated,
    Disabled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteDetail {
    pub route_id: String,
    pub state: ProxyOperatorRouteDetailState,
    pub enabled: bool,
    pub isolated: bool,
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
    pub direct_udp_fallback_available: bool,
    pub direct_udp_targets: Vec<String>,
    pub warnings: Vec<String>,
    pub dispatch_failures_total: u64,
    pub transform_failures_total: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorDestinationDetailState {
    Healthy,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationDetail {
    pub destination_id: String,
    pub state: ProxyOperatorDestinationDetailState,
    pub bind: String,
    pub target: String,
    pub route_ids: Vec<String>,
    pub configured_queue_depth: usize,
    pub live_queue_depth: usize,
    pub drop_policy: DropPolicyConfig,
    pub breaker_open_after_consecutive_failures: u32,
    pub breaker_open_after_consecutive_queue_overflows: u32,
    pub breaker_cooldown_ms: u64,
    pub send_total: u64,
    pub send_failures_total: u64,
    pub drops_total: u64,
    pub breaker_state: Option<BreakerStateSnapshot>,
}

pub fn route_details_from_snapshot(
    snapshot: &ProxyOperatorSnapshot,
) -> Vec<ProxyOperatorRouteDetail> {
    let status = &snapshot.overview.status;
    let assessments = status
        .route_assessments
        .iter()
        .map(|assessment| (assessment.route_id.as_str(), assessment))
        .collect::<BTreeMap<_, _>>();
    let signals = snapshot
        .overview
        .report
        .route_signals
        .iter()
        .map(|signal| (signal.route_id.as_str(), signal))
        .collect::<BTreeMap<_, _>>();
    let fallbacks = status
        .fallback_routes
        .iter()
        .map(|fallback| (fallback.route_id.as_str(), fallback))
        .collect::<BTreeMap<_, _>>();

    status
        .routes
        .iter()
        .map(|route| {
            let assessment = assessments.get(route.id.as_str()).copied();
            let signal = signals.get(route.id.as_str()).copied();
            let fallback = fallbacks.get(route.id.as_str()).copied();
            let warnings = merge_route_warnings(assessment, signal);
            let isolated = signal.map(|entry| entry.isolated).unwrap_or(false);
            let state = route_detail_state(route, isolated, warnings.is_empty());

            ProxyOperatorRouteDetail {
                route_id: route.id.clone(),
                state,
                enabled: route.enabled,
                isolated,
                mode: route.mode,
                traffic_class: route.traffic_class.clone(),
                ingress_ids: route.ingress_ids.clone(),
                address_patterns: route.address_patterns.clone(),
                destination_ids: route.destination_ids.clone(),
                rename_address: route.rename_address.clone(),
                cache_policy: route.cache_policy,
                capture_policy: route.capture_policy,
                rehydrate_on_connect: route.rehydrate_on_connect,
                replay_allowed: route.replay_allowed,
                direct_udp_fallback_available: assessment
                    .map(|entry| entry.direct_udp_fallback_available)
                    .or_else(|| fallback.map(|entry| entry.available))
                    .unwrap_or(false),
                direct_udp_targets: fallback
                    .map(|entry| entry.direct_udp_targets.clone())
                    .unwrap_or_default(),
                warnings,
                dispatch_failures_total: signal
                    .map(|entry| entry.dispatch_failures_total)
                    .unwrap_or(0),
                transform_failures_total: signal
                    .map(|entry| entry.transform_failures_total)
                    .unwrap_or(0),
            }
        })
        .collect()
}

pub fn destination_details_from_snapshot(
    snapshot: &ProxyOperatorSnapshot,
) -> Vec<ProxyOperatorDestinationDetail> {
    let status = &snapshot.overview.status;
    let runtime = status
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .destinations
                .iter()
                .map(|entry| (entry.destination_id.as_str(), entry))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let signals = snapshot
        .overview
        .report
        .destination_signals
        .iter()
        .map(|signal| (signal.destination_id.as_str(), signal))
        .collect::<BTreeMap<_, _>>();

    status
        .destinations
        .iter()
        .map(|destination| {
            let runtime_entry = runtime.get(destination.id.as_str()).copied();
            let signal = signals.get(destination.id.as_str()).copied();
            let live_queue_depth = runtime_entry
                .map(|entry| entry.queue_depth)
                .unwrap_or(destination.queue_depth);
            let send_total = runtime_entry
                .map(|entry| entry.send_total)
                .unwrap_or_default();
            let send_failures_total = signal
                .map(|entry| entry.send_failures_total)
                .unwrap_or_default();
            let drops_total = signal.map(|entry| entry.drops_total).unwrap_or_default();
            let breaker_state = runtime_entry.and_then(|entry| entry.breaker_state.clone());
            let state = destination_detail_state(
                live_queue_depth,
                send_failures_total,
                drops_total,
                breaker_state.as_ref(),
            );

            ProxyOperatorDestinationDetail {
                destination_id: destination.id.clone(),
                state,
                bind: destination.bind.clone(),
                target: destination.target.clone(),
                route_ids: destination.route_ids.clone(),
                configured_queue_depth: destination.queue_depth,
                live_queue_depth,
                drop_policy: destination.drop_policy,
                breaker_open_after_consecutive_failures: destination
                    .breaker_open_after_consecutive_failures,
                breaker_open_after_consecutive_queue_overflows: destination
                    .breaker_open_after_consecutive_queue_overflows,
                breaker_cooldown_ms: destination.breaker_cooldown_ms,
                send_total,
                send_failures_total,
                drops_total,
                breaker_state,
            }
        })
        .collect()
}

fn merge_route_warnings(
    assessment: Option<&UdpProxyRouteAssessment>,
    signal: Option<&ProxyOperatorRouteSignal>,
) -> Vec<String> {
    let mut warnings = BTreeSet::new();
    if let Some(assessment) = assessment {
        warnings.extend(assessment.warnings.iter().cloned());
    }
    if let Some(signal) = signal {
        warnings.extend(signal.config_warnings.iter().cloned());
        if signal.dispatch_failures_total > 0 {
            warnings.insert(format!(
                "dispatch failures observed ({})",
                signal.dispatch_failures_total
            ));
        }
        if signal.transform_failures_total > 0 {
            warnings.insert(format!(
                "transform failures observed ({})",
                signal.transform_failures_total
            ));
        }
    }
    warnings.into_iter().collect()
}

fn route_detail_state(
    route: &UdpProxyRouteStatus,
    isolated: bool,
    warnings_empty: bool,
) -> ProxyOperatorRouteDetailState {
    if !route.enabled {
        return ProxyOperatorRouteDetailState::Disabled;
    }
    if isolated {
        return ProxyOperatorRouteDetailState::Isolated;
    }
    if warnings_empty {
        ProxyOperatorRouteDetailState::Healthy
    } else {
        ProxyOperatorRouteDetailState::Warning
    }
}

fn destination_detail_state(
    live_queue_depth: usize,
    send_failures_total: u64,
    drops_total: u64,
    breaker_state: Option<&BreakerStateSnapshot>,
) -> ProxyOperatorDestinationDetailState {
    if matches!(breaker_state, Some(BreakerStateSnapshot::Open)) {
        return ProxyOperatorDestinationDetailState::Blocked;
    }
    if live_queue_depth > 0
        || send_failures_total > 0
        || drops_total > 0
        || matches!(breaker_state, Some(BreakerStateSnapshot::HalfOpen))
    {
        ProxyOperatorDestinationDetailState::Warning
    } else {
        ProxyOperatorDestinationDetailState::Healthy
    }
}
