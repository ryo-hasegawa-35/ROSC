use std::time::Duration;

use rosc_broker::{
    FrozenStartupBehavior, ManagedProxyStartupOptions, ManagedUdpProxy, ProxyLaunchProfileMode,
    ProxyRuntimeSafetyPolicy,
};
use rosc_config::BrokerConfig;
use rosc_osc::{
    OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;

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

async fn send_packet(target: std::net::SocketAddr, value: f32) {
    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(value)],
    }))
    .unwrap();
    source.send_to(&payload, target).await.unwrap();
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
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        80.0,
    )
    .await;
    let first_message = recv_message(&first_listener).await;
    assert_eq!(first_message.address, "/render/a");

    proxy.reload(config_b).await.unwrap();
    let runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should be present after reload");
    assert_eq!(runtime.config_revision, 2);

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        80.0,
    )
    .await;
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
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
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

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        81.0,
    )
    .await;
    let restored_message = recv_message(&listener).await;
    assert_eq!(restored_message.address, "/render/good");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_status_exposes_runtime_config_after_startup() {
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
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    let status = proxy.app().status_snapshot();
    let runtime = status.runtime.expect("runtime snapshot should be present");

    assert!(!runtime.traffic_frozen);
    assert_eq!(runtime.config_revision, 1);
    assert_eq!(runtime.config_rejections_total, 0);
    assert_eq!(runtime.config_blocked_total, 0);
    assert_eq!(runtime.config_reload_failures_total, 0);

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_start_frozen_blocks_startup_traffic_without_a_race() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let config = proxy_config(
        &ingress_addr.to_string(),
        &listener.local_addr().unwrap().to_string(),
        "/render/start-frozen",
    );

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
    let sender_task = tokio::spawn(async move {
        let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: "/ue5/camera/fov".to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(83.0)],
        }))
        .unwrap();
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                _ = tokio::time::sleep(Duration::from_millis(2)) => {
                    let _ = source.send_to(&payload, ingress_addr).await;
                }
            }
        }
    });

    let mut proxy = ManagedUdpProxy::start(
        config,
        InMemoryTelemetry::default(),
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

    let runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should be present");
    assert!(runtime.traffic_frozen);
    assert_eq!(
        runtime
            .operator_actions_total
            .get("freeze_traffic")
            .copied(),
        Some(1)
    );

    let mut buffer = [0u8; 2048];
    let no_delivery =
        tokio::time::timeout(Duration::from_millis(150), listener.recv_from(&mut buffer)).await;
    assert!(no_delivery.is_err(), "start-frozen traffic should not leak");

    proxy.thaw_traffic();
    let _ = stop_tx.send(());
    let _ = sender_task.await;

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        84.0,
    )
    .await;
    let message = recv_message(&listener).await;
    assert_eq!(message.address, "/render/start-frozen");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_can_freeze_and_thaw_traffic() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut proxy = ManagedUdpProxy::start(
        proxy_config(
            &ingress_addr.to_string(),
            &listener.local_addr().unwrap().to_string(),
            "/render/frozen",
        ),
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    proxy.freeze_traffic();
    let frozen_runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist");
    assert!(frozen_runtime.traffic_frozen);
    assert_eq!(
        frozen_runtime
            .operator_actions_total
            .get("freeze_traffic")
            .copied(),
        Some(1)
    );

    proxy.thaw_traffic();
    let thawed_runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist");
    assert!(!thawed_runtime.traffic_frozen);
    assert_eq!(
        thawed_runtime
            .operator_actions_total
            .get("thaw_traffic")
            .copied(),
        Some(1)
    );

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_can_isolate_and_restore_routes() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut proxy = ManagedUdpProxy::start(
        proxy_config(
            &ingress_addr.to_string(),
            &listener.local_addr().unwrap().to_string(),
            "/render/isolation",
        ),
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    assert!(proxy.isolate_route("camera"));
    let isolated_runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist");
    assert_eq!(isolated_runtime.isolated_route_ids, vec!["camera"]);

    assert!(proxy.restore_route("camera"));
    let restored_runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist");
    assert!(restored_runtime.isolated_route_ids.is_empty());
    assert_eq!(
        restored_runtime
            .operator_actions_total
            .get("isolate_route")
            .copied(),
        Some(1)
    );
    assert_eq!(
        restored_runtime
            .operator_actions_total
            .get("restore_route")
            .copied(),
        Some(1)
    );

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_preserves_frozen_state_across_reload() {
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
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    proxy.freeze_traffic();
    proxy.reload(config_b).await.unwrap();

    let runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist after reload");
    assert!(runtime.traffic_frozen);
    assert_eq!(
        runtime
            .operator_actions_total
            .get("freeze_traffic")
            .copied(),
        Some(1)
    );

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        85.0,
    )
    .await;

    let mut buffer = [0u8; 2048];
    let no_delivery = tokio::time::timeout(
        Duration::from_millis(150),
        second_listener.recv_from(&mut buffer),
    )
    .await;
    assert!(
        no_delivery.is_err(),
        "frozen reload should keep traffic blocked"
    );

    proxy.thaw_traffic();
    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        86.0,
    )
    .await;
    let message = recv_message(&second_listener).await;
    assert_eq!(message.address, "/render/b");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_preserves_route_isolation_across_reload() {
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
        ProxyLaunchProfileMode::Normal,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    assert!(proxy.isolate_route("camera"));
    proxy.reload(config_b).await.unwrap();

    let runtime = proxy
        .app()
        .status_snapshot()
        .runtime
        .expect("runtime snapshot should exist after reload");
    assert_eq!(runtime.isolated_route_ids, vec!["camera"]);
    assert_eq!(
        runtime.operator_actions_total.get("isolate_route").copied(),
        Some(1)
    );

    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        88.0,
    )
    .await;
    let mut buffer = [0u8; 2048];
    let no_delivery = tokio::time::timeout(
        Duration::from_millis(150),
        second_listener.recv_from(&mut buffer),
    )
    .await;
    assert!(
        no_delivery.is_err(),
        "reloaded isolated route should stay blocked"
    );

    assert!(proxy.restore_route("camera"));
    send_packet(
        proxy.app().ingress_local_addr("udp_localhost_in").unwrap(),
        89.0,
    )
    .await;
    let message = recv_message(&second_listener).await;
    assert_eq!(message.address, "/render/b");

    proxy.shutdown().await;
}

