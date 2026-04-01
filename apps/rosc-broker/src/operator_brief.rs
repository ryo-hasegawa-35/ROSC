use serde::Serialize;

use crate::{
    ProxyOperatorDashboard, ProxyOperatorDestinationCasebook, ProxyOperatorDestinationFocusPacket,
    ProxyOperatorDestinationHandoff, ProxyOperatorDestinationLens, ProxyOperatorDestinationTriage,
    ProxyOperatorRouteCasebook, ProxyOperatorRouteFocusPacket, ProxyOperatorRouteHandoff,
    ProxyOperatorRouteLens, ProxyOperatorRouteTriage, ProxyOperatorState,
    ProxyOperatorSuggestedAction,
    operator_context::{
        dashboard_context, destination_state, route_state, scoped_destination_blockers,
        scoped_route_blockers,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteBrief {
    pub route_id: String,
    pub state: String,
    pub summary: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub headline_timeline: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub focus: ProxyOperatorRouteFocusPacket,
    pub lens: Option<ProxyOperatorRouteLens>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationBrief {
    pub destination_id: String,
    pub state: String,
    pub summary: String,
    pub global_blockers: Vec<String>,
    pub scoped_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub headline_timeline: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub focus: ProxyOperatorDestinationFocusPacket,
    pub lens: Option<ProxyOperatorDestinationLens>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorBriefCatalog {
    pub state: String,
    pub global_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub global_next_steps: Vec<String>,
    pub routes: Vec<ProxyOperatorRouteBrief>,
    pub destinations: Vec<ProxyOperatorDestinationBrief>,
}

pub fn proxy_operator_brief_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorBriefCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);
    let global_next_steps = snapshot.triage.global.next_steps.clone();

    let routes = dashboard
        .focus
        .routes
        .iter()
        .cloned()
        .map(|focus| {
            let lens = dashboard
                .lens
                .routes
                .iter()
                .find(|entry| entry.route_id == focus.route_id)
                .cloned();
            let handoff = focus.handoff.as_ref();
            let triage = focus.triage.as_ref();
            let casebook = focus.casebook.as_ref();

            ProxyOperatorRouteBrief {
                route_id: focus.route_id.clone(),
                state: state_label(route_state(snapshot, &focus.route_id)).to_owned(),
                summary: route_summary(&focus, handoff, triage, casebook),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_route_blockers(snapshot, &focus.route_id),
                global_overrides: context.global_overrides.clone(),
                headline_timeline: route_timeline_headlines(&focus),
                next_steps: route_next_steps(handoff, triage, casebook),
                recommended_actions: route_actions(handoff, triage, casebook),
                focus,
                lens,
            }
        })
        .collect();

    let destinations = dashboard
        .focus
        .destinations
        .iter()
        .cloned()
        .map(|focus| {
            let lens = dashboard
                .lens
                .destinations
                .iter()
                .find(|entry| entry.destination_id == focus.destination_id)
                .cloned();
            let handoff = focus.handoff.as_ref();
            let triage = focus.triage.as_ref();
            let casebook = focus.casebook.as_ref();

            ProxyOperatorDestinationBrief {
                destination_id: focus.destination_id.clone(),
                state: state_label(destination_state(snapshot, &focus.destination_id)).to_owned(),
                summary: destination_summary(&focus, handoff, triage, casebook),
                global_blockers: context.global_blockers.clone(),
                scoped_blockers: scoped_destination_blockers(snapshot, &focus.destination_id),
                global_overrides: context.global_overrides.clone(),
                headline_timeline: destination_timeline_headlines(&focus),
                next_steps: destination_next_steps(handoff, triage, casebook),
                recommended_actions: destination_actions(handoff, triage, casebook),
                focus,
                lens,
            }
        })
        .collect();

    ProxyOperatorBriefCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global_blockers: context.global_blockers,
        global_overrides: context.global_overrides,
        global_next_steps,
        routes,
        destinations,
    }
}

fn route_summary(
    focus: &ProxyOperatorRouteFocusPacket,
    handoff: Option<&ProxyOperatorRouteHandoff>,
    triage: Option<&ProxyOperatorRouteTriage>,
    casebook: Option<&ProxyOperatorRouteCasebook>,
) -> String {
    handoff
        .map(|entry| entry.summary.clone())
        .or_else(|| triage.map(|entry| entry.summary.clone()))
        .or_else(|| casebook.map(|entry| entry.summary.clone()))
        .unwrap_or_else(|| {
            format!(
                "{} destination(s), {} warning(s), fallback_ready={}",
                focus.detail.destination_ids.len(),
                focus.detail.warnings.len(),
                focus.detail.direct_udp_fallback_available
            )
        })
}

fn destination_summary(
    focus: &ProxyOperatorDestinationFocusPacket,
    handoff: Option<&ProxyOperatorDestinationHandoff>,
    triage: Option<&ProxyOperatorDestinationTriage>,
    casebook: Option<&ProxyOperatorDestinationCasebook>,
) -> String {
    handoff
        .map(|entry| entry.summary.clone())
        .or_else(|| triage.map(|entry| entry.summary.clone()))
        .or_else(|| casebook.map(|entry| entry.summary.clone()))
        .unwrap_or_else(|| {
            format!(
                "queue_depth={}, send_failures={}, drops={}, linked_routes={}",
                focus.detail.live_queue_depth,
                focus.detail.send_failures_total,
                focus.detail.drops_total,
                focus.detail.route_ids.len()
            )
        })
}

fn route_timeline_headlines(focus: &ProxyOperatorRouteFocusPacket) -> Vec<String> {
    let mut headlines = focus
        .timeline
        .as_ref()
        .map(|timeline| {
            timeline
                .entries
                .iter()
                .take(5)
                .map(|entry| entry.label.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if headlines.is_empty() {
        extend_unique_strings(
            &mut headlines,
            focus
                .trace
                .as_ref()
                .map(|trace| trace.recent_events.iter().map(|event| &event.title)),
        );
    }
    headlines
}

fn destination_timeline_headlines(focus: &ProxyOperatorDestinationFocusPacket) -> Vec<String> {
    let mut headlines = focus
        .timeline
        .as_ref()
        .map(|timeline| {
            timeline
                .entries
                .iter()
                .take(5)
                .map(|entry| entry.label.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if headlines.is_empty() {
        extend_unique_strings(
            &mut headlines,
            focus
                .trace
                .as_ref()
                .map(|trace| trace.recent_events.iter().map(|event| &event.title)),
        );
    }
    headlines
}

fn route_next_steps(
    handoff: Option<&ProxyOperatorRouteHandoff>,
    triage: Option<&ProxyOperatorRouteTriage>,
    casebook: Option<&ProxyOperatorRouteCasebook>,
) -> Vec<String> {
    let mut steps = Vec::new();
    extend_unique_strings(&mut steps, handoff.map(|entry| entry.next_steps.iter()));
    extend_unique_strings(&mut steps, triage.map(|entry| entry.next_steps.iter()));
    extend_unique_strings(&mut steps, casebook.map(|entry| entry.next_steps.iter()));
    steps
}

fn destination_next_steps(
    handoff: Option<&ProxyOperatorDestinationHandoff>,
    triage: Option<&ProxyOperatorDestinationTriage>,
    casebook: Option<&ProxyOperatorDestinationCasebook>,
) -> Vec<String> {
    let mut steps = Vec::new();
    extend_unique_strings(&mut steps, handoff.map(|entry| entry.next_steps.iter()));
    extend_unique_strings(&mut steps, triage.map(|entry| entry.next_steps.iter()));
    extend_unique_strings(&mut steps, casebook.map(|entry| entry.next_steps.iter()));
    steps
}

fn route_actions(
    handoff: Option<&ProxyOperatorRouteHandoff>,
    triage: Option<&ProxyOperatorRouteTriage>,
    casebook: Option<&ProxyOperatorRouteCasebook>,
) -> Vec<ProxyOperatorSuggestedAction> {
    let mut actions = Vec::new();
    extend_unique_actions(&mut actions, handoff.map(|entry| entry.actions.iter()));
    extend_unique_actions(&mut actions, triage.map(|entry| entry.actions.iter()));
    extend_unique_actions(
        &mut actions,
        casebook.map(|entry| entry.recommended_actions.iter()),
    );
    actions
}

fn destination_actions(
    handoff: Option<&ProxyOperatorDestinationHandoff>,
    triage: Option<&ProxyOperatorDestinationTriage>,
    casebook: Option<&ProxyOperatorDestinationCasebook>,
) -> Vec<ProxyOperatorSuggestedAction> {
    let mut actions = Vec::new();
    extend_unique_actions(&mut actions, handoff.map(|entry| entry.actions.iter()));
    extend_unique_actions(&mut actions, triage.map(|entry| entry.actions.iter()));
    extend_unique_actions(
        &mut actions,
        casebook.map(|entry| entry.recommended_actions.iter()),
    );
    actions
}

fn extend_unique_strings<'a, I>(target: &mut Vec<String>, values: Option<I>)
where
    I: IntoIterator<Item = &'a String>,
{
    if let Some(values) = values {
        for value in values {
            if !target.contains(value) {
                target.push(value.clone());
            }
        }
    }
}

fn extend_unique_actions<'a, I>(target: &mut Vec<ProxyOperatorSuggestedAction>, values: Option<I>)
where
    I: IntoIterator<Item = &'a ProxyOperatorSuggestedAction>,
{
    if let Some(values) = values {
        for value in values {
            if !target.contains(value) {
                target.push(value.clone());
            }
        }
    }
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
