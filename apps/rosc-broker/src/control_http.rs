use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};
use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::control_plane::{ControlPlaneActionResult, ControlPlaneError, ProxyControlPlane};
use crate::{
    ProxyOperatorDiagnostics, ProxyOperatorOverrides, ProxyOperatorOverview, ProxyOperatorReport,
    ProxyOperatorSignalScope, ProxyOperatorSignalsView, UdpProxyStatusSnapshot,
    proxy_operator_signals_view,
};

#[cfg(test)]
const CONTROL_REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(100);
#[cfg(not(test))]
const CONTROL_REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Eq, PartialEq)]
struct HttpRequest {
    method: String,
    path: String,
}

#[derive(Serialize)]
struct StatusResponse {
    ok: bool,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
struct ActionResponse {
    ok: bool,
    action: &'static str,
    applied: bool,
    dispatch_count: Option<usize>,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
struct RecentOperatorActionsResponse {
    ok: bool,
    actions: Vec<RecentOperatorAction>,
}

#[derive(Serialize)]
struct RecentConfigEventsResponse {
    ok: bool,
    events: Vec<RecentConfigEvent>,
}

#[derive(Serialize)]
struct OperatorReportResponse {
    ok: bool,
    report: ProxyOperatorReport,
}

#[derive(Serialize)]
struct OperatorOverviewResponse {
    ok: bool,
    overview: ProxyOperatorOverview,
}

#[derive(Serialize)]
struct OperatorDiagnosticsResponse {
    ok: bool,
    diagnostics: Box<ProxyOperatorDiagnostics>,
}

#[derive(Serialize)]
struct OperatorOverridesResponse {
    ok: bool,
    overrides: ProxyOperatorOverrides,
}

#[derive(Serialize)]
struct OperatorSignalsResponse {
    ok: bool,
    scope: ProxyOperatorSignalScope,
    runtime_signals: crate::ProxyOperatorRuntimeSignals,
    route_signals: Vec<crate::ProxyOperatorRouteSignal>,
    destination_signals: Vec<crate::ProxyOperatorDestinationSignal>,
}

#[derive(Serialize)]
struct BlockersResponse {
    ok: bool,
    blockers: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

enum ResponseBody {
    Status(StatusResponse),
    Action(ActionResponse),
    OperatorReport(OperatorReportResponse),
    OperatorOverview(Box<OperatorOverviewResponse>),
    OperatorDiagnostics(Box<OperatorDiagnosticsResponse>),
    OperatorOverrides(OperatorOverridesResponse),
    OperatorSignals(OperatorSignalsResponse),
    Blockers(BlockersResponse),
    RecentOperatorActions(RecentOperatorActionsResponse),
    RecentConfigEvents(RecentConfigEventsResponse),
    Error(ErrorResponse),
}

struct HttpResponse {
    status: &'static str,
    body: ResponseBody,
}

pub(crate) async fn serve_control_connection(
    mut stream: TcpStream,
    control: Arc<dyn ProxyControlPlane>,
) -> io::Result<()> {
    let request =
        match tokio::time::timeout(CONTROL_REQUEST_READ_TIMEOUT, read_http_request(&mut stream))
            .await
        {
            Ok(Ok(request)) => request,
            Ok(Err(error)) => {
                write_json_response(
                    &mut stream,
                    "400 Bad Request",
                    &ResponseBody::Error(ErrorResponse {
                        ok: false,
                        error: error.to_string(),
                    }),
                )
                .await?;
                return Ok(());
            }
            Err(_) => {
                write_json_response(
                    &mut stream,
                    "408 Request Timeout",
                    &ResponseBody::Error(ErrorResponse {
                        ok: false,
                        error: format!(
                            "request headers not received within {} ms",
                            CONTROL_REQUEST_READ_TIMEOUT.as_millis()
                        ),
                    }),
                )
                .await?;
                return Ok(());
            }
        };

    let response = route_request(request, control).await;
    write_json_response(&mut stream, response.status, &response.body).await?;

    Ok(())
}

async fn route_request(request: HttpRequest, control: Arc<dyn ProxyControlPlane>) -> HttpResponse {
    let (path, query) = split_query(&request.path);
    match (request.method.as_str(), path) {
        ("GET", "/status") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Status(StatusResponse {
                ok: true,
                status: control.status_snapshot().await,
            }),
        },
        ("GET", "/report") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::OperatorReport(OperatorReportResponse {
                ok: true,
                report: control.operator_report().await,
            }),
        },
        ("GET", "/overview") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::OperatorOverview(Box::new(OperatorOverviewResponse {
                ok: true,
                overview: control.operator_overview().await,
            })),
        },
        ("GET", "/diagnostics") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::OperatorDiagnostics(Box::new(OperatorDiagnosticsResponse {
                    ok: true,
                    diagnostics: Box::new(control.operator_diagnostics(limit).await),
                })),
            }
        }
        ("GET", "/overrides") => {
            let report = control.operator_report().await;
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::OperatorOverrides(OperatorOverridesResponse {
                    ok: true,
                    overrides: report.overrides,
                }),
            }
        }
        ("GET", "/signals") => {
            let Ok(scope) = signal_scope(query) else {
                return invalid_query_error("scope");
            };
            let report = control.operator_report().await;
            let signals = proxy_operator_signals_view(&report, scope);
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::OperatorSignals(operator_signals_response(signals)),
            }
        }
        ("GET", "/blockers") => {
            let report = control.operator_report().await;
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::Blockers(BlockersResponse {
                    ok: true,
                    blockers: report.blockers,
                }),
            }
        }
        ("GET", "/history/operator-actions") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let status = control.status_snapshot().await;
            let actions = bounded_recent_history(
                status
                    .runtime
                    .map(|runtime| runtime.recent_operator_actions)
                    .unwrap_or_default(),
                limit,
            );
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::RecentOperatorActions(RecentOperatorActionsResponse {
                    ok: true,
                    actions,
                }),
            }
        }
        ("GET", "/history/config-events") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let status = control.status_snapshot().await;
            let events = bounded_recent_history(
                status
                    .runtime
                    .map(|runtime| runtime.recent_config_events)
                    .unwrap_or_default(),
                limit,
            );
            HttpResponse {
                status: "200 OK",
                body: ResponseBody::RecentConfigEvents(RecentConfigEventsResponse {
                    ok: true,
                    events,
                }),
            }
        }
        ("POST", "/freeze") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(
                "freeze_traffic",
                control.freeze_traffic().await,
            )),
        },
        ("POST", "/thaw") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(
                "thaw_traffic",
                control.thaw_traffic().await,
            )),
        },
        ("POST", "/routes/restore-all") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(
                "restore_all_routes",
                control.restore_all_routes().await,
            )),
        },
        _ => route_nested_request(&request, path, query, control).await,
    }
}