#[tokio::test]
async fn managed_proxy_safe_mode_marks_launch_profile_and_disables_optional_features() {
    let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let ingress_addr = reserved.local_addr().unwrap();
    drop(reserved);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let config = BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "{ingress_addr}"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "{target}"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.recovery]
        replay_allowed = true
        rehydrate_on_restart = true

        [routes.cache]
        policy = "last_value_per_address"

        [routes.observability]
        capture = "always_bounded"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
        target = listener.local_addr().unwrap()
    ))
    .unwrap();

    let mut proxy = ManagedUdpProxy::start(
        config,
        InMemoryTelemetry::default(),
        32,
        ProxyRuntimeSafetyPolicy::default(),
        ProxyLaunchProfileMode::SafeMode,
        ManagedProxyStartupOptions::default(),
    )
    .await
    .unwrap();

    let status = proxy.app().status_snapshot();
    assert_eq!(status.launch_profile.mode, ProxyLaunchProfileMode::SafeMode);
    assert_eq!(
        status.launch_profile.disabled_capture_routes,
        vec!["camera"]
    );
    assert_eq!(status.launch_profile.disabled_replay_routes, vec!["camera"]);
    assert_eq!(
        status.launch_profile.disabled_restart_rehydrate_routes,
        vec!["camera"]
    );

    proxy.shutdown().await;
}
