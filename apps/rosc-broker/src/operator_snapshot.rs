use serde::Serialize;

use crate::{
    ProxyOperatorAttention, ProxyOperatorDiagnostics, ProxyOperatorHandoffCatalog,
    ProxyOperatorIncidentDigest, ProxyOperatorIncidents, ProxyOperatorOverview,
    ProxyOperatorReadiness, ProxyOperatorRecovery, ProxyOperatorTriageCatalog,
    ProxyOperatorWorklist, ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot,
    proxy_operator_attention, proxy_operator_diagnostics_from_overview, proxy_operator_handoff,
    proxy_operator_incident_digest, proxy_operator_incidents_from_histories,
    proxy_operator_overview, proxy_operator_readiness_from_overview, proxy_operator_recovery,
    proxy_operator_triage, proxy_operator_worklist,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorSnapshot {
    pub overview: ProxyOperatorOverview,
    pub readiness: ProxyOperatorReadiness,
    pub diagnostics: ProxyOperatorDiagnostics,
    pub attention: ProxyOperatorAttention,
    pub incidents: ProxyOperatorIncidents,
    pub incident_digest: ProxyOperatorIncidentDigest,
    pub recovery: ProxyOperatorRecovery,
    pub worklist: ProxyOperatorWorklist,
    pub handoff: ProxyOperatorHandoffCatalog,
    pub triage: ProxyOperatorTriageCatalog,
}

pub fn proxy_operator_snapshot(
    status: &UdpProxyStatusSnapshot,
    policy: ProxyRuntimeSafetyPolicy,
    history_limit: Option<usize>,
) -> ProxyOperatorSnapshot {
    proxy_operator_snapshot_from_overview(proxy_operator_overview(status, policy), history_limit)
}

pub fn proxy_operator_snapshot_from_overview(
    overview: ProxyOperatorOverview,
    history_limit: Option<usize>,
) -> ProxyOperatorSnapshot {
    let report = overview.report.clone();
    let recent_operator_actions = overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| runtime.recent_operator_actions.clone())
        .unwrap_or_default();
    let recent_config_events = overview
        .status
        .runtime
        .as_ref()
        .map(|runtime| runtime.recent_config_events.clone())
        .unwrap_or_default();

    let readiness = proxy_operator_readiness_from_overview(overview.clone());
    let diagnostics = proxy_operator_diagnostics_from_overview(
        overview.clone(),
        recent_operator_actions.clone(),
        recent_config_events.clone(),
        history_limit,
    );
    let attention = proxy_operator_attention(&report);
    let incidents = proxy_operator_incidents_from_histories(
        &report,
        recent_operator_actions,
        recent_config_events,
        history_limit,
    );
    let provisional_snapshot = ProxyOperatorSnapshot {
        overview,
        readiness,
        diagnostics,
        attention,
        incidents,
        incident_digest: ProxyOperatorIncidentDigest::default(),
        recovery: ProxyOperatorRecovery::default(),
        worklist: ProxyOperatorWorklist::default(),
        handoff: ProxyOperatorHandoffCatalog::default(),
        triage: ProxyOperatorTriageCatalog::default(),
    };
    let incident_digest = proxy_operator_incident_digest(&provisional_snapshot);
    let recovery = proxy_operator_recovery(&provisional_snapshot);
    let worklist = proxy_operator_worklist(&provisional_snapshot);
    let post_recovery_snapshot = ProxyOperatorSnapshot {
        incident_digest,
        recovery,
        worklist,
        ..provisional_snapshot.clone()
    };
    let handoff = proxy_operator_handoff(&post_recovery_snapshot);
    let triage = proxy_operator_triage(&ProxyOperatorSnapshot {
        handoff: handoff.clone(),
        ..post_recovery_snapshot.clone()
    });

    ProxyOperatorSnapshot {
        handoff,
        triage,
        ..post_recovery_snapshot
    }
}
