use std::time::Duration;

use rosc_broker::{ManagedUdpProxy, ProxyRuntimeSafetyPolicy, emit_initial_config_applied};
use rosc_config::BrokerConfig;
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;

fn proxy_config(ingress_bind: &str, destination_addr: &str, rename_address: &str) -> BrokerConfig {
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
        rename_address = "{rename_address}"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#
    ))
    .expect("config should parse")
}

async fn recv_message(listener: &UdpSocket) -> OscMessage {
    let mut buffer = vec![0u8; 2048];
    let (size, _) = tokio::time::timeout(Duration::from_secs(1), listener.recv_from(&mut buffer))
        .await
        .expect("receive should complete")
        .expect("receive should succeed");
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict)
        .expect("packet should parse");
    let ParsedOscPacket::Message(message) = parsed else {
        panic!("expected OSC message");
    };
    message
}

#[tokio::test]
async fn managed_proxy_reloads_to_a_new_destination() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let first_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let second_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let config_a = proxy_config(
        &ingress_addr.to_string(),
        &first_listener.local_addr().unwrap().to_string(),
        "/render/a",
    );
    let config_b = proxy_config(
        &ingress_addr.to_string(),
        &second_listener.local_addr().unwrap().to_string(),
        "/render/b",
    );

    let mut proxy = ManagedUdpProxy::start(
        config_a,
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
    )
    .await
    .unwrap();

    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(80.0)],
    }))
    .unwrap();

    source
        .send_to(
            &payload,
            proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        )
        .await
        .unwrap();
    let first_message = recv_message(&first_listener).await;
    assert_eq!(first_message.address, "/render/a");

    proxy.reload(config_b).await.unwrap();

    source
        .send_to(
            &payload,
            proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        )
        .await
        .unwrap();
    let second_message = recv_message(&second_listener).await;
    assert_eq!(second_message.address, "/render/b");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_rolls_back_when_reload_fails() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let good_config = proxy_config(
        &ingress_addr.to_string(),
        &listener.local_addr().unwrap().to_string(),
        "/render/good",
    );
    let bad_config = proxy_config(
        &ingress_addr.to_string(),
        &ingress_addr.to_string(),
        "/render/bad",
    );

    let mut proxy = ManagedUdpProxy::start(
        good_config,
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
    )
    .await
    .unwrap();

    let error = proxy
        .reload(bad_config)
        .await
        .expect_err("reload should fail");
    assert!(
        error
            .to_string()
            .contains("failed to apply the new proxy configuration")
    );

    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(81.0)],
    }))
    .unwrap();

    source
        .send_to(
            &payload,
            proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        )
        .await
        .unwrap();
    let restored_message = recv_message(&listener).await;
    assert_eq!(restored_message.address, "/render/good");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_status_exposes_runtime_config_after_initial_seed() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let telemetry = InMemoryTelemetry::default();
    let mut proxy = ManagedUdpProxy::start(
        proxy_config(
            &ingress_addr.to_string(),
            &listener.local_addr().unwrap().to_string(),
            "/render/status",
        ),
        telemetry.clone(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
    )
    .await
    .unwrap();

    emit_initial_config_applied(&telemetry, proxy.config());
    let status = proxy.app().status_snapshot();
    let runtime = status.runtime.expect("runtime snapshot should be present");

    assert_eq!(runtime.config_revision, 1);
    assert_eq!(runtime.config_rejections_total, 0);

    proxy.shutdown().await;
}
