use std::io;
use std::sync::Arc;

use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::control_plane::{ControlPlaneActionResult, ControlPlaneError, ProxyControlPlane};
use crate::{
    ProxyOperatorAttention, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverrides, ProxyOperatorOverview, ProxyOperatorReadiness, ProxyOperatorReport,
    ProxyOperatorSignalScope, ProxyOperatorSignalsView, UdpProxyStatusSnapshot,
};

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    ok: bool,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
pub(crate) struct ActionResponse {
    ok: bool,
    action: &'static str,
    applied: bool,
    dispatch_count: Option<usize>,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
pub(crate) struct RecentOperatorActionsResponse {
    ok: bool,
    actions: Vec<RecentOperatorAction>,
}

#[derive(Serialize)]
pub(crate) struct RecentConfigEventsResponse {
    ok: bool,
    events: Vec<RecentConfigEvent>,
}

#[derive(Serialize)]
pub(crate) struct OperatorReportResponse {
    ok: bool,
    report: ProxyOperatorReport,
}

#[derive(Serialize)]
pub(crate) struct OperatorOverviewResponse {
    ok: bool,
    overview: ProxyOperatorOverview,
}

#[derive(Serialize)]
pub(crate) struct OperatorReadinessResponse {
    ok: bool,
    readiness: ProxyOperatorReadiness,
}

#[derive(Serialize)]
pub(crate) struct OperatorDiagnosticsResponse {
    ok: bool,
    diagnostics: Box<ProxyOperatorDiagnostics>,
}

#[derive(Serialize)]
pub(crate) struct OperatorAttentionResponse {
    ok: bool,
    attention: ProxyOperatorAttention,
}

#[derive(Serialize)]
pub(crate) struct OperatorIncidentsResponse {
    ok: bool,
    incidents: ProxyOperatorIncidents,
}

#[derive(Serialize)]
pub(crate) struct OperatorOverridesResponse {
    ok: bool,
    overrides: ProxyOperatorOverrides,
}

#[derive(Serialize)]
pub(crate) struct OperatorSignalsResponse {
    ok: bool,
    scope: ProxyOperatorSignalScope,
    runtime_signals: crate::ProxyOperatorRuntimeSignals,
    route_signals: Vec<crate::ProxyOperatorRouteSignal>,
    destination_signals: Vec<crate::ProxyOperatorDestinationSignal>,
}

#[derive(Serialize)]
pub(crate) struct BlockersResponse {
    ok: bool,
    blockers: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    ok: bool,
    error: String,
}

pub(crate) enum ResponseBody {
    Status(StatusResponse),
    Action(ActionResponse),
    OperatorReport(OperatorReportResponse),
    OperatorOverview(Box<OperatorOverviewResponse>),
    OperatorReadiness(Box<OperatorReadinessResponse>),
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
    pub status: &'static str,
    pub body: ResponseBody,
}

impl ResponseBody {
    pub(crate) fn error(error: String) -> Self {
        Self::Error(ErrorResponse { ok: false, error })
    }

    fn to_json(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Status(body) => serde_json::to_vec(body),
            Self::Action(body) => serde_json::to_vec(body),
            Self::OperatorReport(body) => serde_json::to_vec(body),
            Self::OperatorOverview(body) => serde_json::to_vec(body),
            Self::OperatorReadiness(body) => serde_json::to_vec(body),
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

pub(crate) async fn serve_control_connection(
    stream: TcpStream,
    control: Arc<dyn ProxyControlPlane>,
) -> io::Result<()> {
    super::serve_control_connection_impl(stream, control).await
}

pub(crate) async fn validate_control_listen_target(listen: &str) -> anyhow::Result<()> {
    super::validate_control_listen_target_impl(listen).await
}

pub(crate) async fn write_json_response(
    stream: &mut TcpStream,
    status: &str,
    body: &ResponseBody,
) -> io::Result<()> {
    let payload = body.to_json()?;
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(headers.as_bytes()).await?;
    stream.write_all(&payload).await
}

pub(crate) fn status_response(status: UdpProxyStatusSnapshot) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::Status(StatusResponse { ok: true, status }),
    }
}

pub(crate) fn report_response(report: ProxyOperatorReport) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorReport(OperatorReportResponse { ok: true, report }),
    }
}

