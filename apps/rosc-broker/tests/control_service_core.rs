mod common;

use std::time::Duration;

use common::control_service::{
    json_body, proxy_config, request, send_packet, start_proxy, start_proxy_with_policy,
    start_service,
};
use rosc_osc::{ParsedOscPacket, parse_packet};
use serde_json::json;
use tokio::net::UdpSocket;

#[tokio::test]
async fn control_service_freezes_and_thaws_live_proxy() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let freeze_response = request(
        service.listen_addr(),
        "POST /freeze HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(freeze_response.contains("HTTP/1.1 200 OK"));
    assert!(freeze_response.contains("\"action\":\"freeze_traffic\""));
    assert!(freeze_response.contains("\"applied\":true"));

    send_packet(ingress_addr).await;
    let mut buffer = [0u8; 2048];
    let frozen = tokio::time::timeout(
        Duration::from_millis(200),
        destination_listener.recv_from(&mut buffer),
    )
    .await;
    assert!(frozen.is_err(), "frozen control should stop egress");

    let thaw_response = request(
        service.listen_addr(),
        "POST /thaw HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(thaw_response.contains("HTTP/1.1 200 OK"));
    assert!(thaw_response.contains("\"action\":\"thaw_traffic\""));

    send_packet(ingress_addr).await;
    let (size, _) = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected OSC message after thaw");
    };
    assert_eq!(message.address, "/render/camera/fov");

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_exposes_recent_operator_and_config_history() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let _ = request(
        service.listen_addr(),
        "POST /freeze HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    let _ = request(
        service.listen_addr(),
        "POST /thaw HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;

    let operator_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/operator-actions?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let actions = operator_history["actions"].as_array().unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0]["action"], "thaw_traffic");
    assert_eq!(actions[0]["details"], json!(["applied=true"]));

    let config_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/config-events HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let events = config_history["events"].as_array().unwrap();
    assert!(
        events
            .iter()
            .any(|event| event["kind"] == "Applied" && event["revision"] == 1)
    );
    assert!(events.iter().any(|event| {
        event["kind"] == "LaunchProfileChanged" && event["launch_profile_mode"] == "normal"
    }));

    let invalid_limit = request(
        service.listen_addr(),
        "GET /history/operator-actions?limit=0 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(invalid_limit.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_limit.contains("invalid query parameter `limit`"));

    let status = json_body(
        &request(
            service.listen_addr(),
            "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(
        status["status"]["runtime"]["recent_operator_actions"][0]["action"],
        "freeze_traffic"
    );
    assert_eq!(
        status["status"]["runtime"]["recent_operator_actions"][0]["details"],
        json!(["applied=true"])
    );
    assert_eq!(
        status["status"]["runtime"]["recent_config_events"][0]["kind"],
        "Applied"
    );
    assert_eq!(
        status["status"]["runtime"]["recent_config_events"][1]["kind"],
        "LaunchProfileChanged"
    );
    assert_eq!(
        status["status"]["runtime"]["recent_config_events"][1]["revision"],
        1
    );

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_serves_dashboard_assets() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let dashboard = request(
        service.listen_addr(),
        "GET /dashboard HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(dashboard.contains("HTTP/1.1 200 OK"));
    assert!(dashboard.contains("Content-Type: text/html; charset=utf-8"));
    assert!(dashboard.contains("ROSC Operator Console"));
    assert!(dashboard.contains("/dashboard/app.js"));
    assert!(dashboard.contains("Route next steps"));
    assert!(dashboard.contains("Route focus packet"));
    assert!(dashboard.contains("Route operator brief"));
    assert!(dashboard.contains("Route operator runbook"));
    assert!(dashboard.contains("Route operator mission"));
    assert!(dashboard.contains("Route-linked event history"));

    let css = request(
        service.listen_addr(),
        "GET /dashboard/app.css HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(css.contains("HTTP/1.1 200 OK"));
    assert!(css.contains("Content-Type: text/css; charset=utf-8"));
    assert!(css.contains(":root"));

    let js = request(
        service.listen_addr(),
        "GET /dashboard/app.js HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(js.contains("HTTP/1.1 200 OK"));
    assert!(js.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(js.contains("fetchDashboardData"));
    assert!(js.contains("retryDelayMs"));

    let state_js = request(
        service.listen_addr(),
        "GET /dashboard/dashboard-state.js HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(state_js.contains("HTTP/1.1 200 OK"));
    assert!(state_js.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(state_js.contains("buildTrafficPulse"));

    let render_js = request(
        service.listen_addr(),
        "GET /dashboard/dashboard-render.js HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(render_js.contains("HTTP/1.1 200 OK"));
    assert!(render_js.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(render_js.contains("renderDashboard"));
    assert!(render_js.contains("Disconnected (stale)"));
    assert!(render_js.contains("operator isolation active"));
    assert!(render_js.contains("Focused route handoff"));

    let dashboard_data = json_body(
        &request(
            service.listen_addr(),
            "GET /dashboard/data?limit=2 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(dashboard_data["ok"], true);
    assert_eq!(dashboard_data["dashboard"]["refresh_interval_ms"], 2500);
    assert_eq!(
        dashboard_data["dashboard"]["snapshot"]["overview"]["status"]["summary"]["active_routes"],
        1
    );
    assert_eq!(
        dashboard_data["dashboard"]["traffic"]["has_runtime_status"],
        true
    );
    assert_eq!(
        dashboard_data["dashboard"]["route_details"][0]["route_id"],
        "camera"
    );
    assert_eq!(
        dashboard_data["dashboard"]["destination_details"][0]["destination_id"],
        "udp_renderer"
    );
    assert!(
        dashboard_data["dashboard"]["trace"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["trace"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["timeline_catalog"]["global"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["timeline_catalog"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["timeline_catalog"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["handoff"]["route_handoffs"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["handoff"]["destination_handoffs"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["triage"]["route_triage"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["casebook"]["route_casebooks"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["board"]["degraded_items"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["focus"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["brief"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["dossier"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["runbook"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["focus"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["brief"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["lens"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["lens"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["runbook"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["mission"]["routes"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["mission"]["destinations"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["worklist"]["items"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["incident_digest"]["clusters"]
            .as_array()
            .is_some()
    );
    assert!(
        dashboard_data["dashboard"]["snapshot"]["recovery"]["route_candidates"]
            .as_array()
            .is_some()
    );
    assert_eq!(
        dashboard_data["dashboard"]["timeline"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_exposes_operator_report_blockers_and_scoped_signals() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy_with_policy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
        rosc_broker::ProxyRuntimeSafetyPolicy {
            fail_on_warnings: true,
            require_fallback_ready: true,
        },
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let _ = request(
        service.listen_addr(),
        "POST /freeze HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    let _ = request(
        service.listen_addr(),
        "POST /routes/camera/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;

    let report = json_body(
        &request(
            service.listen_addr(),
            "GET /report HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(report["ok"], true);
    assert_eq!(report["report"]["policy"]["fail_on_warnings"], true);
    assert_eq!(report["report"]["policy"]["require_fallback_ready"], true);
    assert_eq!(report["report"]["state"], "warning");
    assert_eq!(report["report"]["overrides"]["traffic_frozen"], true);
    assert_eq!(
        report["report"]["runtime_signals"]["destinations_with_open_breakers"],
        json!([])
    );
    assert_eq!(
        report["report"]["highlights"]["latest_operator_action"]["action"],
        "isolate_route"
    );

    let overview = json_body(
        &request(
            service.listen_addr(),
            "GET /overview HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(overview["ok"], true);
    assert_eq!(overview["overview"]["report"]["state"], "warning");
    assert_eq!(
        overview["overview"]["problematic_signals"]["scope"],
        "problematic"
    );
    assert!(
        overview["overview"]["problematic_signals"]["route_signals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["route_id"] == "camera" && route["isolated"] == true)
    );

    let readiness = json_body(
        &request(
            service.listen_addr(),
            "GET /readiness HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(readiness["ok"], true);
    assert_eq!(readiness["readiness"]["level"], "degraded");
    assert_eq!(readiness["readiness"]["ready"], false);
    assert_eq!(readiness["readiness"]["flags"]["traffic_flow_ready"], false);
    assert_eq!(readiness["readiness"]["counts"]["problematic_routes"], 1);
    assert_eq!(
        readiness["readiness"]["counts"]["problematic_destinations"],
        0
    );
    assert!(
        readiness["readiness"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "traffic is currently frozen by operator override")
    );

    let readyz_blocked = request(
        service.listen_addr(),
        "GET /readyz HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(readyz_blocked.contains("HTTP/1.1 503 Service Unavailable"));

    let readyz_allowed = request(
        service.listen_addr(),
        "GET /readyz?allow_degraded=true HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(readyz_allowed.contains("HTTP/1.1 200 OK"));

    let invalid_readyz_query = request(
        service.listen_addr(),
        "GET /readyz?allow_degraded=maybe HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(invalid_readyz_query.contains("HTTP/1.1 400 Bad Request"));

    let snapshot = json_body(
        &request(
            service.listen_addr(),
            "GET /snapshot?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(snapshot["ok"], true);
    assert_eq!(snapshot["snapshot"]["readiness"]["level"], "degraded");
    assert_eq!(snapshot["snapshot"]["attention"]["state"], "warning");
    assert_eq!(
        snapshot["snapshot"]["incidents"]["recent_operator_actions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot["snapshot"]["diagnostics"]["overview"]["report"]["state"],
        "warning"
    );

    let blockers = json_body(
        &request(
            service.listen_addr(),
            "GET /blockers HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(blockers["ok"], true);
    assert!(blockers["blockers"].as_array().unwrap().is_empty());

    let diagnostics = json_body(
        &request(
            service.listen_addr(),
            "GET /diagnostics?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(diagnostics["ok"], true);
    assert_eq!(
        diagnostics["diagnostics"]["overview"]["report"]["state"],
        "warning"
    );
    assert_eq!(
        diagnostics["diagnostics"]["overview"]["runtime_summary"]["traffic_frozen"],
        true
    );
    assert_eq!(
        diagnostics["diagnostics"]["recent_operator_actions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let trace = json_body(
        &request(
            service.listen_addr(),
            "GET /trace?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(trace["ok"], true);
    assert!(
        trace["trace"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["route_id"] == "camera")
    );
    assert!(
        trace["trace"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|destination| destination["destination_id"] == "udp_renderer")
    );

    let route_trace = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/trace?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_trace["ok"], true);
    assert_eq!(route_trace["route_trace"]["route_id"], "camera");
    assert!(
        route_trace["route_trace"]["recent_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["kind"] == "override")
    );

    let destination_trace = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/trace?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_trace["ok"], true);
    assert_eq!(
        destination_trace["destination_trace"]["destination_id"],
        "udp_renderer"
    );

    let attention = json_body(
        &request(
            service.listen_addr(),
            "GET /attention HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(attention["ok"], true);
    assert_eq!(attention["attention"]["state"], "warning");
    assert_eq!(attention["attention"]["traffic_frozen"], true);
    assert_eq!(
        attention["attention"]["isolated_route_ids"],
        json!(["camera"])
    );
    assert_eq!(
        attention["attention"]["latest_operator_action"]["action"],
        "isolate_route"
    );

    let incidents = json_body(
        &request(
            service.listen_addr(),
            "GET /incidents?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(incidents["ok"], true);
    assert_eq!(incidents["incidents"]["state"], "warning");
    assert_eq!(
        incidents["incidents"]["problematic_routes"][0]["route_id"],
        "camera"
    );
    assert_eq!(
        incidents["incidents"]["recent_operator_actions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let handoff = json_body(
        &request(
            service.listen_addr(),
            "GET /handoff?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(handoff["ok"], true);
    assert!(
        handoff["handoff"]["route_handoffs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );

    let triage = json_body(
        &request(
            service.listen_addr(),
            "GET /triage?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(triage["ok"], true);
    assert!(
        triage["triage"]["global"]["next_steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step.as_str().unwrap().contains("Thaw traffic"))
    );

    let casebook = json_body(
        &request(
            service.listen_addr(),
            "GET /casebook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(casebook["ok"], true);
    assert!(
        casebook["casebook"]["route_casebooks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );

    let board = json_body(
        &request(
            service.listen_addr(),
            "GET /board?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(board["ok"], true);
    assert!(
        board["board"]["degraded_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        board["board"]["degraded_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["scope"] == "global")
    );

    let focus = json_body(
        &request(
            service.listen_addr(),
            "GET /focus?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(focus["ok"], true);
    assert!(
        focus["focus"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        focus["focus"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let lens = json_body(
        &request(
            service.listen_addr(),
            "GET /lens?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(lens["ok"], true);
    assert!(
        lens["lens"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        lens["lens"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let brief = json_body(
        &request(
            service.listen_addr(),
            "GET /brief?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(brief["ok"], true);
    assert!(brief["brief"]["global_next_steps"].as_array().is_some());
    assert!(
        brief["brief"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        brief["brief"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let dossier = json_body(
        &request(
            service.listen_addr(),
            "GET /dossier?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(dossier["ok"], true);
    assert!(
        dossier["dossier"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        dossier["dossier"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let runbook = json_body(
        &request(
            service.listen_addr(),
            "GET /runbook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(runbook["ok"], true);
    assert_eq!(runbook["runbook"]["global"]["state"], "warning");
    assert!(
        runbook["runbook"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        runbook["runbook"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let mission = json_body(
        &request(
            service.listen_addr(),
            "GET /mission?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(mission["ok"], true);
    assert_eq!(mission["mission"]["global"]["state"], "warning");
    assert!(
        !mission["mission"]["global"]["gate_reasons"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        mission["mission"]["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        mission["mission"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
    );

    let timeline = json_body(
        &request(
            service.listen_addr(),
            "GET /timeline?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(timeline["ok"], true);
    assert!(
        timeline["timeline"]["global"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["label"] == "freeze_traffic")
    );

    let route_handoff = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/handoff?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_handoff["ok"], true);
    assert_eq!(
        route_handoff["handoff"]["route_handoffs"][0]["route_id"],
        "camera"
    );

    let route_triage = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/triage?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_triage["ok"], true);
    assert_eq!(
        route_triage["triage"]["route_triage"][0]["route_id"],
        "camera"
    );

    let route_casebook = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/casebook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_casebook["ok"], true);
    assert_eq!(
        route_casebook["casebook"]["route_casebooks"][0]["route_id"],
        "camera"
    );

    let route_focus = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/focus?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_focus["ok"], true);
    assert_eq!(route_focus["focus"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_focus["focus"]["destinations"], json!([]));

    let route_lens = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/lens?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_lens["ok"], true);
    assert_eq!(route_lens["lens"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_lens["lens"]["destinations"], json!([]));
    assert!(
        route_lens["lens"]["routes"][0]["global_overrides"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == "traffic_frozen")
    );

    let route_brief = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/brief?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_brief["ok"], true);
    assert_eq!(route_brief["brief"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_brief["brief"]["destinations"], json!([]));
    assert!(
        route_brief["brief"]["routes"][0]["global_overrides"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == "traffic_frozen")
    );
    assert!(
        route_brief["brief"]["routes"][0]["scoped_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry.as_str().unwrap().contains("camera"))
    );

    let route_dossier = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/dossier?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_dossier["ok"], true);
    assert_eq!(route_dossier["dossier"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_dossier["dossier"]["destinations"], json!([]));
    assert!(route_dossier["dossier"]["routes"][0]["brief"].is_object());
    assert!(route_dossier["dossier"]["routes"][0]["lens"].is_object());

    let route_runbook = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/runbook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_runbook["ok"], true);
    assert_eq!(route_runbook["runbook"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_runbook["runbook"]["destinations"], json!([]));
    assert!(route_runbook["runbook"]["routes"][0]["dossier"].is_object());
    assert_eq!(route_runbook["runbook"]["routes"][0]["state"], "warning");

    let route_mission = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/mission?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_mission["ok"], true);
    assert_eq!(route_mission["mission"]["routes"][0]["route_id"], "camera");
    assert_eq!(route_mission["mission"]["destinations"], json!([]));
    assert!(route_mission["mission"]["routes"][0]["brief"].is_object());
    assert!(route_mission["mission"]["routes"][0]["dossier"].is_object());
    assert!(route_mission["mission"]["routes"][0]["runbook"].is_object());

    let route_board = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/board?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_board["ok"], true);
    assert!(
        route_board["board"]["degraded_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["route_id"] == "camera")
    );
    assert!(
        route_board["board"]["degraded_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["scope"] == "global")
    );

    let route_timeline = json_body(
        &request(
            service.listen_addr(),
            "GET /routes/camera/timeline?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(route_timeline["ok"], true);
    assert_eq!(
        route_timeline["timeline"]["routes"][0]["route_id"],
        "camera"
    );

    let destination_handoff = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/handoff?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_handoff["ok"], true);
    assert_eq!(
        destination_handoff["handoff"]["destination_handoffs"][0]["destination_id"],
        "udp_renderer"
    );

    let destination_triage = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/triage?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_triage["ok"], true);
    assert_eq!(
        destination_triage["triage"]["destination_triage"][0]["destination_id"],
        "udp_renderer"
    );

    let destination_casebook = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/casebook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_casebook["ok"], true);
    assert_eq!(
        destination_casebook["casebook"]["destination_casebooks"][0]["destination_id"],
        "udp_renderer"
    );

    let destination_focus = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/focus?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_focus["ok"], true);
    assert_eq!(
        destination_focus["focus"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_focus["focus"]["routes"], json!([]));

    let destination_lens = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/lens?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_lens["ok"], true);
    assert_eq!(
        destination_lens["lens"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_lens["lens"]["routes"], json!([]));
    assert!(
        destination_lens["lens"]["destinations"][0]["global_overrides"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == "traffic_frozen")
    );

    let destination_brief = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/brief?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_brief["ok"], true);
    assert_eq!(
        destination_brief["brief"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_brief["brief"]["routes"], json!([]));
    assert!(
        destination_brief["brief"]["destinations"][0]["global_overrides"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == "traffic_frozen")
    );
    assert!(destination_brief["brief"]["destinations"][0]["scoped_blockers"].is_array());

    let destination_dossier = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/dossier?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_dossier["ok"], true);
    assert_eq!(
        destination_dossier["dossier"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_dossier["dossier"]["routes"], json!([]));
    assert!(destination_dossier["dossier"]["destinations"][0]["brief"].is_object());
    assert!(destination_dossier["dossier"]["destinations"][0]["lens"].is_object());

    let destination_runbook = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/runbook?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_runbook["ok"], true);
    assert_eq!(
        destination_runbook["runbook"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_runbook["runbook"]["routes"], json!([]));
    assert!(destination_runbook["runbook"]["destinations"][0]["dossier"].is_object());

    let destination_mission = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/mission?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_mission["ok"], true);
    assert_eq!(
        destination_mission["mission"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );
    assert_eq!(destination_mission["mission"]["routes"], json!([]));
    assert!(destination_mission["mission"]["destinations"][0]["brief"].is_object());
    assert!(destination_mission["mission"]["destinations"][0]["dossier"].is_object());
    assert!(destination_mission["mission"]["destinations"][0]["runbook"].is_object());

    let destination_board = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/board?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_board["ok"], true);
    assert!(
        destination_board["board"]["watch_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["destination_id"] == "udp_renderer")
            || destination_board["board"]["degraded_items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["destination_id"] == "udp_renderer")
    );
    assert!(
        destination_board["board"]["degraded_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["scope"] == "global")
    );

    let destination_timeline = json_body(
        &request(
            service.listen_addr(),
            "GET /destinations/udp_renderer/timeline?limit=4 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(destination_timeline["ok"], true);
    assert_eq!(
        destination_timeline["timeline"]["destinations"][0]["destination_id"],
        "udp_renderer"
    );

    let overrides = json_body(
        &request(
            service.listen_addr(),
            "GET /overrides HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(overrides["ok"], true);
    assert_eq!(overrides["overrides"]["traffic_frozen"], true);
    assert_eq!(
        overrides["overrides"]["launch_profile_mode"],
        json!("normal")
    );

    let signals = json_body(
        &request(
            service.listen_addr(),
            "GET /signals HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(signals["ok"], true);
    assert_eq!(signals["scope"], "all");
    assert_eq!(
        signals["runtime_signals"]["routes_with_dispatch_failures"],
        json!([])
    );
    assert!(
        signals["route_signals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["route_id"] == "camera")
    );
    assert!(
        signals["destination_signals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|destination| destination["destination_id"] == "udp_renderer")
    );

    let problematic_signals = json_body(
        &request(
            service.listen_addr(),
            "GET /signals?scope=problematic HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    assert_eq!(problematic_signals["ok"], true);
    assert_eq!(problematic_signals["scope"], "problematic");
    let filtered_routes = problematic_signals["route_signals"].as_array().unwrap();
    assert_eq!(filtered_routes.len(), 1);
    assert_eq!(filtered_routes[0]["route_id"], "camera");
    assert_eq!(filtered_routes[0]["isolated"], true);
    assert_eq!(problematic_signals["destination_signals"], json!([]));

    let invalid_scope = request(
        service.listen_addr(),
        "GET /signals?scope=unexpected HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(invalid_scope.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_scope.contains("invalid query parameter `scope`"));

    let invalid_diagnostics_limit = request(
        service.listen_addr(),
        "GET /diagnostics?limit=0 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(invalid_diagnostics_limit.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_diagnostics_limit.contains("invalid query parameter `limit`"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}
