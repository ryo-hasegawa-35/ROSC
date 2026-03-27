use rosc_telemetry::{RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction};
use serde::Serialize;

use crate::{
    ProxyOperatorDestinationSignal, ProxyOperatorReport, ProxyOperatorRouteSignal,
    ProxyOperatorState,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorIncidents {
    pub state: ProxyOperatorState,
    pub open_blockers: Vec<String>,
    pub open_warnings: Vec<String>,
    pub latest_operator_action: Option<RecentOperatorAction>,
    pub latest_config_issue: Option<RecentConfigEvent>,
    pub recent_operator_actions: Vec<RecentOperatorAction>,
    pub recent_config_issues: Vec<RecentConfigEvent>,
    pub problematic_routes: Vec<ProxyOperatorRouteSignal>,
    pub problematic_destinations: Vec<ProxyOperatorDestinationSignal>,
}

pub fn proxy_operator_incidents(
    report: &ProxyOperatorReport,
    history_limit: Option<usize>,
) -> ProxyOperatorIncidents {
    proxy_operator_incidents_from_histories(report, Vec::new(), Vec::new(), history_limit)
}

pub fn proxy_operator_incidents_from_histories(
    report: &ProxyOperatorReport,
    recent_operator_actions: Vec<RecentOperatorAction>,
    recent_config_events: Vec<RecentConfigEvent>,
    history_limit: Option<usize>,
) -> ProxyOperatorIncidents {
    let filtered_config_issues = recent_config_events
        .into_iter()
        .filter(|event| {
            !matches!(
                event.kind,
                RecentConfigEventKind::Applied | RecentConfigEventKind::LaunchProfileChanged
            )
        })
        .collect::<Vec<_>>();

    ProxyOperatorIncidents {
        state: report.state.clone(),
        open_blockers: report.blockers.clone(),
        open_warnings: report.warnings.clone(),
        latest_operator_action: report.highlights.latest_operator_action.clone(),
        latest_config_issue: report.highlights.latest_config_issue.clone(),
        recent_operator_actions: bounded_recent(recent_operator_actions, history_limit),
        recent_config_issues: bounded_recent(filtered_config_issues, history_limit),
        problematic_routes: report
            .route_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .cloned()
            .collect(),
        problematic_destinations: report
            .destination_signals
            .iter()
            .filter(|signal| signal.is_problematic())
            .cloned()
            .collect(),
    }
}

fn bounded_recent<T>(entries: Vec<T>, limit: Option<usize>) -> Vec<T> {
    match limit {
        Some(limit) if entries.len() > limit => {
            let start = entries.len() - limit;
            entries.into_iter().skip(start).collect()
        }
        _ => entries,
    }
}
