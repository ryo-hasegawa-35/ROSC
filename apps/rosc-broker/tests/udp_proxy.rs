use std::time::Duration;

use rosc_broker::UdpProxyApp;
use rosc_config::BrokerConfig;
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;

#[tokio::test]
async fn udp_proxy_relays_one_datagram_end_to_end() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
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

    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(80.0)],
    }))
    .unwrap();

    source.send_to(&payload, ingress_addr).await.unwrap();
    assert_eq!(app.relay_once("udp_localhost_in").await.unwrap(), 1);

    let mut buffer = vec![0u8; 2048];
    let (size, _) = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();

    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected relayed OSC message");
    };
    assert_eq!(message.address, "/render/camera/fov");
}

#[tokio::test]
async fn udp_proxy_rehydrates_cached_state_for_destination() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
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

    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(95.0)],
    }))
    .unwrap();

    source.send_to(&payload, ingress_addr).await.unwrap();
    assert_eq!(app.relay_once("udp_localhost_in").await.unwrap(), 1);

    let mut buffer = vec![0u8; 2048];
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(app.rehydrate_destination("udp_renderer").await.unwrap(), 1);

    let (size, _) = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();

    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected rehydrated OSC message");
    };
    assert_eq!(message.address, "/render/camera/fov");
}

#[tokio::test]
async fn udp_proxy_replays_captured_state_to_a_sandbox_destination() {
    let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let live_addr = live_listener.local_addr().unwrap();
    let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sandbox_addr = sandbox_listener.local_addr().unwrap();

    let config = BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:0"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "{live_addr}"

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
    .unwrap();

    let app = UdpProxyApp::from_config(&config, InMemoryTelemetry::default())
        .await
        .unwrap();
    let ingress_addr = app.ingress_local_addr("udp_localhost_in").unwrap();

    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(72.0)],
    }))
    .unwrap();

    source.send_to(&payload, ingress_addr).await.unwrap();
    assert_eq!(app.relay_once("udp_localhost_in").await.unwrap(), 1);

    let mut buffer = vec![0u8; 2048];
    let _ = tokio::time::timeout(Duration::from_secs(1), live_listener.recv_from(&mut buffer))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        app.replay_route_to_sandbox("camera", "sandbox_tap", 10)
            .await
            .unwrap(),
        1
    );

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
}
