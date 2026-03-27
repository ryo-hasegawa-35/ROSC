#![allow(dead_code)]

use std::sync::Arc;

use rosc_broker::{
    ControlService, ManagedProxyStartupOptions, ManagedUdpProxy, ManagedUdpProxyController,
    ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy,
};
use rosc_config::BrokerConfig;
use rosc_osc::{OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet};
use rosc_telemetry::InMemoryTelemetry;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;

pub type SharedProxy = Arc<Mutex<ManagedUdpProxy>>;

pub fn proxy_config(ingress_bind: &str, destination_addr: &str) -> BrokerConfig {
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
        rename_address = "/render/camera/fov"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#
    ))
    .unwrap()
}

pub fn replayable_proxy_config(
    ingress_bind: &str,
    destination_addr: &str,
    sandbox_addr: &str,
) -> BrokerConfig {
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
    .unwrap()
}

pub fn custom_id_proxy_config(
    ingress_bind: &str,
    destination_addr: &str,
    destination_id: &str,
    route_id: &str,
) -> BrokerConfig {
    BrokerConfig::from_toml_str(&format!(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "{ingress_bind}"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "{destination_id}"
        bind = "127.0.0.1:0"
        target = "{destination_addr}"

        [[routes]]
        id = "{route_id}"
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
        target = "{destination_id}"
        transport = "osc_udp"
        "#
    ))
    .unwrap()
}

pub async fn start_proxy(config: BrokerConfig, queue_depth: usize) -> SharedProxy {
    start_proxy_with_policy(config, queue_depth, ProxyRuntimeSafetyPolicy::default()).await
}

pub async fn start_proxy_with_policy(
    config: BrokerConfig,
    queue_depth: usize,
    policy: ProxyRuntimeSafetyPolicy,
) -> SharedProxy {
    Arc::new(Mutex::new(
        ManagedUdpProxy::start(
            config,
            InMemoryTelemetry::default(),
            queue_depth,
            policy,
            ProxyLaunchProfileMode::Normal,
            ManagedProxyStartupOptions::default(),
        )
        .await
        .unwrap(),
    ))
}

pub async fn start_service(proxy: &SharedProxy, listen: &str) -> ControlService {
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(proxy)));
    ControlService::spawn(listen, controller).await.unwrap()
}

pub async fn send_packet(target: std::net::SocketAddr) {
    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(80.0)],
    }))
    .unwrap();
    source.send_to(&payload, target).await.unwrap();
}

pub async fn request(addr: std::net::SocketAddr, raw_request: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(raw_request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

pub async fn open_partial_request(addr: std::net::SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /status HTTP/1.1\r\nHost: localhost\r\n")
        .await
        .unwrap();
    stream
}

pub fn json_body(response: &str) -> Value {
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response body should exist");
    serde_json::from_str(body).expect("response body should be valid JSON")
}
