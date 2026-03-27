use std::io;

use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::{
    ProxyOperatorAttention, ProxyOperatorBoard, ProxyOperatorCasebookCatalog,
    ProxyOperatorDashboard, ProxyOperatorDestinationTrace, ProxyOperatorDiagnostics,
    ProxyOperatorHandoffCatalog, ProxyOperatorIncidents, ProxyOperatorOverrides,
    ProxyOperatorOverview, ProxyOperatorReadiness, ProxyOperatorReport, ProxyOperatorRouteTrace,
    ProxyOperatorSignalScope, ProxyOperatorSnapshot, ProxyOperatorTimelineCatalog,
    ProxyOperatorTraceCatalog, ProxyOperatorTriageCatalog, UdpProxyStatusSnapshot,
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
pub(crate) struct OperatorDashboardResponse {
    pub(crate) ok: bool,
    pub(crate) dashboard: Box<ProxyOperatorDashboard>,
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
pub(crate) struct OperatorTraceResponse {
    pub(crate) ok: bool,
    pub(crate) trace: ProxyOperatorTraceCatalog,
}

#[derive(Serialize)]
pub(crate) struct OperatorRouteTraceResponse {
    pub(crate) ok: bool,
    pub(crate) route_trace: ProxyOperatorRouteTrace,
}

#[derive(Serialize)]
pub(crate) struct OperatorDestinationTraceResponse {
    pub(crate) ok: bool,
    pub(crate) destination_trace: ProxyOperatorDestinationTrace,
}

#[derive(Serialize)]
pub(crate) struct OperatorHandoffResponse {
    pub(crate) ok: bool,
    pub(crate) handoff: ProxyOperatorHandoffCatalog,
}

#[derive(Serialize)]
pub(crate) struct OperatorTimelineResponse {
    pub(crate) ok: bool,
    pub(crate) timeline: ProxyOperatorTimelineCatalog,
}

#[derive(Serialize)]
pub(crate) struct OperatorTriageResponse {
    pub(crate) ok: bool,
    pub(crate) triage: ProxyOperatorTriageCatalog,
}

#[derive(Serialize)]
pub(crate) struct OperatorCasebookResponse {
    pub(crate) ok: bool,
    pub(crate) casebook: ProxyOperatorCasebookCatalog,
}

#[derive(Serialize)]
pub(crate) struct OperatorBoardResponse {
    pub(crate) ok: bool,
    pub(crate) board: ProxyOperatorBoard,
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
    OperatorDashboard(Box<OperatorDashboardResponse>),
    OperatorDiagnostics(Box<OperatorDiagnosticsResponse>),
    OperatorAttention(OperatorAttentionResponse),
    OperatorIncidents(OperatorIncidentsResponse),
    OperatorHandoff(OperatorHandoffResponse),
    OperatorTimeline(OperatorTimelineResponse),
    OperatorTriage(OperatorTriageResponse),
    OperatorCasebook(OperatorCasebookResponse),
    OperatorBoard(OperatorBoardResponse),
    OperatorTrace(OperatorTraceResponse),
    OperatorRouteTrace(OperatorRouteTraceResponse),
    OperatorDestinationTrace(OperatorDestinationTraceResponse),
    OperatorOverrides(OperatorOverridesResponse),
    OperatorSignals(OperatorSignalsResponse),
    Blockers(BlockersResponse),
    RecentOperatorActions(RecentOperatorActionsResponse),
    RecentConfigEvents(RecentConfigEventsResponse),
    StaticAsset {
        content_type: &'static str,
        body: &'static str,
    },
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

    pub(crate) fn to_http_payload(&self) -> io::Result<(&'static str, Vec<u8>)> {
        match self {
            Self::Status(body) => json_payload(body),
            Self::Action(body) => json_payload(body),
            Self::OperatorReport(body) => json_payload(body),
            Self::OperatorOverview(body) => json_payload(body),
            Self::OperatorReadiness(body) => json_payload(body),
            Self::OperatorSnapshot(body) => json_payload(body),
            Self::OperatorDashboard(body) => json_payload(body),
            Self::OperatorDiagnostics(body) => json_payload(body),
            Self::OperatorAttention(body) => json_payload(body),
            Self::OperatorIncidents(body) => json_payload(body),
            Self::OperatorHandoff(body) => json_payload(body),
            Self::OperatorTimeline(body) => json_payload(body),
            Self::OperatorTriage(body) => json_payload(body),
            Self::OperatorCasebook(body) => json_payload(body),
            Self::OperatorBoard(body) => json_payload(body),
            Self::OperatorTrace(body) => json_payload(body),
            Self::OperatorRouteTrace(body) => json_payload(body),
            Self::OperatorDestinationTrace(body) => json_payload(body),
            Self::OperatorOverrides(body) => json_payload(body),
            Self::OperatorSignals(body) => json_payload(body),
            Self::Blockers(body) => json_payload(body),
            Self::RecentOperatorActions(body) => json_payload(body),
            Self::RecentConfigEvents(body) => json_payload(body),
            Self::StaticAsset { content_type, body } => {
                Ok((content_type, body.as_bytes().to_vec()))
            }
            Self::Error(body) => json_payload(body),
        }
    }
}

fn json_payload<T: Serialize>(value: &T) -> io::Result<(&'static str, Vec<u8>)> {
    serde_json::to_vec(value)
        .map(|payload| ("application/json", payload))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
