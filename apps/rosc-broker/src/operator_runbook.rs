use serde::Serialize;

use crate::{
    ProxyOperatorDashboard, ProxyOperatorDossierCatalog, ProxyOperatorRouteDossier,
    ProxyOperatorState, ProxyOperatorSuggestedAction,
    operator_context::{
        dashboard_context, destination_state, route_state, scoped_destination_blockers,
        scoped_route_blockers,
    },
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorGlobalRunbook {
    pub state: String,
    pub headline: String,
    pub global_blockers: Vec<String>,
    pub global_overrides: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub board_highlights: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteRunbook {
    pub route_id: String,
    pub state: String,
    pub headline: String,
    pub scoped_blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_destination_ids: Vec<String>,
    pub recovery_surface: Vec<String>,
    pub board_highlights: Vec<String>,
    pub dossier: ProxyOperatorRouteDossier,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationRunbook {
    pub destination_id: String,
    pub state: String,
    pub headline: String,
    pub scoped_blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub recommended_actions: Vec<ProxyOperatorSuggestedAction>,
    pub linked_route_ids: Vec<String>,
    pub recovery_surface: Vec<String>,
    pub board_highlights: Vec<String>,
    pub dossier: crate::ProxyOperatorDestinationDossier,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRunbookCatalog {
    pub state: String,
    pub global: ProxyOperatorGlobalRunbook,
    pub routes: Vec<ProxyOperatorRouteRunbook>,
    pub destinations: Vec<ProxyOperatorDestinationRunbook>,
}

pub fn proxy_operator_runbook_from_dashboard(
    dashboard: &ProxyOperatorDashboard,
) -> ProxyOperatorRunbookCatalog {
    let snapshot = dashboard.snapshot.as_ref();
    let context = dashboard_context(dashboard);

    let routes = dashboard
        .dossier
        .routes
        .iter()
        .cloned()
        .map(|dossier| {
            let casebook = snapshot
                .casebook
                .route_casebooks
                .iter()
                .find(|entry| entry.route_id == dossier.route_id);
            let board_highlights = dossier
                .board_items
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>();

            ProxyOperatorRouteRunbook {
                route_id: dossier.route_id.clone(),
                state: state_label(route_state(snapshot, &dossier.route_id)).to_owned(),
                headline: casebook
                    .map(|entry| entry.summary.clone())
                    .unwrap_or_else(|| dossier.summary.clone()),
                scoped_blockers: scoped_route_blockers(snapshot, &dossier.route_id),
                next_steps: dossier.next_steps.clone(),
                recommended_actions: dossier.recommended_actions.clone(),
                linked_destination_ids: casebook
                    .map(|entry| entry.linked_destination_ids.clone())
                    .unwrap_or_else(|| dossier.focus.detail.destination_ids.clone()),
                recovery_surface: casebook
                    .map(|entry| entry.recovery_surface.clone())
                    .unwrap_or_default(),
                board_highlights,
                dossier,
            }
        })
        .collect();

    let destinations = dashboard
        .dossier
        .destinations
        .iter()
        .cloned()
        .map(|dossier| {
            let casebook = snapshot
                .casebook
                .destination_casebooks
                .iter()
                .find(|entry| entry.destination_id == dossier.destination_id);
            let board_highlights = dossier
                .board_items
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>();

            ProxyOperatorDestinationRunbook {
                destination_id: dossier.destination_id.clone(),
                state: state_label(destination_state(snapshot, &dossier.destination_id)).to_owned(),
                headline: casebook
                    .map(|entry| entry.summary.clone())
                    .unwrap_or_else(|| dossier.summary.clone()),
                scoped_blockers: scoped_destination_blockers(snapshot, &dossier.destination_id),
                next_steps: dossier.next_steps.clone(),
                recommended_actions: dossier.recommended_actions.clone(),
                linked_route_ids: casebook
                    .map(|entry| entry.linked_route_ids.clone())
                    .unwrap_or_else(|| dossier.focus.detail.route_ids.clone()),
                recovery_surface: casebook
                    .map(|entry| entry.recovery_surface.clone())
                    .unwrap_or_default(),
                board_highlights,
                dossier,
            }
        })
        .collect();

    ProxyOperatorRunbookCatalog {
        state: state_label(snapshot.overview.report.state.clone()).to_owned(),
        global: ProxyOperatorGlobalRunbook {
            state: state_label(snapshot.overview.report.state.clone()).to_owned(),
            headline: global_headline(dashboard),
            global_blockers: context.global_blockers.clone(),
            global_overrides: context.global_overrides.clone(),
            next_steps: snapshot.triage.global.next_steps.clone(),
            recommended_actions: snapshot.triage.global.actions.clone(),
            board_highlights: global_board_highlights(&dashboard.dossier),
        },
        routes,
        destinations,
    }
}

fn global_headline(dashboard: &ProxyOperatorDashboard) -> String {
    let snapshot = dashboard.snapshot.as_ref();
    if snapshot.attention.traffic_frozen {
        "Traffic is globally frozen and should be thawed before trusting focused recovery state."
            .to_owned()
    } else if !snapshot.overview.report.blockers.is_empty() {
        format!(
            "{} config blocker(s) are still active and should be cleared before rollout.",
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
            "{} isolated route(s) still need explicit restore review.",
            snapshot.overview.report.overrides.isolated_route_ids.len()
        )
    } else {
        snapshot.triage.global.summary.clone()
    }
}

fn global_board_highlights(dossier: &ProxyOperatorDossierCatalog) -> Vec<String> {
    let mut highlights = Vec::new();
    for route in &dossier.routes {
        for item in &route.board_items {
            if item.scope == crate::ProxyOperatorBoardScope::Global
                && !highlights.contains(&item.title)
            {
                highlights.push(item.title.clone());
            }
        }
    }
    for destination in &dossier.destinations {
        for item in &destination.board_items {
            if item.scope == crate::ProxyOperatorBoardScope::Global
                && !highlights.contains(&item.title)
            {
                highlights.push(item.title.clone());
            }
        }
    }
    highlights
}

fn state_label(state: ProxyOperatorState) -> &'static str {
    match state {
        ProxyOperatorState::Healthy => "healthy",
        ProxyOperatorState::Warning => "warning",
        ProxyOperatorState::Blocked => "blocked",
    }
}