fn operator_signals_response(signals: ProxyOperatorSignalsView) -> OperatorSignalsResponse {
    OperatorSignalsResponse {
        ok: true,
        scope: signals.scope,
        runtime_signals: signals.runtime_signals,
        route_signals: signals.route_signals,
        destination_signals: signals.destination_signals,
    }
}

async fn route_nested_request(
    request: &HttpRequest,
    path: &str,
    query: Option<&str>,
    control: Arc<dyn ProxyControlPlane>,
) -> HttpResponse {
    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/rehydrate"))
    {
        if request.method != "POST" || destination_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(destination_id) = decode_uri_component(destination_id) else {
            return invalid_component_error("destination id");
        };
        return map_route_result(
            "rehydrate_destination",
            control.rehydrate_destination(&destination_id).await,
        );
    }

    let Some(route_path) = path.strip_prefix("/routes/") else {
        return HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unsupported control route {}", request.path),
            }),
        };
    };

    if let Some(route_id) = route_path.strip_suffix("/isolate") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_route_result("isolate_route", control.isolate_route(&route_id).await);
    }

    if let Some(route_id) = route_path.strip_suffix("/restore") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_route_result("restore_route", control.restore_route(&route_id).await);
    }

    if let Some((route_id, sandbox_destination_id)) = route_path.split_once("/replay/") {
        if request.method != "POST" || route_id.is_empty() || sandbox_destination_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(sandbox_destination_id) = decode_uri_component(sandbox_destination_id) else {
            return invalid_component_error("sandbox destination id");
        };
        let Ok(limit) = replay_limit(query) else {
            return invalid_query_error("limit");
        };
        return map_route_result(
            "sandbox_replay",
            control
                .replay_route_to_sandbox(&route_id, &sandbox_destination_id, limit)
                .await,
        );
    }

    HttpResponse {
        status: "404 Not Found",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("unsupported control route {}", request.path),
        }),
    }
}

fn method_or_path_error(request: &HttpRequest) -> HttpResponse {
    HttpResponse {
        status: "404 Not Found",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("unsupported control route {}", request.path),
        }),
    }
}

