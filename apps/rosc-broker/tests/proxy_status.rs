use rosc_broker::{UdpProxyApp, operator_warnings, proxy_status_from_config, startup_blockers};
use rosc_config::BrokerConfig;
use rosc_osc::{OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet};
use rosc_telemetry::InMemoryTelemetry;

#[test]
fn proxy_status_summarizes_sidecar_routes_and_fallback_targets() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:9000"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"
        [udp_destinations.policy]
        queue_depth = 32

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

        [routes.recovery]
        late_joiner = "latest"
        rehydrate_on_connect = true

        [routes.observability]
        capture = "always_bounded"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let status = proxy_status_from_config(&config).unwrap();

    assert_eq!(status.summary.total_routes, 1);
    assert_eq!(status.summary.active_routes, 1);
    assert_eq!(status.summary.disabled_routes, 0);
    assert_eq!(status.summary.active_ingresses, 1);
    assert_eq!(status.summary.active_destinations, 1);
    assert_eq!(status.summary.fallback_ready_routes, 1);
    assert_eq!(status.summary.fallback_missing_routes, 0);
    assert!(status.runtime.is_none());
    assert_eq!(status.ingresses.len(), 1);
    assert_eq!(status.ingresses[0].route_ids, vec!["camera"]);
    assert_eq!(status.destinations.len(), 1);
    assert_eq!(status.destinations[0].route_ids, vec!["camera"]);
    assert_eq!(status.routes.len(), 1);
    assert_eq!(status.routes[0].destination_ids, vec!["udp_renderer"]);
    assert!(status.routes[0].rehydrate_on_connect);
    assert_eq!(status.fallback_routes.len(), 1);
    assert!(status.fallback_routes[0].available);
    assert_eq!(
        status.fallback_routes[0].direct_udp_targets,
        vec!["127.0.0.1:9001"]
    );
    assert_eq!(status.route_assessments.len(), 1);
    assert!(status.route_assessments[0].active);
    assert!(status.route_assessments[0].direct_udp_fallback_available);
    assert_eq!(status.route_assessments[0].warning_count, 0);
    assert!(status.warnings.is_empty());
}

#[tokio::test]
async fn live_proxy_status_exposes_bound_local_addr_when_requested() {
    let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let destination_addr = destination_listener.local_addr().unwrap();

    let config = BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:0"
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

        [routes.cache]
        policy = "last_value_per_address"
        ttl_ms = 10000
        persist = "warm"

        [routes.recovery]
        late_joiner = "latest"
        rehydrate_on_connect = true

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#
    ))
    .unwrap();

    let app = UdpProxyApp::from_config(&config, InMemoryTelemetry::default())
        .await
        .unwrap();
    let ingress_addr = app.ingress_local_addr("udp_localhost_in").unwrap();
    let source = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(80.0)],
    }))
    .unwrap();
    source.send_to(&payload, ingress_addr).await.unwrap();
    assert_eq!(app.relay_once("udp_localhost_in").await.unwrap(), 1);
    let mut buffer = [0u8; 2048];
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();

    let status = app.status_snapshot();

    assert_eq!(status.ingresses.len(), 1);
    let bound = status.ingresses[0]
        .bound_local_addr
        .as_ref()
        .expect("live status should resolve bound address");
    assert!(bound.starts_with("127.0.0.1:"));
    let runtime = status
        .runtime
        .expect("live status should include runtime snapshot");
    assert_eq!(runtime.config_revision, 0);
    assert_eq!(runtime.config_rejections_total, 0);
    assert_eq!(
        runtime.ingress_packets_total.get("udp_localhost_in"),
        Some(&1)
    );
    assert_eq!(runtime.route_matches_total.get("camera"), Some(&1));
    assert_eq!(runtime.destinations.len(), 1);
    assert_eq!(runtime.destinations[0].destination_id, "udp_renderer");
    assert_eq!(runtime.destinations[0].send_total, 1);
}

#[test]
fn proxy_status_excludes_disabled_routes_from_active_usage_summary() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:9000"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

        [[routes]]
        id = "camera"
        enabled = false
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let status = proxy_status_from_config(&config).unwrap();

    assert_eq!(status.summary.total_routes, 1);
    assert_eq!(status.summary.active_routes, 0);
    assert_eq!(status.summary.disabled_routes, 1);
    assert_eq!(status.summary.active_ingresses, 0);
    assert_eq!(status.summary.active_destinations, 0);
    assert_eq!(status.routes.len(), 1);
    assert!(!status.routes[0].enabled);
    assert!(status.ingresses[0].route_ids.is_empty());
    assert!(status.destinations[0].route_ids.is_empty());
    assert!(status.fallback_routes.is_empty());
    assert_eq!(status.route_assessments.len(), 1);
    assert!(!status.route_assessments[0].active);
    assert!(!status.route_assessments[0].direct_udp_fallback_available);
    assert!(status.warnings.is_empty());
}

#[test]
fn proxy_status_reports_missing_fallback_and_broad_scope_warnings() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:9000"
        mode = "osc1_0_strict"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]

        [[routes.destinations]]
        target = "tap"
        transport = "internal"
        "#,
    )
    .unwrap();

    let status = proxy_status_from_config(&config).unwrap();

    assert_eq!(status.summary.active_routes, 1);
    assert_eq!(status.summary.fallback_ready_routes, 0);
    assert_eq!(status.summary.fallback_missing_routes, 1);
    assert_eq!(status.route_assessments.len(), 1);
    assert_eq!(status.route_assessments[0].warning_count, 3);
    assert!(
        status.route_assessments[0]
            .warnings
            .contains(&"matches all ingresses".to_owned())
    );
    assert!(
        status.route_assessments[0]
            .warnings
            .contains(&"matches all addresses".to_owned())
    );
    assert!(
        status.route_assessments[0]
            .warnings
            .contains(&"no direct udp fallback target".to_owned())
    );
    assert_eq!(status.warnings.len(), 2);
    let warnings = operator_warnings(&status);
    assert_eq!(warnings.len(), 5);
    let blockers = startup_blockers(&status, true, true);
    assert_eq!(blockers.len(), 6);
    assert!(
        blockers
            .iter()
            .any(|entry| entry.contains("direct UDP fallback"))
    );
}
