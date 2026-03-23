use rosc_config::{BrokerConfig, ConfigError};
use rosc_osc::CompatibilityMode;
use rosc_route::{
    DestinationRef, RouteMatchSpec, RouteSpec, TrafficClass, TransformSpec, TransportSelector,
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

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.transform]
        rename_address = "/render/camera/fov"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .expect("config should parse");

    assert_eq!(config.routes.len(), 1);
    assert_eq!(config.routes[0].id, "ue5_camera_fov");
}

#[test]
fn config_loader_rejects_duplicate_route_ids() {
    let config = BrokerConfig {
        routes: vec![sample_route("dup", "a"), sample_route("dup", "b")],
    };
    let error = config
        .validate()
        .expect_err("duplicate route ids must fail");

    assert!(matches!(error, ConfigError::DuplicateRouteId(id) if id == "dup"));
}

fn sample_route(id: &str, target: &str) -> RouteSpec {
    RouteSpec {
        id: id.to_owned(),
        enabled: true,
        mode: CompatibilityMode::Osc1_0Strict,
        class: TrafficClass::StatefulControl,
        match_spec: RouteMatchSpec::default(),
        transform: TransformSpec::default(),
        destinations: vec![DestinationRef {
            target: target.to_owned(),
            transport: TransportSelector::OscUdp,
            enabled: true,
        }],
    }
}
