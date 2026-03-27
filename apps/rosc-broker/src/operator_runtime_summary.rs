use serde::Serialize;

use crate::{ProxyOperatorOverview, UdpProxyRuntimeStatus};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRuntimeSummary {
    pub has_runtime_status: bool,
    pub traffic_frozen: bool,
    pub isolated_route_count: usize,
    pub destination_queue_depth_total: usize,
    pub destinations_with_backlog: usize,
    pub destinations_with_open_breakers: usize,
    pub destinations_with_half_open_breakers: usize,
    pub recent_operator_action_count: usize,
    pub recent_config_event_count: usize,
    pub idle: bool,
}

pub fn proxy_operator_runtime_summary(
    overview: &ProxyOperatorOverview,
) -> ProxyOperatorRuntimeSummary {
    let Some(runtime) = overview.status.runtime.as_ref() else {
        return ProxyOperatorRuntimeSummary::default();
    };

    runtime_summary_from_status(runtime)
}

fn runtime_summary_from_status(runtime: &UdpProxyRuntimeStatus) -> ProxyOperatorRuntimeSummary {
    let destination_queue_depth_total = runtime.destinations.iter().map(|d| d.queue_depth).sum();
    let destinations_with_backlog = runtime
        .destinations
        .iter()
        .filter(|destination| destination.queue_depth > 0)
        .count();
    let destinations_with_open_breakers = runtime
        .destinations
        .iter()
        .filter(|destination| {
            matches!(
                destination.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::Open)
            )
        })
        .count();
    let destinations_with_half_open_breakers = runtime
        .destinations
        .iter()
        .filter(|destination| {
            matches!(
                destination.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::HalfOpen)
            )
        })
        .count();

    ProxyOperatorRuntimeSummary {
        has_runtime_status: true,
        traffic_frozen: runtime.traffic_frozen,
        isolated_route_count: runtime.isolated_route_ids.len(),
        destination_queue_depth_total,
        destinations_with_backlog,
        destinations_with_open_breakers,
        destinations_with_half_open_breakers,
        recent_operator_action_count: runtime.recent_operator_actions.len(),
        recent_config_event_count: runtime.recent_config_events.len(),
        idle: !runtime.traffic_frozen
            && runtime.isolated_route_ids.is_empty()
            && destination_queue_depth_total == 0
            && destinations_with_open_breakers == 0
            && destinations_with_half_open_breakers == 0,
    }
}
