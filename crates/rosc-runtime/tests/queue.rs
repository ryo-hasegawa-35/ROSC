use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_trait::async_trait;
use rosc_config::BrokerConfig;
use rosc_osc::{
    CompatibilityMode, OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet,
};
use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
use rosc_route::{RouteCacheSpec, RouteObservabilitySpec, RouteRecoverySpec, TransformSpec};
use rosc_runtime::{
    BreakerState, DestinationDispatchError, DestinationPolicy, DestinationRegistry,
    DestinationSendError, DestinationWorkerHandle, DropPolicy, EgressSink, IngressQueue,
    QueueError, QueuePolicy, Runtime, UdpEgressSink, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::InMemoryTelemetry;
use tokio::net::UdpSocket;

#[test]
fn ingress_queue_applies_bounded_capacity() {
    let (queue, _rx) = IngressQueue::new(QueuePolicy { max_depth: 1 });
    let packet = sample_packet("/foo");

    queue.try_send(packet.clone()).unwrap();
    let error = queue.try_send(packet).unwrap_err();
    assert!(matches!(error, QueueError::QueueFull));
}

#[tokio::test]
async fn runtime_dispatches_and_exports_health_snapshot() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

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
    let telemetry = InMemoryTelemetry::default();
    let runtime = Runtime {
        routing,
        telemetry: telemetry.clone(),
    };

    let recorder = Arc::new(RecordingSink::default());
    let mut destinations = DestinationRegistry::default();
    destinations.register(DestinationWorkerHandle::spawn(
        "udp_renderer",
        DestinationPolicy::default(),
        recorder.clone(),
        Arc::new(telemetry.clone()),
    ));

    let packet = sample_packet("/ue5/camera/fov");
    let outcome = runtime.dispatch_packet(&packet, &destinations).await;
    assert!(outcome.failures.is_empty());
    assert_eq!(outcome.dispatched, 1);

    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    assert_eq!(recorder.sent.lock().unwrap().len(), 1);
    assert_eq!(
        destinations.status("udp_renderer").unwrap().breaker_state,
        BreakerState::Closed
    );

    let health = telemetry.render_prometheus();
    assert!(health.contains("rosc_route_matches_total{route_id=\"fov\"} 1"));
    assert!(health.contains("rosc_destination_send_total{destination_id=\"udp_renderer\"} 1"));
}

#[tokio::test]
async fn failing_destination_opens_breaker_without_blocking_healthy_peer() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "healthy"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

        [[udp_destinations]]
        id = "failing"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9002"

        [[routes]]
        id = "fanout"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "healthy"
        transport = "osc_udp"
        [[routes.destinations]]
        target = "failing"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let routing = rosc_route::RoutingEngine::new(config.routes).unwrap();
    let telemetry = InMemoryTelemetry::default();
    let runtime = Runtime {
        routing,
        telemetry: telemetry.clone(),
    };

    let healthy = Arc::new(RecordingSink::default());
    let failing = Arc::new(FailingSink);

    let mut destinations = DestinationRegistry::default();
    destinations.register(DestinationWorkerHandle::spawn(
        "healthy",
        DestinationPolicy::default(),
        healthy.clone(),
        Arc::new(telemetry.clone()),
    ));
    destinations.register(DestinationWorkerHandle::spawn(
        "failing",
        DestinationPolicy {
            drop_policy: DropPolicy::DropNewest,
            ..DestinationPolicy::default()
        },
        failing,
        Arc::new(telemetry.clone()),
    ));

    for _ in 0..3 {
        let outcome = runtime
            .dispatch_packet(&sample_packet("/ue5/camera/fov"), &destinations)
            .await;
        assert!(outcome.failures.is_empty());
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(healthy.sent.lock().unwrap().len(), 3);
    assert_eq!(
        destinations.status("failing").unwrap().breaker_state,
        BreakerState::Open
    );

    let error = destinations
        .dispatch(rosc_route::RouteDispatch {
            route_id: "manual".to_owned(),
            destination: rosc_route::DestinationRef {
                target: "failing".to_owned(),
                transport: rosc_route::TransportSelector::OscUdp,
                enabled: true,
            },
            packet: sample_packet("/ue5/camera/fov"),
            transform: TransformSpec::default(),
            cache: RouteCacheSpec::default(),
            recovery: RouteRecoverySpec::default(),
            observability: RouteObservabilitySpec::default(),
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        rosc_runtime::RuntimeDispatchError::Destination(
            DestinationDispatchError::BreakerOpen { .. }
        )
    ));
}

