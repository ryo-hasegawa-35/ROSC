use serde::Serialize;

use crate::{
    ProxyOperatorSnapshot, ProxyOperatorSuggestedAction, ProxyOperatorTraceCatalog,
    ProxyOperatorTraceEvent, ProxyOperatorTraceEventLevel, proxy_operator_trace,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteHandoff {
    pub route_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub next_steps: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationHandoff {
    pub destination_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub next_steps: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorHandoffCatalog {
    pub state: String,
    pub route_handoffs: Vec<ProxyOperatorRouteHandoff>,
    pub destination_handoffs: Vec<ProxyOperatorDestinationHandoff>,
}

pub fn proxy_operator_handoff(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorHandoffCatalog {
    proxy_operator_handoff_from_trace(snapshot, proxy_operator_trace(snapshot))
}

pub fn proxy_operator_handoff_from_trace(
    snapshot: &ProxyOperatorSnapshot,
    trace: ProxyOperatorTraceCatalog,
) -> ProxyOperatorHandoffCatalog {
    let route_handoffs = trace
        .routes
        .into_iter()
        .map(|trace| ProxyOperatorRouteHandoff {
            route_id: trace.route_id.clone(),
            level: trace.level,
            summary: trace.summary.clone(),
            next_steps: route_next_steps(&trace, snapshot.overview.report.overrides.traffic_frozen),
            actions: trace.actions.clone(),
            recent_events: trace.recent_events.clone(),
        })
        .collect::<Vec<_>>();

    let destination_handoffs = trace
        .destinations
        .into_iter()
        .map(|trace| ProxyOperatorDestinationHandoff {
            destination_id: trace.destination_id.clone(),
            level: trace.level,
            summary: trace.summary.clone(),
            next_steps: destination_next_steps(
                &trace,
                snapshot.overview.report.overrides.traffic_frozen,
            ),
            actions: trace.actions.clone(),
            recent_events: trace.recent_events.clone(),
        })
        .collect::<Vec<_>>();

    ProxyOperatorHandoffCatalog {
        state: match snapshot.readiness.level {
            crate::ProxyOperatorReadinessLevel::Ready => "ready",
            crate::ProxyOperatorReadinessLevel::Degraded => "degraded",
            crate::ProxyOperatorReadinessLevel::Blocked => "blocked",
        }
        .to_owned(),
        route_handoffs,
        destination_handoffs,
    }
}

fn route_next_steps(trace: &crate::ProxyOperatorRouteTrace, traffic_frozen: bool) -> Vec<String> {
    let mut steps = Vec::new();
    if traffic_frozen {
        steps.push(
            "Thaw traffic before expecting this route to resume live forwarding or recovery."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("operator isolation"))
    {
        steps.push(
            "Confirm the route can resume live forwarding, then restore isolation.".to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("fallback"))
    {
        steps.push(
            "Add or verify direct UDP fallback coverage before relying on recovery automation."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("dispatch failures"))
    {
        steps.push("Inspect downstream destination health and breaker state before replaying or rehydrating.".to_owned());
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("transform failures"))
    {
        steps.push(
            "Validate route transform and payload shape against the expected OSC contract."
                .to_owned(),
        );
    }
    if steps.is_empty() {
        steps.push(
            "Monitor this route and keep the focus view open for repeated runtime pressure."
                .to_owned(),
        );
    }
    steps
}

fn destination_next_steps(
    trace: &crate::ProxyOperatorDestinationTrace,
    traffic_frozen: bool,
) -> Vec<String> {
    let mut steps = Vec::new();
    if traffic_frozen {
        steps.push(
            "Thaw traffic before treating this destination as fully recovered or stable."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("breaker is open"))
    {
        steps.push(
            "Verify the downstream target is reachable before attempting rehydrate.".to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("breaker is half-open"))
    {
        steps.push(
            "Keep traffic scoped and observe the next breaker probe before widening load."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("queue backlog"))
    {
        steps.push(
            "Reduce ingress pressure or increase queue headroom so backlog can drain cleanly."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("send failures"))
    {
        steps.push(
            "Inspect socket/target health and confirm the destination process is still alive."
                .to_owned(),
        );
    }
    if trace
        .open_reasons
        .iter()
        .any(|reason| reason.contains("drops observed"))
    {
        steps.push(
            "Review queue policy and packet rate so drops stop before replay or rehydrate."
                .to_owned(),
        );
    }
    if steps.is_empty() {
        steps.push(
            "Destination looks stable now; keep it under observation instead of forcing recovery."
                .to_owned(),
        );
    }
    steps
}
