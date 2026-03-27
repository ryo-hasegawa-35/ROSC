use std::sync::Arc;
use std::time::Duration;

use rosc_broker::{
    ControlService, ManagedProxyStartupOptions, ManagedUdpProxy, ManagedUdpProxyController,
    ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy,
};
use rosc_config::BrokerConfig;
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;

fn proxy_config(ingress_bind: &str, destination_addr: &str) -> BrokerConfig {
    BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "{ingress_bind}"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "{destination_addr}"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.transform]
        rename_address = "/render/camera/fov"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#
    ))
    .unwrap()
}

fn replayable_proxy_config(
    ingress_bind: &str,
    destination_addr: &str,
    sandbox_addr: &str,
) -> BrokerConfig {
    BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "{ingress_bind}"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "{destination_addr}"

        [[udp_destinations]]
        id = "sandbox_tap"
        bind = "127.0.0.1:0"
        target = "{sandbox_addr}"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.transform]
        rename_address = "/render/camera/fov"

        [routes.cache]
        policy = "last_value_per_address"
        ttl_ms = 10000
        persist = "warm"

        [routes.recovery]
        late_joiner = "latest"
        rehydrate_on_connect = true
        replay_allowed = true

        [routes.observability]
        capture = "always_bounded"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#
    ))
    .unwrap()
}

fn custom_id_proxy_config(
    ingress_bind: &str,
    destination_addr: &str,
    destination_id: &str,
    route_id: &str,
) -> BrokerConfig {
    BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "{ingress_bind}"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "{destination_id}"
        bind = "127.0.0.1:0"
        target = "{destination_addr}"

        [[routes]]
        id = "{route_id}"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.transform]
        rename_address = "/render/camera/fov"

        [routes.cache]
        policy = "last_value_per_address"
        ttl_ms = 10000
        persist = "warm"

        [routes.recovery]
        late_joiner = "latest"
        rehydrate_on_connect = true

        [[routes.destinations]]
        target = "{destination_id}"
        transport = "osc_udp"
        "#
    ))
    .unwrap()
}

async fn start_proxy(config: BrokerConfig, queue_depth: usize) -> Arc<Mutex<ManagedUdpProxy>> {
    Arc::new(Mutex::new(
        ManagedUdpProxy::start(
            config,
            InMemoryTelemetry::default(),
            queue_depth,
            ProxyRuntimeSafetyPolicy::default(),
            ProxyLaunchProfileMode::Normal,
            ManagedProxyStartupOptions::default(),
        )
        .await
        .unwrap(),
    ))
}

async fn send_packet(target: std::net::SocketAddr) {
    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(80.0)],
    }))
    .unwrap();
    source.send_to(&payload, target).await.unwrap();
}

