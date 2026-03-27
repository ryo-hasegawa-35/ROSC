use rosc_telemetry::{BreakerStateSnapshot, RecentConfigEventKind};

use crate::UdpProxyStatusSnapshot;

use super::types::{
    ProxyOperatorDestinationSignal, ProxyOperatorHighlights, ProxyOperatorOverrides,
    ProxyOperatorRouteSignal, ProxyOperatorRuntimeSignals, ProxyOperatorState,
};

pub(super) fn operator_state(
    status: &UdpProxyStatusSnapshot,
    warnings: &[String],
    blockers: &[String],
) -> ProxyOperatorState {
    if !blockers.is_empty() {
        return ProxyOperatorState::Blocked;
    }

    let has_runtime_override = status
        .runtime
        .as_ref()
        .is_some_and(|runtime| runtime.traffic_frozen || !runtime.isolated_route_ids.is_empty());
    if !warnings.is_empty() || has_runtime_override {
        ProxyOperatorState::Warning
    } else {
        ProxyOperatorState::Healthy
    }
}

pub(super) fn operator_highlights(status: &UdpProxyStatusSnapshot) -> ProxyOperatorHighlights {
    let Some(runtime) = &status.runtime else {
        return ProxyOperatorHighlights::default();
    };

    ProxyOperatorHighlights {
        latest_operator_action: runtime.recent_operator_actions.last().cloned(),
        latest_config_issue: runtime
            .recent_config_events
            .iter()
            .rev()
            .find(|event| {
                !matches!(
                    event.kind,
                    RecentConfigEventKind::Applied | RecentConfigEventKind::LaunchProfileChanged
                )
            })
            .cloned(),
    }
}

pub(super) fn operator_overrides(status: &UdpProxyStatusSnapshot) -> ProxyOperatorOverrides {
    let traffic_frozen = status
        .runtime
        .as_ref()
        .map(|runtime| runtime.traffic_frozen)
        .unwrap_or(false);
    let isolated_route_ids = status
        .runtime
        .as_ref()
        .map(|runtime| runtime.isolated_route_ids.clone())
        .unwrap_or_default();
    ProxyOperatorOverrides {
        launch_profile_mode: status.launch_profile.mode.as_str().to_owned(),
        traffic_frozen,
        isolated_route_ids,
        disabled_capture_routes: status.launch_profile.disabled_capture_routes.clone(),
        disabled_replay_routes: status.launch_profile.disabled_replay_routes.clone(),
        disabled_restart_rehydrate_routes: status
            .launch_profile
            .disabled_restart_rehydrate_routes
            .clone(),
    }
}

pub(super) fn operator_runtime_signals(
    status: &UdpProxyStatusSnapshot,
    route_signals: &[ProxyOperatorRouteSignal],
    destination_signals: &[ProxyOperatorDestinationSignal],
) -> ProxyOperatorRuntimeSignals {
    let ingresses_with_drops = status
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .ingress_drops_total
                .iter()
                .filter(|(_, total)| **total > 0)
                .map(|(ingress_id, _)| ingress_id.clone())
                .collect()
        })
        .unwrap_or_default();

    ProxyOperatorRuntimeSignals {
        ingresses_with_drops,
        routes_with_dispatch_failures: route_signals
            .iter()
            .filter(|signal| signal.dispatch_failures_total > 0)
            .map(|signal| signal.route_id.clone())
            .collect(),
        routes_with_transform_failures: route_signals
            .iter()
            .filter(|signal| signal.transform_failures_total > 0)
            .map(|signal| signal.route_id.clone())
            .collect(),
        destinations_with_drops: destination_signals
            .iter()
            .filter(|signal| signal.drops_total > 0)
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_send_failures: destination_signals
            .iter()
            .filter(|signal| signal.send_failures_total > 0)
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_open_breakers: destination_signals
            .iter()
            .filter(|signal| signal.breaker_state == Some(BreakerStateSnapshot::Open))
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_half_open_breakers: destination_signals
            .iter()
            .filter(|signal| signal.breaker_state == Some(BreakerStateSnapshot::HalfOpen))
            .map(|signal| signal.destination_id.clone())
            .collect(),
    }
}

pub(super) fn operator_route_signals(
    status: &UdpProxyStatusSnapshot,
) -> Vec<ProxyOperatorRouteSignal> {
    let runtime = status.runtime.as_ref();
    status
        .route_assessments
        .iter()
        .map(|assessment| ProxyOperatorRouteSignal {
            route_id: assessment.route_id.clone(),
            active: assessment.active,
            isolated: runtime
                .is_some_and(|runtime| runtime.isolated_route_ids.contains(&assessment.route_id)),
            direct_udp_fallback_available: assessment.direct_udp_fallback_available,
            config_warnings: assessment.warnings.clone(),
            dispatch_failures_total: runtime
                .and_then(|runtime| runtime.dispatch_failures_total.get(&assessment.route_id))
                .copied()
                .unwrap_or_default(),
            transform_failures_total: runtime
                .and_then(|runtime| {
                    runtime
                        .route_transform_failures_total
                        .get(&assessment.route_id)
                })
                .copied()
                .unwrap_or_default(),
        })
        .collect()
}

pub(super) fn operator_destination_signals(
    status: &UdpProxyStatusSnapshot,
) -> Vec<ProxyOperatorDestinationSignal> {
    status
        .destinations
        .iter()
        .map(|destination| {
            let runtime = status.runtime.as_ref().and_then(|runtime| {
                runtime.destinations.iter().find(|runtime_destination| {
                    runtime_destination.destination_id == destination.id
                })
            });
            let drops_total = status
                .runtime
                .as_ref()
                .and_then(|runtime| runtime.destination_drops_total.get(&destination.id))
                .copied()
                .unwrap_or_default();
            ProxyOperatorDestinationSignal {
                destination_id: destination.id.clone(),
                queue_depth: runtime
                    .map(|runtime| runtime.queue_depth)
                    .unwrap_or_default(),
                send_total: runtime
                    .map(|runtime| runtime.send_total)
                    .unwrap_or_default(),
                send_failures_total: runtime
                    .map(|runtime| runtime.send_failures_total)
                    .unwrap_or_default(),
                drops_total,
                breaker_state: runtime.and_then(|runtime| runtime.breaker_state.clone()),
            }
        })
        .collect()
}

pub(super) fn recent_config_event_kind_label(kind: &RecentConfigEventKind) -> &'static str {
    match kind {
        RecentConfigEventKind::Applied => "applied",
        RecentConfigEventKind::Rejected => "rejected",
        RecentConfigEventKind::Blocked => "blocked",
        RecentConfigEventKind::ReloadFailed => "reload_failed",
        RecentConfigEventKind::LaunchProfileChanged => "launch_profile_changed",
    }
}
