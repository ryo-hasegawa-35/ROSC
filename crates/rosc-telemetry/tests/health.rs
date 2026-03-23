use rosc_telemetry::{BreakerStateSnapshot, BrokerEvent, InMemoryTelemetry, TelemetrySink};

#[test]
fn in_memory_telemetry_renders_prometheus_text() {
    let telemetry = InMemoryTelemetry::default();
    telemetry.emit(BrokerEvent::PacketAccepted {
        ingress_id: "udp_localhost_in".to_owned(),
    });
    telemetry.emit(BrokerEvent::RouteMatched {
        route_id: "camera_fov".to_owned(),
    });
    telemetry.emit(BrokerEvent::RouteTransformFailed {
        route_id: "camera_fov".to_owned(),
    });
    telemetry.emit(BrokerEvent::CacheStored {
        route_id: "camera_fov".to_owned(),
    });
    telemetry.emit(BrokerEvent::CacheEntriesChanged {
        route_id: "camera_fov".to_owned(),
        entries: 2,
    });
    telemetry.emit(BrokerEvent::CaptureStored {
        route_id: "camera_fov".to_owned(),
    });
    telemetry.emit(BrokerEvent::CaptureEntriesChanged {
        route_id: "camera_fov".to_owned(),
        entries: 4,
    });
    telemetry.emit(BrokerEvent::RecoveryRehydrate {
        route_id: "camera_fov".to_owned(),
        destination_id: "udp_renderer".to_owned(),
        count: 2,
    });
    telemetry.emit(BrokerEvent::RecoveryReplay {
        route_id: "camera_fov".to_owned(),
        destination_id: "sandbox_tap".to_owned(),
        count: 1,
    });
    telemetry.emit(BrokerEvent::QueueDepthChanged {
        queue_id: "udp_renderer".to_owned(),
        depth: 3,
    });
    telemetry.emit(BrokerEvent::DestinationSent {
        destination_id: "udp_renderer".to_owned(),
    });
    telemetry.emit(BrokerEvent::DestinationBreakerChanged {
        destination_id: "udp_renderer".to_owned(),
        state: BreakerStateSnapshot::HalfOpen,
        reason: "cooldown_elapsed".to_owned(),
    });
    telemetry.emit(BrokerEvent::ConfigApplied {
        revision: 4,
        added_routes: 1,
        removed_routes: 0,
        changed_routes: 2,
    });
    telemetry.emit(BrokerEvent::ConfigRejected);

    let text = telemetry.render_prometheus();
    assert!(text.contains("rosc_ingress_packets_total{ingress_id=\"udp_localhost_in\"} 1"));
    assert!(text.contains("rosc_route_matches_total{route_id=\"camera_fov\"} 1"));
    assert!(text.contains("rosc_route_transform_failures_total{route_id=\"camera_fov\"} 1"));
    assert!(text.contains("rosc_cache_entries{route_id=\"camera_fov\"} 2"));
    assert!(text.contains("rosc_cache_writes_total{route_id=\"camera_fov\"} 1"));
    assert!(text.contains("rosc_capture_entries{route_id=\"camera_fov\"} 4"));
    assert!(text.contains("rosc_capture_writes_total{route_id=\"camera_fov\"} 1"));
    assert!(text.contains(
        "rosc_recovery_rehydrate_total{route_id=\"camera_fov\",destination_id=\"udp_renderer\"} 2"
    ));
    assert!(text.contains(
        "rosc_recovery_replay_total{route_id=\"camera_fov\",destination_id=\"sandbox_tap\"} 1"
    ));
    assert!(text.contains("rosc_queue_depth{queue_id=\"udp_renderer\"} 3"));
    assert!(text.contains("rosc_destination_send_total{destination_id=\"udp_renderer\"} 1"));
    assert!(text.contains("rosc_destination_breaker_state{destination_id=\"udp_renderer\"} 2"));
    assert!(text.contains("rosc_config_revision 4"));
    assert!(text.contains("rosc_config_rejections_total 1"));
}
