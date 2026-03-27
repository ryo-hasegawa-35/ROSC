use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::{ProxyOperatorReport, ProxyOperatorState};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorAttention {
    pub state: ProxyOperatorState,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub traffic_frozen: bool,
    pub isolated_route_ids: Vec<String>,
    pub latest_operator_action: Option<RecentOperatorAction>,
    pub latest_config_issue: Option<RecentConfigEvent>,
    pub problematic_route_ids: Vec<String>,
    pub problematic_destination_ids: Vec<String>,
    pub destinations_with_backlog: Vec<String>,
    pub destinations_with_open_breakers: Vec<String>,
    pub destinations_with_half_open_breakers: Vec<String>,
}

pub fn proxy_operator_attention(report: &ProxyOperatorReport) -> ProxyOperatorAttention {
    ProxyOperatorAttention {
        state: report.state.clone(),
        blockers: report.blockers.clone(),
        warnings: report.warnings.clone(),
        traffic_frozen: report.overrides.traffic_frozen,
        isolated_route_ids: report.overrides.isolated_route_ids.clone(),
        latest_operator_action: report.highlights.latest_operator_action.clone(),
        latest_config_issue: report.highlights.latest_config_issue.clone(),
        problematic_route_ids: report
            .route_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .map(|signal| signal.route_id.clone())
            .collect(),
        problematic_destination_ids: report
            .destination_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_backlog: report
            .destination_signals
            .iter()
            .filter(|signal| signal.queue_depth > 0)
            .map(|signal| signal.destination_id.clone())
            .collect(),
        destinations_with_open_breakers: report
            .runtime_signals
            .destinations_with_open_breakers
            .clone(),
        destinations_with_half_open_breakers: report
            .runtime_signals
            .destinations_with_half_open_breakers
            .clone(),
    }
}