async fn request(addr: std::net::SocketAddr, raw_request: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(raw_request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

async fn open_partial_request(addr: std::net::SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /status HTTP/1.1\r\nHost: localhost\r\n")
        .await
        .unwrap();
    stream
}

fn json_body(response: &str) -> Value {
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response body should exist");
    serde_json::from_str(body).expect("response body should be valid JSON")
}

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
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

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
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

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
    assert_eq!(actions[0]["details"], serde_json::json!(["applied=true"]));

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
    assert!(
        events.iter().any(|event| {
            event["kind"] == "LaunchProfileChanged" && event["launch_profile_mode"] == "normal"
        }),
        "expected launch profile event in history: {events:?}"
    );

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
        serde_json::json!(["applied=true"])
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
async fn control_service_exposes_operator_report_blockers_and_scoped_signals() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = Arc::new(Mutex::new(
        ManagedUdpProxy::start(
            proxy_config(
                "127.0.0.1:0",
                &destination_listener.local_addr().unwrap().to_string(),
            ),
            InMemoryTelemetry::default(),
            32,
            ProxyRuntimeSafetyPolicy {
                fail_on_warnings: true,
                require_fallback_ready: true,
            },
            ProxyLaunchProfileMode::Normal,
            ManagedProxyStartupOptions::default(),
        )
        .await
        .unwrap(),
    ));
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

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
        serde_json::json!([])
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
        serde_json::json!(["camera"])
    );
    assert_eq!(
        attention["attention"]["latest_operator_action"]["action"],
        "isolate_route"
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
        serde_json::json!("normal")
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
        serde_json::json!([])
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
    assert_eq!(
        problematic_signals["destination_signals"],
        serde_json::json!([])
    );

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

#[tokio::test]
async fn control_service_can_isolate_routes_and_report_unknown_routes() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let isolate_response = request(
        service.listen_addr(),
        "POST /routes/camera/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(isolate_response.contains("HTTP/1.1 200 OK"));
    assert!(isolate_response.contains("\"isolated_route_ids\":[\"camera\"]"));

    let status_response = request(
        service.listen_addr(),
        "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(status_response.contains("HTTP/1.1 200 OK"));
    assert!(status_response.contains("\"isolated_route_ids\":[\"camera\"]"));

    let missing_response = request(
        service.listen_addr(),
        "POST /routes/missing/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(missing_response.contains("HTTP/1.1 404 Not Found"));
    assert!(missing_response.contains("unknown route `missing`"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_can_restore_all_isolated_routes() {
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
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let _ = request(
        service.listen_addr(),
        "POST /routes/camera/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;

    let restore_response = request(
        service.listen_addr(),
        "POST /routes/restore-all HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(restore_response.contains("HTTP/1.1 200 OK"));
    assert!(restore_response.contains("\"action\":\"restore_all_routes\""));
    assert!(restore_response.contains("\"dispatch_count\":1"));
    assert!(restore_response.contains("\"isolated_route_ids\":[]"));

    let operator_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/operator-actions HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let actions = operator_history["actions"].as_array().unwrap();
    assert!(actions.iter().any(|action| {
        action["action"] == "restore_all_routes"
            && action["details"]
                == serde_json::json!(["restored_count=1", "route_ids=camera", "applied=true"])
    }));

    send_packet(ingress_addr).await;
    let mut buffer = [0u8; 2048];
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_can_rehydrate_and_replay_to_sandbox() {
    let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        replayable_proxy_config(
            "127.0.0.1:0",
            &live_listener.local_addr().unwrap().to_string(),
            &sandbox_listener.local_addr().unwrap().to_string(),
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
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    send_packet(ingress_addr).await;
    let mut buffer = [0u8; 2048];
    let _ = tokio::time::timeout(Duration::from_secs(1), live_listener.recv_from(&mut buffer))
        .await
        .unwrap()
        .unwrap();

    let rehydrate_response = request(
        service.listen_addr(),
        "POST /destinations/udp_renderer/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(rehydrate_response.contains("HTTP/1.1 200 OK"));
    assert!(rehydrate_response.contains("\"action\":\"rehydrate_destination\""));
    assert!(rehydrate_response.contains("\"dispatch_count\":1"));

    let (size, _) =
        tokio::time::timeout(Duration::from_secs(1), live_listener.recv_from(&mut buffer))
            .await
            .unwrap()
            .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected rehydrated OSC message");
    };
    assert_eq!(message.address, "/render/camera/fov");

    let replay_response = request(
        service.listen_addr(),
        "POST /routes/camera/replay/sandbox_tap?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(replay_response.contains("HTTP/1.1 200 OK"));
    assert!(replay_response.contains("\"action\":\"sandbox_replay\""));
    assert!(replay_response.contains("\"dispatch_count\":1"));

    let operator_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/operator-actions HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let actions = operator_history["actions"].as_array().unwrap();
    assert!(actions.iter().any(|action| {
        action["action"] == "rehydrate_destination"
            && action["details"]
                == serde_json::json!([
                    "destination_id=udp_renderer",
                    "dispatch_count=1",
                    "applied=true"
                ])
    }));
    assert!(actions.iter().any(|action| {
        action["action"] == "sandbox_replay"
            && action["details"]
                == serde_json::json!([
                    "route_id=camera",
                    "sandbox_destination_id=sandbox_tap",
                    "limit=1",
                    "dispatch_count=1",
                    "applied=true"
                ])
    }));

    let (size, _) = tokio::time::timeout(
        Duration::from_secs(1),
        sandbox_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected sandbox replay OSC message");
    };
    assert_eq!(message.address, "/render/camera/fov");

    let unknown_destination_response = request(
        service.listen_addr(),
        "POST /destinations/missing/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(unknown_destination_response.contains("HTTP/1.1 404 Not Found"));
    assert!(unknown_destination_response.contains("unknown destination `missing`"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_decodes_percent_encoded_route_and_destination_ids() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let route_id = "camera/main?1";
    let destination_id = "udp/renderer?1";
    let proxy = start_proxy(
        custom_id_proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
            destination_id,
            route_id,
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
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let isolate_response = request(
        service.listen_addr(),
        "POST /routes/camera%2Fmain%3F1/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(isolate_response.contains("HTTP/1.1 200 OK"));
    assert!(isolate_response.contains("\"isolated_route_ids\":[\"camera/main?1\"]"));

    send_packet(ingress_addr).await;
    let mut buffer = [0u8; 2048];
    let blocked = tokio::time::timeout(
        Duration::from_millis(200),
        destination_listener.recv_from(&mut buffer),
    )
    .await;
    assert!(blocked.is_err());

    let restore_response = request(
        service.listen_addr(),
        "POST /routes/camera%2Fmain%3F1/restore HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(restore_response.contains("HTTP/1.1 200 OK"));

    send_packet(ingress_addr).await;
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();

    let rehydrate_response = request(
        service.listen_addr(),
        "POST /destinations/udp%2Frenderer%3F1/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(rehydrate_response.contains("HTTP/1.1 200 OK"));
    assert!(rehydrate_response.contains("\"dispatch_count\":1"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_rejects_invalid_percent_encoding() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let response = request(
        service.listen_addr(),
        "POST /routes/camera%2/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    assert!(response.contains("invalid percent-encoding in route id"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_rejects_invalid_replay_limit() {
    let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        replayable_proxy_config(
            "127.0.0.1:0",
            &live_listener.local_addr().unwrap().to_string(),
            &sandbox_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let zero_response = request(
        service.listen_addr(),
        "POST /routes/camera/replay/sandbox_tap?limit=0 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(zero_response.contains("HTTP/1.1 400 Bad Request"));
    assert!(zero_response.contains("invalid query parameter `limit`"));

    let malformed_response = request(
        service.listen_addr(),
        "POST /routes/camera/replay/sandbox_tap?limit=1x HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(malformed_response.contains("HTTP/1.1 400 Bad Request"));
    assert!(malformed_response.contains("invalid query parameter `limit`"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_rejects_non_loopback_listener() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let error = match ControlService::spawn("0.0.0.0:0", controller).await {
        Ok(_) => panic!("non-loopback control listener should be rejected"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("control listener must bind to a loopback address")
    );

    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_accepts_localhost_listener() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("localhost:0", controller)
        .await
        .unwrap();
    assert!(service.listen_addr().ip().is_loopback());

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_slow_client_does_not_block_other_requests() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let slow_stream = open_partial_request(service.listen_addr()).await;
    let fast_response = tokio::time::timeout(
        Duration::from_millis(500),
        request(
            service.listen_addr(),
            "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
        ),
    )
    .await
    .expect("a slow client should not block later requests");
    assert!(fast_response.contains("HTTP/1.1 200 OK"));

    drop(slow_stream);
    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_shutdown_is_not_blocked_by_partial_request() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let _slow_stream = open_partial_request(service.listen_addr()).await;
    tokio::time::timeout(Duration::from_millis(500), service.shutdown())
        .await
        .expect("shutdown should not wait on a partial request")
        .unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_times_out_partial_request_and_recovers() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let mut service = ControlService::spawn("127.0.0.1:0", controller)
        .await
        .unwrap();

    let mut slow_stream = open_partial_request(service.listen_addr()).await;
    let mut timeout_response = String::new();
    tokio::time::timeout(
        Duration::from_secs(3),
        slow_stream.read_to_string(&mut timeout_response),
    )
    .await
    .expect("partial request should receive a timeout response")
    .unwrap();
    assert!(timeout_response.contains("HTTP/1.1 408 Request Timeout"));
    assert!(timeout_response.contains("request headers not received within"));

    let fast_response = request(
        service.listen_addr(),
        "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(fast_response.contains("HTTP/1.1 200 OK"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}
