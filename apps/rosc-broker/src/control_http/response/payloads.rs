use std::io;

use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::{
    ProxyOperatorAttention, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverrides, ProxyOperatorOverview, ProxyOperatorReadiness, ProxyOperatorReport,
    ProxyOperatorSignalScope, ProxyOperatorSnapshot, UdpProxyStatusSnapshot,
};

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    pub(crate) ok: bool,
    pub(crate) status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
pub(crate) struct ActionResponse {
    pub(crate) ok: bool,
    pub(crate) action: &'static str,
    pub(crate) applied: bool,
    pub(crate) dispatch_count: Option<usize>,
    pub(crate) status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
pub(crate) struct RecentOperatorActionsResponse {
    pub(crate) ok: bool,
    pub(crate) actions: Vec<RecentOperatorAction>,
}

#[derive(Serialize)]
pub(crate) struct RecentConfigEventsResponse {
    pub(crate) ok: bool,
    pub(crate) events: Vec<RecentConfigEvent>,
}

#[derive(Serialize)]
pub(crate) struct OperatorReportResponse {
    pub(crate) ok: bool,
    pub(crate) report: ProxyOperatorReport,
}

#[derive(Serialize)]
pub(crate) struct OperatorOverviewResponse {
    pub(crate) ok: bool,
    pub(crate) overview: ProxyOperatorOverview,
}

#[derive(Serialize)]
pub(crate) struct OperatorReadinessResponse {
    pub(crate) ok: bool,
    pub(crate) readiness: ProxyOperatorReadiness,
}

#[derive(Serialize)]
pub(crate) struct OperatorSnapshotResponse {
    pub(crate) ok: bool,
    pub(crate) snapshot: Box<ProxyOperatorSnapshot>,
}

#[derive(Serialize)]
pub(crate) struct OperatorDiagnosticsResponse {
    pub(crate) ok: bool,
    pub(crate) diagnostics: Box<ProxyOperatorDiagnostics>,
}

#[derive(Serialize)]
pub(crate) struct OperatorAttentionResponse {
    pub(crate) ok: bool,
    pub(crate) attention: ProxyOperatorAttention,
}

#[derive(Serialize)]
pub(crate) struct OperatorIncidentsResponse {
    pub(crate) ok: bool,
    pub(crate) incidents: ProxyOperatorIncidents,
}

#[derive(Serialize)]
pub(crate) struct OperatorOverridesResponse {
    pub(crate) ok: bool,
    pub(crate) overrides: ProxyOperatorOverrides,
}

#[derive(Serialize)]
pub(crate) struct OperatorSignalsResponse {
    pub(crate) ok: bool,
    pub(crate) scope: ProxyOperatorSignalScope,
    pub(crate) runtime_signals: crate::ProxyOperatorRuntimeSignals,
    pub(crate) route_signals: Vec<crate::ProxyOperatorRouteSignal>,
    pub(crate) destination_signals: Vec<crate::ProxyOperatorDestinationSignal>,
}

#[derive(Serialize)]
pub(crate) struct BlockersResponse {
    pub(crate) ok: bool,
    pub(crate) blockers: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) ok: bool,
    pub(crate) error: String,
}

pub(crate) enum ResponseBody {
    Status(StatusResponse),
    Action(ActionResponse),
    OperatorReport(OperatorReportResponse),
    OperatorOverview(Box<OperatorOverviewResponse>),
    OperatorReadiness(Box<OperatorReadinessResponse>),
    OperatorSnapshot(Box<OperatorSnapshotResponse>),
    OperatorDiagnostics(Box<OperatorDiagnosticsResponse>),
    OperatorAttention(OperatorAttentionResponse),
    OperatorIncidents(OperatorIncidentsResponse),
    OperatorOverrides(OperatorOverridesResponse),
    OperatorSignals(OperatorSignalsResponse),
    Blockers(BlockersResponse),
    RecentOperatorActions(RecentOperatorActionsResponse),
    RecentConfigEvents(RecentConfigEventsResponse),
    Error(ErrorResponse),
}

pub(crate) struct HttpResponse {
    pub(crate) status: &'static str,
    pub(crate) body: ResponseBody,
}

impl ResponseBody {
    pub(crate) fn error(error: String) -> Self {
        Self::Error(ErrorResponse { ok: false, error })
    }

    pub(crate) fn to_json(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Status(body) => serde_json::to_vec(body),
            Self::Action(body) => serde_json::to_vec(body),
            Self::OperatorReport(body) => serde_json::to_vec(body),
            Self::OperatorOverview(body) => serde_json::to_vec(body),
            Self::OperatorReadiness(body) => serde_json::to_vec(body),
            Self::OperatorSnapshot(body) => serde_json::to_vec(body),
            Self::OperatorDiagnostics(body) => serde_json::to_vec(body),
            Self::OperatorAttention(body) => serde_json::to_vec(body),
            Self::OperatorIncidents(body) => serde_json::to_vec(body),
            Self::OperatorOverrides(body) => serde_json::to_vec(body),
            Self::OperatorSignals(body) => serde_json::to_vec(body),
            Self::Blockers(body) => serde_json::to_vec(body),
            Self::RecentOperatorActions(body) => serde_json::to_vec(body),
            Self::RecentConfigEvents(body) => serde_json::to_vec(body),
            Self::Error(body) => serde_json::to_vec(body),
        }
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}
