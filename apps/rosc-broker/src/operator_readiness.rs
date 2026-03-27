use serde::Serialize;

use crate::{
    ProxyOperatorOverview, ProxyOperatorRuntimeSummary, ProxyOperatorState,
    ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot, proxy_operator_overview,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorReadinessLevel {
    Ready,
    Degraded,
    Blocked,
}

impl ProxyOperatorReadinessLevel {
    pub fn is_acceptable(self, allow_degraded: bool) -> bool {
        matches!(self, Self::Ready) || (allow_degraded && matches!(self, Self::Degraded))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorReadinessFlags {
    pub control_plane_ready: bool,
    pub traffic_flow_ready: bool,
    pub fallback_complete: bool,
    pub operator_intervention_required: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorReadinessCounts {
    pub total_routes: usize,
    pub active_routes: usize,
    pub active_ingresses: usize,
    pub active_destinations: usize,
    pub fallback_ready_routes: usize,
    pub fallback_missing_routes: usize,
    pub problematic_routes: usize,
    pub problematic_destinations: usize,
    pub blockers: usize,
    pub warnings: usize,
    pub isolated_routes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorReadiness {
    pub level: ProxyOperatorReadinessLevel,
    pub ready: bool,
    pub state: ProxyOperatorState,
    pub flags: ProxyOperatorReadinessFlags,
    pub launch_profile_mode: String,
    pub counts: ProxyOperatorReadinessCounts,
    pub reasons: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub runtime_summary: ProxyOperatorRuntimeSummary,
}

impl ProxyOperatorReadiness {
    pub fn is_acceptable(&self, allow_degraded: bool) -> bool {
        self.level.is_acceptable(allow_degraded)
    }
}

pub fn proxy_operator_readiness(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
) -> ProxyOperatorReadiness {
    proxy_operator_readiness_from_overview(proxy_operator_overview(status, policy))
}

pub fn proxy_operator_readiness_from_overview(
    overview: ProxyOperatorOverview,
) -> ProxyOperatorReadiness {
    let problematic_routes = overview.problematic_signals.route_signals.len();
    let problematic_destinations = overview.problematic_signals.destination_signals.len();
    let isolated_routes = overview.report.overrides.isolated_route_ids.len();

    let flags = ProxyOperatorReadinessFlags {
        control_plane_ready: overview.report.state != ProxyOperatorState::Blocked,
        traffic_flow_ready: !overview.report.overrides.traffic_frozen && isolated_routes == 0,
        fallback_complete: overview.status.summary.fallback_missing_routes == 0,
        operator_intervention_required: overview.report.state == ProxyOperatorState::Blocked
            || overview.report.overrides.traffic_frozen
            || isolated_routes > 0
            || problematic_routes > 0
            || problematic_destinations > 0,
    };

    let counts = ProxyOperatorReadinessCounts {
        total_routes: overview.status.summary.total_routes,
        active_routes: overview.status.summary.active_routes,
        active_ingresses: overview.status.summary.active_ingresses,
        active_destinations: overview.status.summary.active_destinations,
        fallback_ready_routes: overview.status.summary.fallback_ready_routes,
        fallback_missing_routes: overview.status.summary.fallback_missing_routes,
        problematic_routes,
        problematic_destinations,
        blockers: overview.report.blockers.len(),
        warnings: overview.report.warnings.len(),
        isolated_routes,
    };

    let level = if !flags.control_plane_ready {
        ProxyOperatorReadinessLevel::Blocked
    } else if flags.operator_intervention_required || counts.warnings > 0 {
        ProxyOperatorReadinessLevel::Degraded
    } else {
        ProxyOperatorReadinessLevel::Ready
    };

    ProxyOperatorReadiness {
        ready: level == ProxyOperatorReadinessLevel::Ready,
        state: overview.report.state.clone(),
        launch_profile_mode: overview.report.overrides.launch_profile_mode.clone(),
        reasons: readiness_reasons(&overview, &counts, &flags),
        blockers: overview.report.blockers,
        warnings: overview.report.warnings,
        runtime_summary: overview.runtime_summary,
        level,
        flags,
        counts,
    }
}

fn readiness_reasons(
    overview: &ProxyOperatorOverview,
    counts: &ProxyOperatorReadinessCounts,
    flags: &ProxyOperatorReadinessFlags,
) -> Vec<String> {
    let mut reasons = Vec::new();

    if !flags.control_plane_ready {
        reasons.extend(overview.report.blockers.iter().cloned());
    } else {
        reasons.extend(overview.report.warnings.iter().cloned());
    }

    if overview.report.overrides.traffic_frozen {
        reasons.push("traffic is currently frozen by operator override".to_owned());
    }

    if counts.isolated_routes > 0 {
        reasons.push(format!(
            "{} route(s) are currently isolated",
            counts.isolated_routes
        ));
    }

    if counts.problematic_routes > 0 {
        reasons.push(format!(
            "{} route(s) currently report problematic operator signals",
            counts.problematic_routes
        ));
    }

    if counts.problematic_destinations > 0 {
        reasons.push(format!(
            "{} destination(s) currently report problematic operator signals",
            counts.problematic_destinations
        ));
    }

    if !flags.fallback_complete {
        reasons.push(format!(
            "{} active route(s) are missing direct UDP fallback targets",
            counts.fallback_missing_routes
        ));
    }

    if reasons.is_empty() {
        reasons.push("no active readiness blockers or warnings".to_owned());
    }

    reasons
}
