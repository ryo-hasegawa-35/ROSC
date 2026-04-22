use serde::Serialize;

use crate::{
    ProxyOperatorHandoffCatalog, ProxyOperatorSnapshot, ProxyOperatorSuggestedAction,
    ProxyOperatorSuggestedActionKind, ProxyOperatorTimelineCatalog, ProxyOperatorTimelineEntry,
    ProxyOperatorTraceEvent, ProxyOperatorTraceEventLevel, proxy_operator_handoff,
    proxy_operator_timeline,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorGlobalTriage {
    pub state: String,
    pub summary: String,
    pub next_steps: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteTriage {
    pub route_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub next_steps: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
    pub timeline: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationTriage {
    pub destination_id: String,
    pub level: ProxyOperatorTraceEventLevel,
    pub summary: String,
    pub next_steps: Vec<String>,
    pub actions: Vec<ProxyOperatorSuggestedAction>,
    pub recent_events: Vec<ProxyOperatorTraceEvent>,
    pub timeline: Vec<ProxyOperatorTimelineEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorTriageCatalog {
    pub state: String,
    pub global: ProxyOperatorGlobalTriage,
    pub route_triage: Vec<ProxyOperatorRouteTriage>,
    pub destination_triage: Vec<ProxyOperatorDestinationTriage>,
}

impl Default for ProxyOperatorGlobalTriage {
    fn default() -> Self {
        Self {
            state: "ready".to_owned(),
            summary: "No immediate global operator intervention is required.".to_owned(),
            next_steps: vec!["Continue monitoring runtime health and recent history.".to_owned()],
            actions: Vec::new(),
        }
    }
}

pub fn proxy_operator_triage(snapshot: &ProxyOperatorSnapshot) -> ProxyOperatorTriageCatalog {
    let handoff = if snapshot.handoff.route_handoffs.is_empty()
        && snapshot.handoff.destination_handoffs.is_empty()
    {
        proxy_operator_handoff(snapshot)
    } else {
        snapshot.handoff.clone()
    };
    let timeline = proxy_operator_timeline(snapshot);

    let route_triage = handoff
        .route_handoffs
        .iter()
        .map(|handoff| ProxyOperatorRouteTriage {
            route_id: handoff.route_id.clone(),
            level: handoff.level,
            summary: handoff.summary.clone(),
            next_steps: handoff.next_steps.clone(),
            actions: handoff.actions.clone(),
            recent_events: handoff.recent_events.clone(),
            timeline: timeline
                .routes
                .iter()
                .find(|entry| entry.route_id == handoff.route_id)
                .map(|entry| entry.entries.clone())
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    let destination_triage = handoff
        .destination_handoffs
        .iter()
        .map(|handoff| ProxyOperatorDestinationTriage {
            destination_id: handoff.destination_id.clone(),
            level: handoff.level,
            summary: handoff.summary.clone(),
            next_steps: handoff.next_steps.clone(),
            actions: handoff.actions.clone(),
            recent_events: handoff.recent_events.clone(),
            timeline: timeline
                .destinations
                .iter()
                .find(|entry| entry.destination_id == handoff.destination_id)
                .map(|entry| entry.entries.clone())
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    ProxyOperatorTriageCatalog {
        state: handoff.state.clone(),
        global: global_triage(snapshot, &handoff, &timeline),
        route_triage,
        destination_triage,
    }
}

fn global_triage(
    snapshot: &ProxyOperatorSnapshot,
    handoff: &ProxyOperatorHandoffCatalog,
    timeline: &ProxyOperatorTimelineCatalog,
) -> ProxyOperatorGlobalTriage {
    let mut next_steps = Vec::new();
    let mut actions = Vec::new();

    if snapshot.overview.report.overrides.traffic_frozen {
        next_steps.push(
            "Thaw traffic first so live dispatch and destination pressure can move again."
                .to_owned(),
        );
        actions.push(ProxyOperatorSuggestedAction {
            kind: ProxyOperatorSuggestedActionKind::ThawTraffic,
            label: "Thaw traffic".to_owned(),
            route_id: None,
            destination_id: None,
        });
    }
    if !snapshot.overview.report.blockers.is_empty() {
        next_steps.push(
            "Resolve current config blockers before widening traffic or trusting recovery automation."
                .to_owned(),
        );
    }
    if !snapshot
        .overview
        .report
        .overrides
        .isolated_route_ids
        .is_empty()
    {
        next_steps.push(
            "Review isolated routes and restore only after downstream destinations are healthy."
                .to_owned(),
        );
    }
    if handoff
        .destination_handoffs
        .iter()
        .any(|entry| entry.level == ProxyOperatorTraceEventLevel::Blocked)
    {
        next_steps.push(
            "Clear blocked destinations before replay or rehydrate so recovery does not amplify failure."
                .to_owned(),
        );
    }
    if next_steps.is_empty() {
        next_steps.push("No immediate global intervention is required.".to_owned());
    }

    let summary = if snapshot.overview.report.overrides.traffic_frozen {
        "Traffic is globally frozen; operator action should resume live flow before deeper triage."
            .to_owned()
    } else if !snapshot.overview.report.blockers.is_empty() {
        format!(
            "{} config blocker(s) are still active.",
            snapshot.overview.report.blockers.len()
        )
    } else if !snapshot
        .overview
        .report
        .overrides
        .isolated_route_ids
        .is_empty()
    {
        format!(
            "{} route(s) remain isolated and should be reviewed before restore.",
            snapshot.overview.report.overrides.isolated_route_ids.len()
        )
    } else if timeline.global.is_empty() {
        "Global timeline is quiet and no high-priority operator action is pending.".to_owned()
    } else {
        "No global override is blocking traffic, but recent history still deserves review."
            .to_owned()
    };

    ProxyOperatorGlobalTriage {
        state: handoff.state.clone(),
        summary,
        next_steps,
        actions,
    }
}
