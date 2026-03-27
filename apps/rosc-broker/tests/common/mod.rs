#![allow(dead_code)]

pub mod control_service;

use rosc_config::BrokerConfig;

pub fn broad_scope_config() -> BrokerConfig {
    BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"

        [[routes]]
        id = "unsafe"
        enabled = true
        mode = "osc1_0_strict"
        class = "SensorStream"
        [routes.match]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "shadow"
        transport = "internal"
        "#,
    )
    .expect("config should parse")
}
