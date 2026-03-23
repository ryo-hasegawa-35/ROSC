use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use rosc_config::BrokerConfig;
use rosc_osc::{
    CompatibilityMode, OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet,
};
use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
use rosc_runtime::{
    IngressQueue, QueueError, QueuePolicy, Runtime, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::{BrokerEvent, TelemetrySink};
use tokio::net::UdpSocket;

#[test]
fn ingress_queue_applies_bounded_capacity() {
    let (queue, _rx) = IngressQueue::new(QueuePolicy { max_depth: 1 });
    let packet = PacketEnvelope::parse_osc(
        vec![
            0x2f, 0x66, 0x6f, 0x6f, 0, 0, 0, 0, 0x2c, 0x69, 0, 0, 0, 0, 0, 1,
        ],
        IngressMetadata {
            ingress_id: "udp_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: None,
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .unwrap();

    queue.try_send(packet.clone()).unwrap();
    let error = queue.try_send(packet).unwrap_err();
    assert!(matches!(error, QueueError::QueueFull));
}

#[test]
fn runtime_emits_route_match_events() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[routes]]
        id = "fov"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let routing = rosc_route::RoutingEngine::new(config.routes).unwrap();
    let sink = RecordingSink::default();
    let runtime = Runtime {
        routing,
        telemetry: sink.clone(),
    };

    let packet = PacketEnvelope::parse_osc(
        vec![
            0x2f, 0x75, 0x65, 0x35, 0x2f, 0x63, 0x61, 0x6d, 0x65, 0x72, 0x61, 0x2f, 0x66, 0x6f,
            0x76, 0x00, 0x2c, 0x66, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
        ],
        IngressMetadata {
            ingress_id: "udp_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: None,
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .unwrap();

    assert_eq!(runtime.route_packet(&packet).unwrap(), 1);
    let events = sink.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], BrokerEvent::RouteMatched { route_id } if route_id == "fov"));
}

#[tokio::test]
async fn udp_ingress_binding_receives_and_parses_datagrams() {
    let binding = UdpIngressBinding::bind(
        "127.0.0.1:0",
        UdpIngressConfig {
            ingress_id: "udp_localhost_in".to_owned(),
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            max_packet_size: 2048,
        },
    )
    .await
    .unwrap();
    let target = binding.local_addr().unwrap();

    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/ue5/camera/fov".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![OscArgument::Float32(90.0)],
    }))
    .unwrap();
    client.send_to(&payload, target).await.unwrap();

    let packet = binding.recv_next().await.unwrap();
    assert_eq!(packet.metadata.ingress_id, "udp_localhost_in");
    assert_eq!(packet.metadata.transport, TransportKind::OscUdp);
    assert_eq!(packet.address(), Some("/ue5/camera/fov"));
}

#[derive(Clone, Default)]
struct RecordingSink {
    events: Arc<Mutex<Vec<BrokerEvent>>>,
}

impl TelemetrySink for RecordingSink {
    fn emit(&self, event: BrokerEvent) {
        self.events.lock().unwrap().push(event);
    }
}
