use rosc_broker::proxy_status_from_config;
use rosc_config::BrokerConfig;

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
    assert!(status.warnings.is_empty());
}
