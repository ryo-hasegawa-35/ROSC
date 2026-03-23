use rosc_config::{BrokerConfig, ConfigError, ConfigManager};
use rosc_osc::CompatibilityMode;
use rosc_route::{
    CachePolicy, DestinationRef, LateJoinerPolicy, PersistPolicy, RouteCacheSpec, RouteMatchSpec,
    RouteRecoverySpec, RouteSpec, TrafficClass, TransformSpec, TransportSelector,
};

#[test]
fn config_loader_accepts_phase_01_style_routes() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[routes]]
        id = "ue5_camera_fov"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:9000"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

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
        "#,
    )
    .expect("config should parse");

    assert_eq!(config.schema_version, 1);
    assert_eq!(config.udp_ingresses.len(), 1);
    assert_eq!(config.udp_destinations.len(), 1);
    assert_eq!(config.routes.len(), 1);
    assert_eq!(config.routes[0].id, "ue5_camera_fov");
}

#[test]
fn config_loader_rejects_duplicate_route_ids() {
    let config = BrokerConfig {
        schema_version: 1,
        routes: vec![sample_route("dup", "a"), sample_route("dup", "b")],
        udp_ingresses: Vec::new(),
        udp_destinations: Vec::new(),
    };
    let error = config
        .validate()
        .expect_err("duplicate route ids must fail");

    assert!(matches!(error, ConfigError::DuplicateRouteId(id) if id == "dup"));
}

#[test]
fn config_manager_preserves_last_known_good_on_invalid_candidate() {
    let mut manager = ConfigManager::default();
    let result = manager
        .apply_toml_str(
            r#"
            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"
            [[udp_destinations]]
            id = "udp_renderer"
            bind = "0.0.0.0:0"
            target = "127.0.0.1:9001"
            [routes.match]
            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#,
        )
        .expect("initial config should apply");

    assert_eq!(result.revision, 1);
    assert_eq!(
        manager.current().expect("current config").config.routes[0].id,
        "camera"
    );

    let error = manager.apply_toml_str(
        r#"
        schema_version = 99
        [[routes]]
        id = "broken"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"
        [routes.match]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    );

    assert!(matches!(
        error,
        Err(ConfigError::UnsupportedSchemaVersion(99))
    ));
    let current = manager.current().expect("last known good should remain");
    assert_eq!(current.revision, 1);
    assert_eq!(current.config.routes[0].id, "camera");
}

#[test]
fn config_manager_reports_route_diff() {
    let mut manager = ConfigManager::default();
    manager
        .apply_toml_str(
            r#"
            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"
            [[udp_destinations]]
            id = "udp_renderer"
            bind = "0.0.0.0:0"
            target = "127.0.0.1:9001"
            [routes.match]
            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#,
        )
        .unwrap();

    let diff = manager
        .preview_toml_diff(
            r#"
            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"
            [[udp_destinations]]
            id = "udp_renderer"
            bind = "0.0.0.0:0"
            target = "127.0.0.1:9001"
            [[udp_destinations]]
            id = "tap"
            bind = "0.0.0.0:0"
            target = "127.0.0.1:9002"
            [routes.match]
            [routes.transform]
            rename_address = "/render/camera/fov"
            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"

            [[routes]]
            id = "tracking"
            enabled = true
            mode = "osc1_1_extended"
            class = "SensorStream"
            [routes.match]
            [[routes.destinations]]
            target = "tap"
            transport = "internal"
            "#,
        )
        .unwrap();

    assert_eq!(diff.added_routes, vec!["tracking"]);
    assert_eq!(diff.changed_routes, vec!["camera"]);
    assert!(diff.removed_routes.is_empty());
}

#[test]
fn config_loader_rejects_unknown_udp_destination_reference() {
    let error = BrokerConfig::from_toml_str(
        r#"
        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        [[routes.destinations]]
        target = "missing_udp_target"
        transport = "osc_udp"
        "#,
    )
    .expect_err("unknown udp target should fail validation");

    assert!(matches!(
        error,
        ConfigError::UnknownUdpDestinationReference { route_id, destination_id }
        if route_id == "camera" && destination_id == "missing_udp_target"
    ));
}

#[test]
fn config_loader_rejects_rehydrate_without_cache() {
    let error = BrokerConfig::from_toml_str(
        r#"
        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        [routes.recovery]
        late_joiner = "latest"
        rehydrate_on_connect = true
        [[routes.destinations]]
        target = "loopback"
        transport = "internal"
        "#,
    )
    .expect_err("rehydrate requires cache policy");

    assert!(matches!(
        error,
        ConfigError::RecoveryWithoutCache { route_id } if route_id == "camera"
    ));
}

fn sample_route(id: &str, target: &str) -> RouteSpec {
    RouteSpec {
        id: id.to_owned(),
        enabled: true,
        mode: CompatibilityMode::Osc1_0Strict,
        class: TrafficClass::StatefulControl,
        match_spec: RouteMatchSpec::default(),
        transform: TransformSpec::default(),
        cache: RouteCacheSpec {
            policy: CachePolicy::NoCache,
            ttl_ms: None,
            persist: PersistPolicy::Ephemeral,
        },
        recovery: RouteRecoverySpec {
            late_joiner: LateJoinerPolicy::Disabled,
            rehydrate_on_connect: false,
            rehydrate_on_restart: false,
            replay_allowed: false,
        },
        destinations: vec![DestinationRef {
            target: target.to_owned(),
            transport: TransportSelector::OscUdp,
            enabled: true,
        }],
    }
}