fn map_route_result(
    action: &'static str,
    result: Result<ControlPlaneActionResult, ControlPlaneError>,
) -> HttpResponse {
    match result {
        Ok(result) => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(action, result)),
        },
        Err(ControlPlaneError::UnknownRoute(route_id)) => HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unknown route `{route_id}`"),
            }),
        },
        Err(ControlPlaneError::UnknownDestination(destination_id)) => HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unknown destination `{destination_id}`"),
            }),
        },
        Err(ControlPlaneError::ActionFailed(message)) => HttpResponse {
            status: "422 Unprocessable Entity",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: message,
            }),
        },
    }
}

fn action_response(action: &'static str, result: ControlPlaneActionResult) -> ActionResponse {
    ActionResponse {
        ok: true,
        action,
        applied: result.applied,
        dispatch_count: result.dispatch_count,
        status: result.status,
    }
}

fn split_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (path, None),
    }
}

fn replay_limit(query: Option<&str>) -> Result<usize, ()> {
    let Some(value) = query_parameter(query, "limit") else {
        return Ok(100);
    };

    let limit = value.parse::<usize>().map_err(|_| ())?;
    if limit == 0 {
        return Err(());
    }

    Ok(limit)
}

fn history_limit(query: Option<&str>) -> Result<Option<usize>, ()> {
    let Some(value) = query_parameter(query, "limit") else {
        return Ok(None);
    };

    let limit = value.parse::<usize>().map_err(|_| ())?;
    if limit == 0 {
        return Err(());
    }

    Ok(Some(limit))
}

fn signal_scope(query: Option<&str>) -> Result<ProxyOperatorSignalScope, ()> {
    match query_parameter(query, "scope") {
        Some(value) => ProxyOperatorSignalScope::parse(value).ok_or(()),
        None => Ok(ProxyOperatorSignalScope::All),
    }
}

fn query_parameter<'a>(query: Option<&'a str>, key: &str) -> Option<&'a str> {
    query.and_then(|query| {
        query.split('&').find_map(|pair| {
            let (parameter_key, value) = pair.split_once('=')?;
            (parameter_key == key).then_some(value)
        })
    })
}

fn bounded_recent_history<T>(entries: Vec<T>, limit: Option<usize>) -> Vec<T> {
    match limit {
        Some(limit) if entries.len() > limit => {
            let start = entries.len() - limit;
            entries.into_iter().skip(start).collect()
        }
        _ => entries,
    }
}

fn decode_uri_component(component: &str) -> Result<String, ()> {
    let bytes = component.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(());
                }
                let high = decode_hex_nibble(bytes[index + 1]).ok_or(())?;
                let low = decode_hex_nibble(bytes[index + 2]).ok_or(())?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).map_err(|_| ())
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn invalid_component_error(label: &str) -> HttpResponse {
    HttpResponse {
        status: "400 Bad Request",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("invalid percent-encoding in {label}"),
        }),
    }
}

fn invalid_query_error(label: &str) -> HttpResponse {
    HttpResponse {
        status: "400 Bad Request",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("invalid query parameter `{label}`"),
        }),
    }
}

pub(crate) async fn validate_control_listen_target(listen: &str) -> Result<()> {
    let mut resolved_any = false;
    for addr in tokio::net::lookup_host(listen)
        .await
        .with_context(|| format!("failed to resolve control listener on {listen}"))?
    {
        resolved_any = true;
        ensure!(
            addr.ip().is_loopback(),
            "control listener must bind to a loopback address, got {addr}"
        );
    }

    if !resolved_any {
        return Err(anyhow!(
            "failed to resolve control listener on {listen}: no socket addresses"
        ));
    }

    Ok(())
}

async fn read_http_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 8192 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request headers exceed 8192 bytes",
            ));
        }
    }

    let request = String::from_utf8_lossy(&buffer);
    let Some(request_line) = request.lines().next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request line",
        ));
    };

    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request method",
        ));
    };
    let Some(path) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request path",
        ));
    };

    Ok(HttpRequest {
        method: method.to_owned(),
        path: path.to_owned(),
    })
}

async fn write_json_response(
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

impl ResponseBody {
    fn to_json(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Status(body) => serde_json::to_vec(body),
            Self::Action(body) => serde_json::to_vec(body),
            Self::OperatorReport(body) => serde_json::to_vec(body),
            Self::OperatorOverview(body) => serde_json::to_vec(body),
            Self::OperatorDiagnostics(body) => serde_json::to_vec(body),
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
