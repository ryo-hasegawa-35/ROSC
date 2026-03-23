use std::time::Duration;

use rosc_osc::{CompatibilityMode, OscArgument, OscMessage, ParsedOscPacket, TypeTagSource};
use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
use rosc_recovery::{RecoveryEngine, RehydrateRequest};
use rosc_route::{
    CachePolicy, DestinationRef, LateJoinerPolicy, PersistPolicy, RouteCacheSpec, RouteDispatch,
    RouteRecoverySpec, TransformSpec, TransportSelector,
};
use rosc_telemetry::InMemoryTelemetry;

fn sample_dispatch(route_id: &str, destination_id: &str, address: &str) -> RouteDispatch {
    let packet = PacketEnvelope::parse_osc(
        rosc_osc::encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: address.to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(0.5)],
        }))
        .unwrap(),
        IngressMetadata {
            ingress_id: "udp_localhost_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: Some("127.0.0.1:9000".to_owned()),
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: std::time::SystemTime::now(),
        },
    )
    .unwrap();

    RouteDispatch {
        route_id: route_id.to_owned(),
        destination: DestinationRef {
            target: destination_id.to_owned(),
            transport: TransportSelector::OscUdp,
            enabled: true,
        },
        packet,
        transform: TransformSpec::default(),
        cache: RouteCacheSpec {
            policy: CachePolicy::LastValuePerAddress,
            ttl_ms: Some(10),
            persist: PersistPolicy::Warm,
        },
        recovery: RouteRecoverySpec {
            late_joiner: LateJoinerPolicy::Latest,
            rehydrate_on_connect: true,
            rehydrate_on_restart: false,
            replay_allowed: false,
        },
    }
}

#[test]
fn recovery_engine_rehydrates_latest_value_per_address() {
    let telemetry = InMemoryTelemetry::default();
    let engine = RecoveryEngine::new(telemetry);
    engine.observe_dispatches(&[
        sample_dispatch("camera", "udp_renderer", "/render/camera/fov"),
        sample_dispatch("camera", "udp_renderer", "/render/camera/fov"),
        sample_dispatch("camera", "udp_renderer", "/render/camera/zoom"),
    ]);

    let outcome = engine
        .rehydrate(RehydrateRequest {
            route_id: Some("camera".to_owned()),
            destination_id: Some("udp_renderer".to_owned()),
        })
        .unwrap();

    assert_eq!(outcome.stale_evictions, 0);
    assert_eq!(outcome.dispatches.len(), 2);
}

#[test]
fn recovery_engine_evicts_stale_entries_before_rehydrate() {
    let telemetry = InMemoryTelemetry::default();
    let engine = RecoveryEngine::new(telemetry.clone());
    engine.observe_dispatches(&[sample_dispatch(
        "camera",
        "udp_renderer",
        "/render/camera/fov",
    )]);

    std::thread::sleep(Duration::from_millis(20));

    let outcome = engine
        .rehydrate(RehydrateRequest {
            route_id: Some("camera".to_owned()),
            destination_id: Some("udp_renderer".to_owned()),
        })
        .unwrap();

    assert_eq!(outcome.dispatches.len(), 0);
    assert_eq!(outcome.stale_evictions, 1);
    let metrics = telemetry.render_prometheus();
    assert!(metrics.contains("rosc_cache_evictions_total{route_id=\"camera\",reason=\"stale\"} 1"));
}

#[test]
fn recovery_engine_requires_a_selector() {
    let engine = RecoveryEngine::new(InMemoryTelemetry::default());
    let error = engine.rehydrate(RehydrateRequest::default()).unwrap_err();
    assert_eq!(
        error.to_string(),
        "rehydrate requests must specify at least one selector"
    );
}
