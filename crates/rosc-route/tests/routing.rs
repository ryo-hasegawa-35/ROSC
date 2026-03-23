use std::time::SystemTime;

use rosc_osc::{
    CompatibilityMode, OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet,
};
use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
use rosc_route::{
    DestinationRef, RouteMatchSpec, RouteSpec, RoutingEngine, TrafficClass, TransformSpec,
    TransportSelector,
};

#[test]
fn routing_engine_matches_and_renames_exact_routes() {
    let route = RouteSpec {
        id: "camera_fov".to_owned(),
        enabled: true,
        mode: CompatibilityMode::Osc1_0Strict,
        class: TrafficClass::StatefulControl,
        match_spec: RouteMatchSpec {
            ingress_ids: vec!["udp_localhost_in".to_owned()],
            source_endpoints: vec![],
            address_patterns: vec!["/ue5/camera/fov".to_owned()],
            protocols: vec![TransportSelector::OscUdp],
        },
        transform: TransformSpec {
            rename_address: Some("/render/camera/fov".to_owned()),
        },
        destinations: vec![DestinationRef {
            target: "udp_renderer".to_owned(),
            transport: TransportSelector::OscUdp,
            enabled: true,
        }],
    };
    let engine = RoutingEngine::new(vec![route]).expect("route should compile");

    let source = PacketEnvelope::parse_osc(
        encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: "/ue5/camera/fov".to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(90.0)],
        }))
        .unwrap(),
        IngressMetadata {
            ingress_id: "udp_localhost_in".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: Some("127.0.0.1:7000".to_owned()),
            compatibility_mode: CompatibilityMode::Osc1_0Strict,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .expect("packet should parse");

    let dispatches = engine.route(&source).expect("routing should succeed");
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].destination.target, "udp_renderer");
    assert_eq!(dispatches[0].packet.address(), Some("/render/camera/fov"));
}

#[test]
fn traversal_wildcard_requires_extended_mode() {
    let route = RouteSpec {
        id: "bad".to_owned(),
        enabled: true,
        mode: CompatibilityMode::Osc1_0Strict,
        class: TrafficClass::SensorStream,
        match_spec: RouteMatchSpec {
            ingress_ids: vec![],
            source_endpoints: vec![],
            address_patterns: vec!["//tracking".to_owned()],
            protocols: vec![],
        },
        transform: TransformSpec::default(),
        destinations: vec![DestinationRef {
            target: "tap".to_owned(),
            transport: TransportSelector::Internal,
            enabled: true,
        }],
    };

    assert!(RoutingEngine::new(vec![route]).is_err());
}

#[test]
fn traversal_wildcard_matches_in_extended_mode() {
    let route = RouteSpec {
        id: "tracking".to_owned(),
        enabled: true,
        mode: CompatibilityMode::Osc1_1Extended,
        class: TrafficClass::SensorStream,
        match_spec: RouteMatchSpec {
            ingress_ids: vec![],
            source_endpoints: vec![],
            address_patterns: vec!["//tracking".to_owned()],
            protocols: vec![TransportSelector::OscUdp],
        },
        transform: TransformSpec::default(),
        destinations: vec![DestinationRef {
            target: "tap".to_owned(),
            transport: TransportSelector::Internal,
            enabled: true,
        }],
    };
    let engine = RoutingEngine::new(vec![route]).expect("extended route should compile");

    let source = PacketEnvelope::parse_osc(
        encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: "/td/rig/tracking".to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::True],
        }))
        .unwrap(),
        IngressMetadata {
            ingress_id: "udp_ext".to_owned(),
            transport: TransportKind::OscUdp,
            source_endpoint: None,
            compatibility_mode: CompatibilityMode::Osc1_1Extended,
            received_at: SystemTime::UNIX_EPOCH,
        },
    )
    .expect("packet should parse");

    let dispatches = engine.route(&source).expect("routing should succeed");
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].packet.address(), Some("/td/rig/tracking"));
}
