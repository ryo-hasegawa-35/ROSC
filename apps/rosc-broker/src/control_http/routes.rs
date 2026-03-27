use std::sync::Arc;

use crate::control_plane::ProxyControlPlane;
use crate::{ProxyOperatorSignalScope, proxy_operator_attention, proxy_operator_signals_view};

use super::request::{
    HttpRequest, allow_degraded, decode_uri_component, history_limit, query_parameter,
    replay_limit, split_query,
};
use super::response::{
    HttpResponse, blockers_response, config_events_response, dashboard_css_response,
    dashboard_data_response, dashboard_html_response, dashboard_js_response,
    dashboard_render_js_response, dashboard_state_js_response, destination_trace_response,
    diagnostics_response, incidents_response, invalid_component_error, invalid_query_error,
    map_action_result, operator_actions_response, operator_signals_response, overrides_response,
    overview_response, readiness_response, report_response, route_trace_response,
    snapshot_response, status_response, trace_response, unsupported_route_error,
};

pub(crate) async fn route_request(
    request: HttpRequest,
    control: Arc<dyn ProxyControlPlane>,
) -> HttpResponse {
    let (path, query) = split_query(&request.path);
    match (request.method.as_str(), path) {
        ("GET", "/dashboard") | ("GET", "/dashboard/") => dashboard_html_response(),
        ("GET", "/dashboard/app.css") => dashboard_css_response(),
        ("GET", "/dashboard/app.js") => dashboard_js_response(),
        ("GET", "/dashboard/dashboard-state.js") => dashboard_state_js_response(),
        ("GET", "/dashboard/dashboard-render.js") => dashboard_render_js_response(),
        ("GET", "/dashboard/data") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            dashboard_data_response(control.operator_dashboard(limit).await)
        }
        ("GET", "/status") => status_response(control.status_snapshot().await),
        ("GET", "/report") => report_response(control.operator_report().await),
        ("GET", "/overview") => overview_response(control.operator_overview().await),
        ("GET", "/readiness") => {
            let overview = control.operator_overview().await;
            readiness_response(crate::proxy_operator_readiness_from_overview(overview))
        }
        ("GET", "/readyz") => {
            let Ok(allow_degraded) = allow_degraded(query) else {
                return invalid_query_error("allow_degraded");
            };
            let overview = control.operator_overview().await;
            super::response::readyz_response(
                crate::proxy_operator_readiness_from_overview(overview),
                allow_degraded,
            )
        }
        ("GET", "/snapshot") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            snapshot_response(control.operator_snapshot(limit).await)
        }
        ("GET", "/diagnostics") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            diagnostics_response(control.operator_diagnostics(limit).await)
        }
        ("GET", "/attention") => {
            let report = control.operator_report().await;
            super::response::attention_response(proxy_operator_attention(&report))
        }
        ("GET", "/incidents") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            incidents_response(control.operator_incidents(limit).await)
        }
        ("GET", "/trace") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            trace_response(dashboard.trace)
        }
        ("GET", "/overrides") => {
            let report = control.operator_report().await;
            overrides_response(report.overrides)
        }
        ("GET", "/signals") => {
            let Ok(scope) = signal_scope(query) else {
                return invalid_query_error("scope");
            };
            let report = control.operator_report().await;
            operator_signals_response(proxy_operator_signals_view(&report, scope))
        }
        ("GET", "/blockers") => {
            let report = control.operator_report().await;
            blockers_response(report.blockers)
        }
        ("GET", "/history/operator-actions") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let diagnostics = control.operator_diagnostics(limit).await;
            operator_actions_response(diagnostics.recent_operator_actions)
        }
        ("GET", "/history/config-events") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let diagnostics = control.operator_diagnostics(limit).await;
            config_events_response(diagnostics.recent_config_events)
        }
        ("POST", "/freeze") => {
            map_action_result("freeze_traffic", Ok(control.freeze_traffic().await))
        }
        ("POST", "/thaw") => map_action_result("thaw_traffic", Ok(control.thaw_traffic().await)),
        ("POST", "/routes/restore-all") => {
            map_action_result("restore_all_routes", Ok(control.restore_all_routes().await))
        }
        _ => route_nested_request(&request, path, query, control).await,
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
        .and_then(|path| path.strip_suffix("/trace"))
    {
        if request.method != "GET" || destination_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(destination_id) = decode_uri_component(destination_id) else {
            return invalid_component_error("destination id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let dashboard = control.operator_dashboard(limit).await;
        let Some(destination_trace) = dashboard
            .trace
            .destinations
            .into_iter()
            .find(|trace| trace.destination_id == destination_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return destination_trace_response(destination_trace);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/rehydrate"))
    {
        if request.method != "POST" || destination_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(destination_id) = decode_uri_component(destination_id) else {
            return invalid_component_error("destination id");
        };
        return map_action_result(
            "rehydrate_destination",
            control.rehydrate_destination(&destination_id).await,
        );
    }

    let Some(route_path) = path.strip_prefix("/routes/") else {
        return unsupported_route_error(&request.path);
    };

    if let Some(route_id) = route_path.strip_suffix("/trace") {
        if request.method != "GET" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let dashboard = control.operator_dashboard(limit).await;
        let Some(route_trace) = dashboard
            .trace
            .routes
            .into_iter()
            .find(|trace| trace.route_id == route_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return route_trace_response(route_trace);
    }

    if let Some(route_id) = route_path.strip_suffix("/isolate") {
        if request.method != "POST" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_action_result("isolate_route", control.isolate_route(&route_id).await);
    }

    if let Some(route_id) = route_path.strip_suffix("/restore") {
        if request.method != "POST" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_action_result("restore_route", control.restore_route(&route_id).await);
    }

    if let Some((route_id, sandbox_destination_id)) = route_path.split_once("/replay/") {
        if request.method != "POST" || route_id.is_empty() || sandbox_destination_id.is_empty() {
            return unsupported_route_error(&request.path);
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
        return map_action_result(
            "sandbox_replay",
            control
                .replay_route_to_sandbox(&route_id, &sandbox_destination_id, limit)
                .await,
        );
    }

    unsupported_route_error(&request.path)
}

fn signal_scope(query: Option<&str>) -> Result<ProxyOperatorSignalScope, ()> {
    match query_parameter(query, "scope") {
        Some(value) => ProxyOperatorSignalScope::parse(value).ok_or(()),
        None => Ok(ProxyOperatorSignalScope::All),
    }
}
