use serde::Serialize;

use crate::{
    ProxyOperatorRouteSignal, ProxyOperatorSnapshot, ProxyOperatorState,
    ProxyOperatorSuggestedAction, ProxyOperatorSuggestedActionKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorIncidentLevel {
    Blocked,
    Degraded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorIncidentScope {
    Global,
    Config,
    Route,
    Destination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorIncidentCluster {
    pub id: String,
    pub scope: ProxyOperatorIncidentScope,
    pub level: ProxyOperatorIncidentLevel,
    pub title: String,
    pub summary: String,
    pub route_id: Option<String>,
    pub destination_id: Option<String>,
    pub reasons: Vec<String>,
    pub action: Option<ProxyOperatorSuggestedAction>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorIncidentDigest {
    pub state: String,
    pub blocked_count: usize,
    pub degraded_count: usize,
    pub clusters: Vec<ProxyOperatorIncidentCluster>,
}

pub fn proxy_operator_incident_digest(
    snapshot: &ProxyOperatorSnapshot,
) -> ProxyOperatorIncidentDigest {
    let mut clusters = Vec::new();

    if !snapshot.overview.report.blockers.is_empty() {
        clusters.push(ProxyOperatorIncidentCluster {
            id: "config-blockers".to_owned(),
            scope: ProxyOperatorIncidentScope::Config,
            level: ProxyOperatorIncidentLevel::Blocked,
            title: "Configuration blockers".to_owned(),
            summary: format!(
                "{} blocker(s) are preventing a clean ready state.",
                snapshot.overview.report.blockers.len()
            ),
            route_id: None,
            destination_id: None,
            reasons: snapshot.overview.report.blockers.clone(),
            action: None,
        });
    }

    if snapshot.overview.report.overrides.traffic_frozen {
        clusters.push(ProxyOperatorIncidentCluster {
            id: "traffic-frozen".to_owned(),
            scope: ProxyOperatorIncidentScope::Global,
            level: ProxyOperatorIncidentLevel::Degraded,
            title: "Traffic frozen".to_owned(),
            summary: "Live dispatch is paused under an operator override.".to_owned(),
            route_id: None,
            destination_id: None,
            reasons: vec![
                "traffic is currently frozen by operator override".to_owned(),
                "live packets remain queued until thaw".to_owned(),
            ],
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::ThawTraffic,
                label: "Thaw traffic".to_owned(),
                route_id: None,
                destination_id: None,
            }),
        });
    }

    for route in &snapshot.incidents.problematic_routes {
        clusters.push(route_cluster(route));
    }

    for destination in &snapshot.incidents.problematic_destinations {
        clusters.push(ProxyOperatorIncidentCluster {
            id: format!("destination:{}", destination.destination_id),
            scope: ProxyOperatorIncidentScope::Destination,
            level: if matches!(
                destination.breaker_state,
                Some(rosc_telemetry::BreakerStateSnapshot::Open)
            ) {
                ProxyOperatorIncidentLevel::Blocked
            } else {
                ProxyOperatorIncidentLevel::Degraded
            },
            title: format!(
                "Destination `{}` needs recovery",
                destination.destination_id
            ),
            summary: "Destination health is degraded by queue, drop, or breaker pressure."
                .to_owned(),
            route_id: None,
            destination_id: Some(destination.destination_id.clone()),
            reasons: destination_reasons(destination),
            action: Some(ProxyOperatorSuggestedAction {
                kind: ProxyOperatorSuggestedActionKind::RehydrateDestination,
                label: "Rehydrate destination".to_owned(),
                route_id: None,
                destination_id: Some(destination.destination_id.clone()),
            }),
        });
    }

    clusters.sort_by_key(|cluster| {
        (
            cluster.level,
            cluster.scope_order(),
            cluster.id.to_ascii_lowercase(),
        )
    });

    ProxyOperatorIncidentDigest {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        blocked_count: clusters
            .iter()
            .filter(|cluster| cluster.level == ProxyOperatorIncidentLevel::Blocked)
            .count(),
        degraded_count: clusters
            .iter()
            .filter(|cluster| cluster.level == ProxyOperatorIncidentLevel::Degraded)
            .count(),
        clusters,
    }
}

impl ProxyOperatorIncidentCluster {
    fn scope_order(&self) -> u8 {
        match self.scope {
            ProxyOperatorIncidentScope::Global => 0,
            ProxyOperatorIncidentScope::Config => 1,
            ProxyOperatorIncidentScope::Route => 2,
            ProxyOperatorIncidentScope::Destination => 3,
        }
    }
}

fn route_cluster(signal: &ProxyOperatorRouteSignal) -> ProxyOperatorIncidentCluster {
    let isolated = signal.isolated;
    ProxyOperatorIncidentCluster {
        id: format!("route:{}", signal.route_id),
        scope: ProxyOperatorIncidentScope::Route,
        level: if !signal.direct_udp_fallback_available {
            ProxyOperatorIncidentLevel::Blocked
        } else {
            ProxyOperatorIncidentLevel::Degraded
        },
        title: if isolated {
            format!("Route `{}` is isolated", signal.route_id)
        } else {
            format!("Route `{}` needs attention", signal.route_id)
        },
        summary: if isolated {
            "This route is intentionally isolated and not forwarding live traffic.".to_owned()
        } else {
            "This route is reporting operator-visible issues.".to_owned()
        },
        route_id: Some(signal.route_id.clone()),
        destination_id: None,
        reasons: route_reasons(signal),
        action: Some(ProxyOperatorSuggestedAction {
            kind: if isolated {
                ProxyOperatorSuggestedActionKind::RestoreRoute
            } else {
                ProxyOperatorSuggestedActionKind::FocusRoute
            },
            label: if isolated {
                "Restore route".to_owned()
            } else {
                "Focus route".to_owned()
            },
            route_id: Some(signal.route_id.clone()),
            destination_id: None,
        }),
    }
}

fn route_reasons(signal: &ProxyOperatorRouteSignal) -> Vec<String> {
    let mut reasons = Vec::new();
    if signal.isolated {
        reasons.push("route is currently isolated".to_owned());
    }
    if !signal.direct_udp_fallback_available {
        reasons.push("route is missing direct UDP fallback coverage".to_owned());
    }
    reasons.extend(signal.config_warnings.iter().cloned());
    if signal.dispatch_failures_total > 0 {
        reasons.push(format!(
            "dispatch failures observed ({})",
            signal.dispatch_failures_total
        ));
    }
    if signal.transform_failures_total > 0 {
        reasons.push(format!(
            "transform failures observed ({})",
            signal.transform_failures_total
        ));
    }
    if reasons.is_empty() {
        reasons.push("route is marked problematic by operator policy".to_owned());
    }
    reasons
}

fn destination_reasons(signal: &crate::ProxyOperatorDestinationSignal) -> Vec<String> {
    let mut reasons = Vec::new();
    if signal.queue_depth > 0 {
        reasons.push(format!("queue backlog observed ({})", signal.queue_depth));
    }
    if signal.send_failures_total > 0 {
        reasons.push(format!(
            "send failures observed ({})",
            signal.send_failures_total
        ));
    }
    if signal.drops_total > 0 {
        reasons.push(format!("drops observed ({})", signal.drops_total));
    }
    match signal.breaker_state {
        Some(rosc_telemetry::BreakerStateSnapshot::Open) => {
            reasons.push("destination breaker is open".to_owned())
        }
        Some(rosc_telemetry::BreakerStateSnapshot::HalfOpen) => {
            reasons.push("destination breaker is half-open".to_owned())
        }
        Some(rosc_telemetry::BreakerStateSnapshot::Closed) | None => {}
    }
    if reasons.is_empty() {
        reasons.push("destination is marked problematic by operator policy".to_owned());
    }
    reasons
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
