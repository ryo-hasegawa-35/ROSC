use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rosc_broker::{ManagedProxyFileSupervisor, ProxyReloadOutcome, ProxyRuntimeSafetyPolicy};
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;

fn unique_config_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("rosc-managed-proxy-{nonce}.toml"))
}

fn proxy_config(ingress_bind: &str, destination_addr: &str, rename_address: &str) -> String {
    format!(
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
    )
}

fn blocked_config() -> &'static str {
    r#"
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
    "#
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

async fn send_test_packet(target: std::net::SocketAddr) {
    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(82.0)],
    }))
    .unwrap();
    source.send_to(&payload, target).await.unwrap();
}

#[tokio::test]
async fn proxy_reload_supervisor_blocks_unsafe_candidate_and_keeps_last_known_good() {
    let path = unique_config_path();
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    fs::write(
        &path,
        proxy_config(
            "127.0.0.1:0",
            &listener.local_addr().unwrap().to_string(),
            "/render/good",
        ),
    )
    .unwrap();

    let mut supervisor = ManagedProxyFileSupervisor::start(
        &path,
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy {
            fail_on_warnings: true,
            require_fallback_ready: true,
        },
    )
    .await
    .unwrap();

    fs::write(&path, blocked_config()).unwrap();
    let outcome = supervisor.poll_once().await.unwrap();
    match outcome {
        ProxyReloadOutcome::Blocked(reasons) => {
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("direct UDP fallback"))
            );
        }
        other => panic!("expected blocked config, got {other:?}"),
    }
    assert_eq!(supervisor.current_revision(), Some(1));
    let runtime = supervisor
        .status_snapshot()
        .runtime
        .expect("managed proxy status should expose runtime");
    assert_eq!(runtime.config_revision, 1);
    assert_eq!(runtime.config_rejections_total, 1);

    send_test_packet(
        supervisor
            .proxy()
            .app()
            .ingress_local_addr("udp_localhost_in")
            .unwrap(),
    )
    .await;
    let message = recv_message(&listener).await;
    assert_eq!(message.address, "/render/good");

    supervisor.shutdown().await;
    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn proxy_reload_supervisor_rolls_back_after_runtime_reload_failure() {
    let path = unique_config_path();
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    fs::write(
        &path,
        proxy_config(
            "127.0.0.1:0",
            &listener.local_addr().unwrap().to_string(),
            "/render/good",
        ),
    )
    .unwrap();

    let mut supervisor = ManagedProxyFileSupervisor::start(
        &path,
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
    )
    .await
    .unwrap();
    let ingress_addr = supervisor
        .proxy()
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();

    fs::write(
        &path,
        proxy_config(
            &ingress_addr.to_string(),
            &ingress_addr.to_string(),
            "/render/bad",
        ),
    )
    .unwrap();

    let outcome = supervisor.poll_once().await.unwrap();
    match outcome {
        ProxyReloadOutcome::ReloadFailed(reason) => {
            assert!(reason.contains("failed to reload managed proxy"));
        }
        other => panic!("expected runtime reload failure, got {other:?}"),
    }
    assert_eq!(supervisor.current_revision(), Some(1));
    let runtime = supervisor
        .status_snapshot()
        .runtime
        .expect("managed proxy status should expose runtime");
    assert_eq!(runtime.config_revision, 1);
    assert_eq!(runtime.config_rejections_total, 1);

    send_test_packet(
        supervisor
            .proxy()
            .app()
            .ingress_local_addr("udp_localhost_in")
            .unwrap(),
    )
    .await;
    let message = recv_message(&listener).await;
    assert_eq!(message.address, "/render/good");

    supervisor.shutdown().await;
    let _ = fs::remove_file(path);
}
