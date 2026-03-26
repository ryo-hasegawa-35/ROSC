use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rosc_broker::{
    FrozenStartupBehavior, ManagedProxyFileSupervisor, ManagedProxyStartupOptions,
    ProxyLaunchProfileMode, ProxyReloadOutcome, ProxyRuntimeSafetyPolicy,
};
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;

static UNIQUE_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_config_path() -> PathBuf {
    let nonce = UNIQUE_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("rosc-managed-proxy-{pid}-{nonce}.toml"))
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
    let telemetry = InMemoryTelemetry::default();
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
        telemetry.clone(),
        32,
        ProxyRuntimeSafetyPolicy {
            fail_on_warnings: true,
            require_fallback_ready: true,
        },
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
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
    assert_eq!(runtime.config_rejections_total, 0);
    assert_eq!(runtime.config_blocked_total, 1);
    assert_eq!(runtime.config_reload_failures_total, 0);
    let metrics = telemetry.snapshot();
    assert_eq!(metrics.config_added_ingresses_total, 1);
    assert_eq!(metrics.config_added_destinations_total, 1);
    assert_eq!(metrics.config_added_routes_total, 1);

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
    let telemetry = InMemoryTelemetry::default();
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
        telemetry.clone(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
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
    assert_eq!(runtime.config_rejections_total, 0);
    assert_eq!(runtime.config_blocked_total, 0);
    assert_eq!(runtime.config_reload_failures_total, 1);
    let metrics = telemetry.snapshot();
    assert_eq!(metrics.config_added_ingresses_total, 1);
    assert_eq!(metrics.config_added_destinations_total, 1);
    assert_eq!(metrics.config_added_routes_total, 1);

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
async fn proxy_reload_supervisor_applies_single_config_transition_per_reload() {
    let path = unique_config_path();
    let first_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let second_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let telemetry = InMemoryTelemetry::default();

    fs::write(
        &path,
        proxy_config(
            "127.0.0.1:0",
            &first_listener.local_addr().unwrap().to_string(),
            "/render/a",
        ),
    )
    .unwrap();

    let mut supervisor = ManagedProxyFileSupervisor::start(
        &path,
        telemetry.clone(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    let after_start = telemetry.snapshot();
    assert_eq!(after_start.config_revision, 1);
    assert_eq!(after_start.config_added_ingresses_total, 1);
    assert_eq!(after_start.config_added_destinations_total, 1);
    assert_eq!(after_start.config_added_routes_total, 1);

    fs::write(
        &path,
        proxy_config(
            "127.0.0.1:0",
            &second_listener.local_addr().unwrap().to_string(),
            "/render/b",
        ),
    )
    .unwrap();

    let outcome = supervisor.poll_once().await.unwrap();
    match outcome {
        ProxyReloadOutcome::Applied(applied) => {
            assert_eq!(applied.revision, 2);
            assert_eq!(applied.diff.changed_destinations, vec!["udp_renderer"]);
            assert_eq!(applied.diff.changed_routes, vec!["camera"]);
        }
        other => panic!("expected applied reload, got {other:?}"),
    }

    let after_reload = telemetry.snapshot();
    assert_eq!(after_reload.config_revision, 2);
    assert_eq!(after_reload.config_added_ingresses_total, 1);
    assert_eq!(after_reload.config_added_destinations_total, 1);
    assert_eq!(after_reload.config_added_routes_total, 1);
    assert_eq!(after_reload.config_changed_ingresses_total, 0);
    assert_eq!(after_reload.config_changed_destinations_total, 1);
    assert_eq!(after_reload.config_changed_routes_total, 1);

    supervisor.shutdown().await;
    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn proxy_reload_supervisor_can_start_frozen() {
    let path = unique_config_path();
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let telemetry = InMemoryTelemetry::default();

    fs::write(
        &path,
        proxy_config(
            "127.0.0.1:0",
            &listener.local_addr().unwrap().to_string(),
            "/render/frozen",
        ),
    )
    .unwrap();

    let mut supervisor = ManagedProxyFileSupervisor::start(
        &path,
        telemetry,
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions {
            frozen_behavior: FrozenStartupBehavior::OperatorRequested,
            ..ManagedProxyStartupOptions::default()
        },
    )
    .await
    .unwrap();

    let runtime = supervisor
        .status_snapshot()
        .runtime
        .expect("managed proxy status should expose runtime");
    assert!(runtime.traffic_frozen);
    assert_eq!(
        runtime
            .operator_actions_total
            .get("freeze_traffic")
            .copied(),
        Some(1)
    );

    supervisor.shutdown().await;
    let _ = fs::remove_file(path);
}
