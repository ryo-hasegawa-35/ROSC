use std::time::Duration;

use rosc_osc::{
    CompatibilityMode, OscArgument, OscBundle, OscMessage, ParsedOscPacket, TypeTagSource,
};
use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
use rosc_recovery::{RecoveryAction, RecoveryEngine, RehydrateRequest, SandboxReplayRequest};
use rosc_route::{
    CachePolicy, CapturePolicy, DestinationRef, LateJoinerPolicy, PersistPolicy, RouteCacheSpec,
    RouteDispatch, RouteObservabilitySpec, RouteRecoverySpec, TransformSpec, TransportSelector,
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
            replay_allowed: true,
        },
        observability: RouteObservabilitySpec {
            capture: CapturePolicy::AlwaysBounded,
        },
    }
}

fn sample_bundle_dispatch(route_id: &str, destination_id: &str) -> RouteDispatch {
    let packet = PacketEnvelope::parse_osc(
        rosc_osc::encode_packet(&ParsedOscPacket::Bundle(OscBundle {
            timetag: 1,
            elements: vec![ParsedOscPacket::Message(OscMessage {
                address: "/render/camera/fov".to_owned(),
                type_tag_source: TypeTagSource::Explicit,
                arguments: vec![OscArgument::Float32(0.5)],
            })],
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
            replay_allowed: true,
        },
        observability: RouteObservabilitySpec {
            capture: CapturePolicy::AlwaysBounded,
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
    assert!(
        outcome
            .dispatches
            .iter()
            .all(|dispatch| dispatch.packet.metadata.ingress_id == "rehydrate:camera")
    );
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
    assert!(metrics.contains("rosc_cache_entries{route_id=\"camera\"} 0"));
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

#[test]
fn recovery_engine_replays_to_a_sandbox_destination_and_records_audit() {
    let telemetry = InMemoryTelemetry::default();
    let engine = RecoveryEngine::with_limits(telemetry.clone(), 8, 8);
    engine.observe_dispatches(&[
        sample_dispatch("camera", "udp_renderer", "/render/camera/fov"),
        sample_dispatch("camera", "udp_renderer", "/render/camera/zoom"),
    ]);

    let outcome = engine
        .sandbox_replay(SandboxReplayRequest {
            route_id: "camera".to_owned(),
            source_destination_id: Some("udp_renderer".to_owned()),
            sandbox_destination_id: "sandbox_tap".to_owned(),
            limit: 10,
        })
        .unwrap();

    assert_eq!(outcome.dispatches.len(), 2);
    assert!(outcome.dispatches.iter().all(|dispatch| {
        dispatch.destination.destination_id() == "sandbox_tap"
            && dispatch.packet.metadata.ingress_id == "replay:camera"
            && dispatch.packet.metadata.transport == TransportKind::Internal
    }));

    let audit = engine.audit_records();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].action, RecoveryAction::SandboxReplay);
    assert_eq!(audit[0].target_destination_id, "sandbox_tap");

    let metrics = telemetry.render_prometheus();
    assert!(metrics.contains("rosc_capture_entries{route_id=\"camera\"} 2"));
    assert!(metrics.contains("rosc_capture_writes_total{route_id=\"camera\"} 2"));
    assert!(metrics.contains(
        "rosc_recovery_replay_total{route_id=\"camera\",destination_id=\"sandbox_tap\"} 2"
    ));
}

#[test]
fn recovery_engine_skips_routes_that_do_not_allow_replay() {
    let telemetry = InMemoryTelemetry::default();
    let engine = RecoveryEngine::with_limits(telemetry.clone(), 8, 8);
    let mut dispatch = sample_dispatch("camera", "udp_renderer", "/render/camera/fov");
    dispatch.recovery.replay_allowed = false;
    engine.observe_dispatches(&[dispatch]);

    let outcome = engine
        .sandbox_replay(SandboxReplayRequest {
            route_id: "camera".to_owned(),
            source_destination_id: Some("udp_renderer".to_owned()),
            sandbox_destination_id: "sandbox_tap".to_owned(),
            limit: 10,
        })
        .unwrap();

    assert!(outcome.dispatches.is_empty());
    assert!(engine.audit_records().is_empty());
    let metrics = telemetry.render_prometheus();
    assert!(!metrics.contains("rosc_recovery_replay_total"));
}

#[test]
fn recovery_engine_still_captures_addressless_packets_when_cache_is_enabled() {
    let telemetry = InMemoryTelemetry::default();
    let engine = RecoveryEngine::with_limits(telemetry.clone(), 8, 8);
    engine.observe_dispatches(&[sample_bundle_dispatch("camera", "udp_renderer")]);

    let outcome = engine
        .sandbox_replay(SandboxReplayRequest {
            route_id: "camera".to_owned(),
            source_destination_id: Some("udp_renderer".to_owned()),
            sandbox_destination_id: "sandbox_tap".to_owned(),
            limit: 10,
        })
        .unwrap();

    assert_eq!(outcome.dispatches.len(), 1);

    let metrics = telemetry.render_prometheus();
    assert!(metrics.contains("rosc_capture_entries{route_id=\"camera\"} 1"));
    assert!(metrics.contains("rosc_capture_writes_total{route_id=\"camera\"} 1"));
    assert!(!metrics.contains("rosc_cache_entries{route_id=\"camera\"} 1"));
    assert!(!metrics.contains("rosc_cache_writes_total{route_id=\"camera\"} 1"));
}
