use super::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::commands::ProxyCommandOptions;
use crate::commands::runtime::live::startup_options;

static UNIQUE_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_config_path() -> PathBuf {
    let nonce = UNIQUE_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("rosc-commands-{pid}-{nonce}.toml"))
}

fn proxy_config(ingress_bind: &str, destination_addr: &str) -> rosc_config::BrokerConfig {
    rosc_config::BrokerConfig::from_toml_str(&format!(
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

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
    ))
    .unwrap()
}

fn proxy_config_toml(ingress_bind: &str, destination_addr: &str) -> String {
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

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
    )
}

#[tokio::test]
async fn start_managed_proxy_sidecars_releases_ingress_port_when_control_startup_fails() {
    let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let telemetry = InMemoryTelemetry::default();
    let proxy = Arc::new(Mutex::new(
        rosc_broker::ManagedUdpProxy::start(
            proxy_config(
                "127.0.0.1:0",
                &destination_listener.local_addr().unwrap().to_string(),
            ),
            telemetry.clone(),
            32,
            rosc_broker::ProxyRuntimeSafetyPolicy::default(),
            rosc_broker::ProxyLaunchProfileMode::Normal,
            startup_options(ProxyCommandOptions {
                fail_on_warnings: false,
                require_fallback_ready: false,
                safe_mode: false,
                start_frozen: false,
            }),
        )
        .await
        .unwrap(),
    ));
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
        rosc_broker::ManagedUdpProxyController::new(Arc::clone(&proxy)),
    );

    let error = match start_managed_proxy_sidecars(
        &proxy,
        telemetry,
        None,
        Some("0.0.0.0:0"),
        control_plane,
    )
    .await
    {
        Ok(_) => panic!("non-loopback control listener should fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("control listener must bind to a loopback address")
    );

    let rebound = tokio::net::UdpSocket::bind(ingress_addr)
        .await
        .expect("ingress port should be released after control startup failure");
    assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
}

#[tokio::test]
async fn start_managed_proxy_sidecars_releases_ingress_port_when_health_startup_fails() {
    let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let occupied_health = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let telemetry = InMemoryTelemetry::default();
    let proxy = Arc::new(Mutex::new(
        rosc_broker::ManagedUdpProxy::start(
            proxy_config(
                "127.0.0.1:0",
                &destination_listener.local_addr().unwrap().to_string(),
            ),
            telemetry.clone(),
            32,
            rosc_broker::ProxyRuntimeSafetyPolicy::default(),
            rosc_broker::ProxyLaunchProfileMode::Normal,
            startup_options(ProxyCommandOptions {
                fail_on_warnings: false,
                require_fallback_ready: false,
                safe_mode: false,
                start_frozen: false,
            }),
        )
        .await
        .unwrap(),
    ));
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
        rosc_broker::ManagedUdpProxyController::new(Arc::clone(&proxy)),
    );

    let error = match start_managed_proxy_sidecars(
        &proxy,
        telemetry,
        Some(&occupied_health.local_addr().unwrap().to_string()),
        None,
        control_plane,
    )
    .await
    {
        Ok(_) => panic!("occupied health listener should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("failed to bind health listener"));

    let rebound = tokio::net::UdpSocket::bind(ingress_addr)
        .await
        .expect("ingress port should be released after health startup failure");
    assert_eq!(rebound.local_addr().unwrap(), ingress_addr);
}

#[tokio::test]
async fn start_supervisor_sidecars_releases_ingress_port_when_control_startup_fails() {
    let destination_listener = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let path = unique_config_path();
    fs::write(
        &path,
        proxy_config_toml(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
    )
    .unwrap();

    let telemetry = InMemoryTelemetry::default();
    let supervisor = Arc::new(Mutex::new(
        rosc_broker::ManagedProxyFileSupervisor::start(
            &path,
            telemetry.clone(),
            32,
            rosc_broker::ProxyRuntimeSafetyPolicy::default(),
            rosc_broker::ProxyLaunchProfileMode::Normal,
            startup_options(ProxyCommandOptions {
                fail_on_warnings: false,
                require_fallback_ready: false,
                safe_mode: false,
                start_frozen: false,
            }),
        )
        .await
        .unwrap(),
    ));
    let ingress_addr = supervisor
        .lock()
        .await
        .proxy()
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let control_plane: Arc<dyn rosc_broker::ProxyControlPlane> = Arc::new(
        rosc_broker::ManagedProxyFileSupervisorController::new(Arc::clone(&supervisor)),
    );

    let error = match start_supervisor_sidecars(
        &supervisor,
        telemetry,
        None,
        Some("0.0.0.0:0"),
        control_plane,
    )
    .await
    {
        Ok(_) => panic!("non-loopback control listener should fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("control listener must bind to a loopback address")
    );

    let rebound = tokio::net::UdpSocket::bind(ingress_addr)
        .await
        .expect("ingress port should be released after supervisor control startup failure");
    assert_eq!(rebound.local_addr().unwrap(), ingress_addr);

    let _ = fs::remove_file(path);
}