#[tokio::test]
async fn route_transform_failure_does_not_block_other_matching_routes() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[routes]]
        id = "rename_bundle"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        protocols = ["osc_udp"]
        [routes.transform]
        rename_address = "/renamed"
        [[routes.destinations]]
        target = "broken"
        transport = "internal"

        [[routes]]
        id = "tap_bundle"
        enabled = true
        mode = "osc1_0_strict"
        class = "Telemetry"
        [routes.match]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "healthy"
        transport = "internal"
        "#,
    )
    .unwrap();

    let routing = rosc_route::RoutingEngine::new(config.routes).unwrap();
    let telemetry = InMemoryTelemetry::default();
    let runtime = Runtime {
        routing,
        telemetry: telemetry.clone(),
    };

    let healthy = Arc::new(RecordingSink::default());
    let broken = Arc::new(RecordingSink::default());
    let mut destinations = DestinationRegistry::default();
    destinations.register(DestinationWorkerHandle::spawn(
        "healthy",
        DestinationPolicy::default(),
        healthy.clone(),
        Arc::new(telemetry.clone()),
    ));
    destinations.register(DestinationWorkerHandle::spawn(
        "broken",
        DestinationPolicy::default(),
        broken,
        Arc::new(telemetry.clone()),
    ));

    let packet = PacketEnvelope::parse_osc(
        encode_packet(&ParsedOscPacket::Bundle(rosc_osc::OscBundle {
            timetag: 1,
            elements: vec![ParsedOscPacket::Message(OscMessage {
                address: "/foo".to_owned(),
                type_tag_source: TypeTagSource::Explicit,
                arguments: vec![OscArgument::Int32(1)],
            })],
        }))
        .unwrap(),
        IngressMetadata {
            ingress_id: "udp_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: None,
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .unwrap();

    let outcome = runtime.dispatch_packet(&packet, &destinations).await;
    assert_eq!(outcome.dispatched, 1);
    assert_eq!(outcome.failures.len(), 1);
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;

    assert_eq!(healthy.sent.lock().unwrap().len(), 1);
    let health = telemetry.render_prometheus();
    assert!(health.contains("rosc_route_matches_total{route_id=\"tap_bundle\"} 1"));
    assert!(health.contains("rosc_route_transform_failures_total{route_id=\"rename_bundle\"} 1"));
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

#[tokio::test]
async fn udp_egress_sink_writes_raw_datagrams() {
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let target = listener.local_addr().unwrap();
    let sink = UdpEgressSink::bind("127.0.0.1:0", target).await.unwrap();
    let packet = sample_packet("/ue5/camera/fov");

    sink.send(&packet).await.unwrap();

    let mut buf = vec![0u8; 2048];
    let (size, _) = listener.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..size], packet.raw_bytes.as_ref());
}

#[tokio::test]
async fn destination_breaker_recovers_after_cooldown_and_successful_probe() {
    let telemetry = InMemoryTelemetry::default();
    let worker = DestinationWorkerHandle::spawn(
        "flaky",
        DestinationPolicy {
            breaker: rosc_runtime::BreakerPolicy {
                open_after_consecutive_failures: 1,
                open_after_consecutive_queue_overflows: 3,
                cooldown: std::time::Duration::from_millis(30),
            },
            ..DestinationPolicy::default()
        },
        Arc::new(FlakySink {
            remaining_failures: Mutex::new(1),
            sent: Mutex::new(Vec::new()),
        }),
        Arc::new(telemetry),
    );

    worker
        .enqueue(sample_packet("/ue5/camera/fov"))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(worker.status().breaker_state, BreakerState::Open);

    tokio::time::sleep(std::time::Duration::from_millis(35)).await;
    worker
        .enqueue(sample_packet("/ue5/camera/fov"))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(35)).await;

    let status = worker.status();
    assert_eq!(status.breaker_state, BreakerState::Closed);
    assert_eq!(status.sent_total, 1);
}

#[tokio::test]
async fn drop_newest_is_not_counted_as_successful_dispatch() {
    let config = BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "saturated"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

        [[routes]]
        id = "fov"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "saturated"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let routing = rosc_route::RoutingEngine::new(config.routes).unwrap();
    let telemetry = InMemoryTelemetry::default();
    let runtime = Runtime {
        routing,
        telemetry: telemetry.clone(),
    };

    let mut destinations = DestinationRegistry::default();
    destinations.register(DestinationWorkerHandle::spawn(
        "saturated",
        DestinationPolicy {
            queue_depth: 0,
            drop_policy: DropPolicy::DropNewest,
            ..DestinationPolicy::default()
        },
        Arc::new(RecordingSink::default()),
        Arc::new(telemetry),
    ));

    let outcome = runtime
        .dispatch_packet(&sample_packet("/ue5/camera/fov"), &destinations)
        .await;

    assert_eq!(outcome.dispatched, 0);
    assert!(outcome.successful_dispatches.is_empty());
    assert_eq!(outcome.failures.len(), 1);
    assert_eq!(outcome.failures[0].reason, "destination_queue_drop_newest");
}

fn sample_packet(address: &str) -> PacketEnvelope {
    PacketEnvelope::parse_osc(
        encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: address.to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(90.0)],
        }))
        .unwrap(),
        IngressMetadata {
            ingress_id: "udp_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: None,
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .unwrap()
}

#[derive(Default)]
struct RecordingSink {
    sent: Mutex<Vec<String>>,
}

#[async_trait]
impl EgressSink for RecordingSink {
    async fn send(&self, packet: &PacketEnvelope) -> Result<(), DestinationSendError> {
        self.sent
            .lock()
            .unwrap()
            .push(packet.address().unwrap_or("<bundle>").to_owned());
        Ok(())
    }
}

struct FailingSink;

#[async_trait]
impl EgressSink for FailingSink {
    async fn send(&self, _packet: &PacketEnvelope) -> Result<(), DestinationSendError> {
        Err(DestinationSendError::Custom("simulated failure".to_owned()))
    }
}

struct FlakySink {
    remaining_failures: Mutex<usize>,
    sent: Mutex<Vec<String>>,
}

#[async_trait]
impl EgressSink for FlakySink {
    async fn send(&self, packet: &PacketEnvelope) -> Result<(), DestinationSendError> {
        let mut remaining = self.remaining_failures.lock().unwrap();
        if *remaining > 0 {
            *remaining -= 1;
            return Err(DestinationSendError::Custom("temporary failure".to_owned()));
        }
        self.sent
            .lock()
            .unwrap()
            .push(packet.address().unwrap_or("<bundle>").to_owned());
        Ok(())
    }
}
