use crate::control_http::dashboard::{
    DASHBOARD_CSS, DASHBOARD_HTML, DASHBOARD_JS, DASHBOARD_RENDER_JS, DASHBOARD_STATE_JS,
};
use crate::control_plane::{ControlPlaneActionResult, ControlPlaneError};
use crate::{
    ProxyOperatorAttention, ProxyOperatorBoard, ProxyOperatorBriefCatalog,
    ProxyOperatorCasebookCatalog, ProxyOperatorDashboard, ProxyOperatorDestinationTrace,
    ProxyOperatorDiagnostics, ProxyOperatorDossierCatalog, ProxyOperatorFocusCatalog,
    ProxyOperatorHandoffCatalog, ProxyOperatorIncidents, ProxyOperatorLensCatalog,
    ProxyOperatorOverrides, ProxyOperatorOverview, ProxyOperatorReadiness, ProxyOperatorReport,
    ProxyOperatorRouteTrace, ProxyOperatorRunbookCatalog, ProxyOperatorSignalsView,
    ProxyOperatorSnapshot, ProxyOperatorTimelineCatalog, ProxyOperatorTraceCatalog,
    ProxyOperatorTriageCatalog, UdpProxyStatusSnapshot,
};
use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};

use super::payloads::{
    ActionResponse, BlockersResponse, HttpResponse, OperatorAttentionResponse,
    OperatorBoardResponse, OperatorBriefResponse, OperatorCasebookResponse,
    OperatorDashboardResponse, OperatorDestinationTraceResponse, OperatorDiagnosticsResponse,
    OperatorDossierResponse, OperatorFocusResponse, OperatorHandoffResponse,
    OperatorIncidentsResponse, OperatorLensResponse, OperatorOverridesResponse,
    OperatorOverviewResponse, OperatorReadinessResponse, OperatorReportResponse,
    OperatorRouteTraceResponse, OperatorRunbookResponse, OperatorSignalsResponse,
    OperatorSnapshotResponse, OperatorTimelineResponse, OperatorTraceResponse,
    OperatorTriageResponse, RecentConfigEventsResponse, RecentOperatorActionsResponse,
    ResponseBody, StatusResponse,
};

pub(crate) fn status_response(status: UdpProxyStatusSnapshot) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::Status(StatusResponse { ok: true, status }),
    }
}

pub(crate) fn dashboard_html_response() -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::StaticAsset {
            content_type: "text/html; charset=utf-8",
            body: DASHBOARD_HTML,
        },
    }
}

pub(crate) fn dashboard_css_response() -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::StaticAsset {
            content_type: "text/css; charset=utf-8",
            body: DASHBOARD_CSS,
        },
    }
}

pub(crate) fn dashboard_js_response() -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::StaticAsset {
            content_type: "application/javascript; charset=utf-8",
            body: DASHBOARD_JS,
        },
    }
}

pub(crate) fn dashboard_state_js_response() -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::StaticAsset {
            content_type: "application/javascript; charset=utf-8",
            body: DASHBOARD_STATE_JS,
        },
    }
}

pub(crate) fn dashboard_render_js_response() -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::StaticAsset {
            content_type: "application/javascript; charset=utf-8",
            body: DASHBOARD_RENDER_JS,
        },
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

pub(crate) fn readyz_response(
    readiness: ProxyOperatorReadiness,
    allow_degraded: bool,
) -> HttpResponse {
    HttpResponse {
        status: if readiness.is_acceptable(allow_degraded) {
            "200 OK"
        } else {
            "503 Service Unavailable"
        },
        body: ResponseBody::OperatorReadiness(Box::new(OperatorReadinessResponse {
            ok: true,
            readiness,
        })),
    }
}

pub(crate) fn snapshot_response(snapshot: ProxyOperatorSnapshot) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorSnapshot(Box::new(OperatorSnapshotResponse {
            ok: true,
            snapshot: Box::new(snapshot),
        })),
    }
}

pub(crate) fn dashboard_data_response(dashboard: ProxyOperatorDashboard) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorDashboard(Box::new(OperatorDashboardResponse {
            ok: true,
            dashboard: Box::new(dashboard),
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

pub(crate) fn handoff_response(handoff: ProxyOperatorHandoffCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorHandoff(OperatorHandoffResponse { ok: true, handoff }),
    }
}

pub(crate) fn timeline_response(timeline: ProxyOperatorTimelineCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorTimeline(OperatorTimelineResponse { ok: true, timeline }),
    }
}

pub(crate) fn triage_response(triage: ProxyOperatorTriageCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorTriage(OperatorTriageResponse { ok: true, triage }),
    }
}

pub(crate) fn casebook_response(casebook: ProxyOperatorCasebookCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorCasebook(OperatorCasebookResponse { ok: true, casebook }),
    }
}

pub(crate) fn board_response(board: ProxyOperatorBoard) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorBoard(OperatorBoardResponse { ok: true, board }),
    }
}

pub(crate) fn focus_response(focus: ProxyOperatorFocusCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorFocus(OperatorFocusResponse { ok: true, focus }),
    }
}

pub(crate) fn lens_response(lens: ProxyOperatorLensCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorLens(OperatorLensResponse { ok: true, lens }),
    }
}

pub(crate) fn brief_response(brief: ProxyOperatorBriefCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorBrief(OperatorBriefResponse { ok: true, brief }),
    }
}

pub(crate) fn dossier_response(dossier: ProxyOperatorDossierCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorDossier(OperatorDossierResponse { ok: true, dossier }),
    }
}

pub(crate) fn runbook_response(runbook: ProxyOperatorRunbookCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorRunbook(OperatorRunbookResponse { ok: true, runbook }),
    }
}

pub(crate) fn trace_response(trace: ProxyOperatorTraceCatalog) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorTrace(OperatorTraceResponse { ok: true, trace }),
    }
}

pub(crate) fn route_trace_response(route_trace: ProxyOperatorRouteTrace) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorRouteTrace(OperatorRouteTraceResponse {
            ok: true,
            route_trace,
        }),
    }
}

pub(crate) fn destination_trace_response(
    destination_trace: ProxyOperatorDestinationTrace,
) -> HttpResponse {
    HttpResponse {
        status: "200 OK",
        body: ResponseBody::OperatorDestinationTrace(OperatorDestinationTraceResponse {
            ok: true,
            destination_trace,
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
