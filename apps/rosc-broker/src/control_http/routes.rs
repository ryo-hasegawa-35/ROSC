use std::sync::Arc;

use crate::control_plane::ProxyControlPlane;
use crate::{
    ProxyOperatorBoardScope, ProxyOperatorSignalScope, ProxyOperatorTimelineCatalog,
    proxy_operator_attention, proxy_operator_signals_view,
};

use super::request::{
    HttpRequest, allow_degraded, decode_uri_component, history_limit, query_parameter,
    replay_limit, split_query,
};
use super::response::{
    HttpResponse, blockers_response, board_response, brief_response, casebook_response,
    config_events_response, dashboard_css_response, dashboard_data_response,
    dashboard_html_response, dashboard_js_response, dashboard_render_js_response,
    dashboard_state_js_response, destination_trace_response, diagnostics_response,
    dossier_response, focus_response, handoff_response, incidents_response,
    invalid_component_error, invalid_query_error, lens_response, map_action_result,
    operator_actions_response, operator_signals_response, overrides_response, overview_response,
    readiness_response, report_response, route_trace_response, snapshot_response, status_response,
    timeline_response, trace_response, triage_response, unsupported_route_error,
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
        ("GET", "/handoff") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let snapshot = control.operator_snapshot(limit).await;
            handoff_response(snapshot.handoff)
        }
        ("GET", "/triage") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let snapshot = control.operator_snapshot(limit).await;
            triage_response(snapshot.triage)
        }
        ("GET", "/casebook") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let snapshot = control.operator_snapshot(limit).await;
            casebook_response(snapshot.casebook)
        }
        ("GET", "/board") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let snapshot = control.operator_snapshot(limit).await;
            board_response(snapshot.board)
        }
        ("GET", "/timeline") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            timeline_response(dashboard.timeline_catalog)
        }
        ("GET", "/trace") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            trace_response(dashboard.trace)
        }
        ("GET", "/focus") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            focus_response(dashboard.focus)
        }
        ("GET", "/lens") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            lens_response(dashboard.lens)
        }
        ("GET", "/brief") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            brief_response(dashboard.brief)
        }
        ("GET", "/dossier") => {
            let Ok(limit) = history_limit(query) else {
                return invalid_query_error("limit");
            };
            let dashboard = control.operator_dashboard(limit).await;
            dossier_response(dashboard.dossier)
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
        .and_then(|path| path.strip_suffix("/dossier"))
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
        let mut dossier = dashboard.dossier;
        let Some(destination_dossier) = dossier
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        dossier.routes.clear();
        dossier.destinations = vec![destination_dossier];
        return dossier_response(dossier);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/brief"))
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
        let mut brief = dashboard.brief;
        let Some(destination_brief) = brief
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        brief.routes.clear();
        brief.destinations = vec![destination_brief];
        return brief_response(brief);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/lens"))
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
        let mut lens = dashboard.lens;
        let Some(destination_lens) = lens
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        lens.routes.clear();
        lens.destinations = vec![destination_lens];
        return lens_response(lens);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/focus"))
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
        let mut focus = dashboard.focus;
        let Some(destination_focus) = focus
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        focus.routes.clear();
        focus.destinations = vec![destination_focus];
        return focus_response(focus);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/board"))
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
        let snapshot = control.operator_snapshot(limit).await;
        let mut board = snapshot.board;
        board.blocked_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id.as_str())
        });
        board.degraded_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id.as_str())
        });
        board.watch_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id.as_str())
        });
        return board_response(board);
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/casebook"))
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
        let snapshot = control.operator_snapshot(limit).await;
        let Some(destination_casebook) = snapshot
            .casebook
            .destination_casebooks
            .into_iter()
            .find(|casebook| casebook.destination_id == destination_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return casebook_response(crate::ProxyOperatorCasebookCatalog {
            state: snapshot.casebook.state,
            route_casebooks: Vec::new(),
            destination_casebooks: vec![destination_casebook],
        });
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/triage"))
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
        let snapshot = control.operator_snapshot(limit).await;
        let Some(destination_triage) = snapshot
            .triage
            .destination_triage
            .into_iter()
            .find(|triage| triage.destination_id == destination_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return triage_response(crate::ProxyOperatorTriageCatalog {
            state: snapshot.triage.state,
            global: snapshot.triage.global,
            route_triage: Vec::new(),
            destination_triage: vec![destination_triage],
        });
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/timeline"))
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
        let timeline_catalog = dashboard.timeline_catalog;
        let global = timeline_catalog.global.clone();
        let Some(destination_timeline) = timeline_catalog
            .destinations
            .into_iter()
            .find(|timeline| timeline.destination_id == destination_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return timeline_response(ProxyOperatorTimelineCatalog {
            global,
            routes: Vec::new(),
            destinations: vec![destination_timeline],
        });
    }

    if let Some(destination_id) = path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/handoff"))
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
        let snapshot = control.operator_snapshot(limit).await;
        let Some(destination_handoff) = snapshot
            .handoff
            .destination_handoffs
            .into_iter()
            .find(|handoff| handoff.destination_id == destination_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return handoff_response(crate::ProxyOperatorHandoffCatalog {
            state: snapshot.handoff.state,
            route_handoffs: Vec::new(),
            destination_handoffs: vec![destination_handoff],
        });
    }

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

    if let Some(route_id) = route_path.strip_suffix("/brief") {
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
        let mut brief = dashboard.brief;
        let Some(route_brief) = brief
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        brief.routes = vec![route_brief];
        brief.destinations.clear();
        return brief_response(brief);
    }

    if let Some(route_id) = route_path.strip_suffix("/dossier") {
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
        let mut dossier = dashboard.dossier;
        let Some(route_dossier) = dossier
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        dossier.routes = vec![route_dossier];
        dossier.destinations.clear();
        return dossier_response(dossier);
    }

    if let Some(route_id) = route_path.strip_suffix("/lens") {
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
        let mut lens = dashboard.lens;
        let Some(route_lens) = lens
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        lens.routes = vec![route_lens];
        lens.destinations.clear();
        return lens_response(lens);
    }

    if let Some(route_id) = route_path.strip_suffix("/focus") {
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
        let mut focus = dashboard.focus;
        let Some(route_focus) = focus
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            return unsupported_route_error(&request.path);
        };
        focus.routes = vec![route_focus];
        focus.destinations.clear();
        return focus_response(focus);
    }

    if let Some(route_id) = route_path.strip_suffix("/board") {
        if request.method != "GET" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let snapshot = control.operator_snapshot(limit).await;
        let mut board = snapshot.board;
        board.blocked_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id.as_str())
        });
        board.degraded_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id.as_str())
        });
        board.watch_items.retain(|item| {
            item.scope == ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id.as_str())
        });
        return board_response(board);
    }

    if let Some(route_id) = route_path.strip_suffix("/handoff") {
        if request.method != "GET" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let snapshot = control.operator_snapshot(limit).await;
        let Some(route_handoff) = snapshot
            .handoff
            .route_handoffs
            .into_iter()
            .find(|handoff| handoff.route_id == route_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return handoff_response(crate::ProxyOperatorHandoffCatalog {
            state: snapshot.handoff.state,
            route_handoffs: vec![route_handoff],
            destination_handoffs: Vec::new(),
        });
    }

    if let Some(route_id) = route_path.strip_suffix("/casebook") {
        if request.method != "GET" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let snapshot = control.operator_snapshot(limit).await;
        let Some(route_casebook) = snapshot
            .casebook
            .route_casebooks
            .into_iter()
            .find(|casebook| casebook.route_id == route_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return casebook_response(crate::ProxyOperatorCasebookCatalog {
            state: snapshot.casebook.state,
            route_casebooks: vec![route_casebook],
            destination_casebooks: Vec::new(),
        });
    }

    if let Some(route_id) = route_path.strip_suffix("/triage") {
        if request.method != "GET" || route_id.is_empty() {
            return unsupported_route_error(&request.path);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(limit) = history_limit(query) else {
            return invalid_query_error("limit");
        };
        let snapshot = control.operator_snapshot(limit).await;
        let Some(route_triage) = snapshot
            .triage
            .route_triage
            .into_iter()
            .find(|triage| triage.route_id == route_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return triage_response(crate::ProxyOperatorTriageCatalog {
            state: snapshot.triage.state,
            global: snapshot.triage.global,
            route_triage: vec![route_triage],
            destination_triage: Vec::new(),
        });
    }

    if let Some(route_id) = route_path.strip_suffix("/timeline") {
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
        let timeline_catalog = dashboard.timeline_catalog;
        let global = timeline_catalog.global.clone();
        let Some(route_timeline) = timeline_catalog
            .routes
            .into_iter()
            .find(|timeline| timeline.route_id == route_id)
        else {
            return unsupported_route_error(&request.path);
        };
        return timeline_response(ProxyOperatorTimelineCatalog {
            global,
            routes: vec![route_timeline],
            destinations: Vec::new(),
        });
    }

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