pub(crate) fn overview_response(overview: ProxyOperatorOverview) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorOverview(Box::new(OperatorOverviewResponse {
            ok: true,
            overview,
        })),
    }
}

pub(crate) fn readiness_response(readiness: ProxyOperatorReadiness) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorReadiness(Box::new(OperatorReadinessResponse {
            ok: true,
            readiness,
        })),
    }
}

pub(crate) fn diagnostics_response(diagnostics: ProxyOperatorDiagnostics) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorDiagnostics(Box::new(OperatorDiagnosticsResponse {
            ok: true,
            diagnostics: Box::new(diagnostics),
        })),
    }
}

pub(crate) fn attention_response(attention: ProxyOperatorAttention) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorAttention(OperatorAttentionResponse {
            ok: true,
            attention,
        }),
    }
}

pub(crate) fn incidents_response(incidents: ProxyOperatorIncidents) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorIncidents(OperatorIncidentsResponse {
            ok: true,
            incidents,
        }),
    }
}

pub(crate) fn overrides_response(overrides: ProxyOperatorOverrides) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorOverrides(OperatorOverridesResponse {
            ok: true,
            overrides,
        }),
    }
}

pub(crate) fn blockers_response(blockers: Vec<String>) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::Blockers(BlockersResponse { ok: true, blockers }),
    }
}

pub(crate) fn operator_actions_response(actions: Vec<RecentOperatorAction>) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::RecentOperatorActions(RecentOperatorActionsResponse {
            ok: true,
            actions,
        }),
    }
}

pub(crate) fn config_events_response(events: Vec<RecentConfigEvent>) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::RecentConfigEvents(RecentConfigEventsResponse { ok: true, events }),
    }
}

pub(crate) fn action_response(
    action: &'static str,
    result: ControlPlaneActionResult,
) -> ResponseBody {
    ResponseBody::Action(ActionResponse {
        ok: true,
        action,
        applied: result.applied,
        dispatch_count: result.dispatch_count,
        status: result.status,
    })
}

pub(crate) fn map_action_result(
    action: &'static str,
    result: Result<ControlPlaneActionResult, ControlPlaneError>,
) -> HttpResponse {
    match result {
        Ok(result) => HttpResponse {
            status: "200 OK",
            body: action_response(action, result),
        },
        Err(ControlPlaneError::UnknownRoute(route_id)) => {
            not_found_error(format!("unknown route `{route_id}`"))
        }
        Err(ControlPlaneError::UnknownDestination(destination_id)) => {
            not_found_error(format!("unknown destination `{destination_id}`"))
        }
        Err(ControlPlaneError::ActionFailed(message)) => HttpResponse {
            status: "422 Unprocessable Entity",
            body: ResponseBody::error(message),
        },
    }
}

pub(crate) fn operator_signals_response(signals: ProxyOperatorSignalsView) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorSignals(OperatorSignalsResponse {
            ok: true,
            scope: signals.scope,
            runtime_signals: signals.runtime_signals,
            route_signals: signals.route_signals,
            destination_signals: signals.destination_signals,
        }),
    }
}

pub(crate) fn invalid_component_error(label: &str) -> HttpResponse {
    HttpResponse {
        status: "400 Bad Request",
        body: ResponseBody::error(format!("invalid percent-encoding in {label}")),
    }
}

pub(crate) fn invalid_query_error(label: &str) -> HttpResponse {
    HttpResponse {
        status: "400 Bad Request",
        body: ResponseBody::error(format!("invalid query parameter `{label}`")),
    }
}

pub(crate) fn unsupported_route_error(path: &str) -> HttpResponse {
    not_found_error(format!("unsupported control route {path}"))
}

pub(crate) fn not_found_error(error: String) -> HttpResponse {
    HttpResponse {
        status: "404 Not Found",
        body: ResponseBody::error(error),
    }
}
