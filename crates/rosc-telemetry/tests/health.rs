use rosc_telemetry::{
    BreakerStateSnapshot, BrokerEvent, InMemoryTelemetry, RecentConfigEventKind, TelemetrySink,
};

#[test]
fn in_memory_telemetry_renders_prometheus_text() {
    let telemetry = InMemoryTelemetry::default();
    telemetry.emit(BrokerEvent::PacketAccepted {
        ingress_id: "udp_localhost_in".to_owned(),
    });
    telemetry.emit(BrokerEvent::PacketDropped {
        ingress_id: "udp_localhost_in".to_owned(),
        reason: "queue_full".to_owned(),
    });
    telemetry.emit(BrokerEvent::DispatchFailed {
        route_id: "camera_fov".to_owned(),
        destination_id: "udp_renderer".to_owned(),
        reason: "breaker_open".to_owned(),
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
    telemetry.emit(BrokerEvent::DestinationDropped {
        destination_id: "udp_renderer".to_owned(),
        reason: "queue_overflow".to_owned(),
    });
    telemetry.emit(BrokerEvent::DestinationBreakerChanged {
        destination_id: "udp_renderer".to_owned(),
        state: BreakerStateSnapshot::HalfOpen,
        reason: "cooldown_elapsed".to_owned(),
    });
    telemetry.emit(BrokerEvent::RouteIsolationChanged {
        route_id: "camera_fov".to_owned(),
        isolated: true,
    });
    telemetry.emit(BrokerEvent::OperatorAction {
        action: "freeze_traffic".to_owned(),
    });
    telemetry.emit(BrokerEvent::TrafficFreezeChanged { frozen: true });
    telemetry.emit(BrokerEvent::ConfigApplied {
        revision: 4,
        added_ingresses: 1,
        removed_ingresses: 0,
        changed_ingresses: 2,
        added_destinations: 3,
        removed_destinations: 0,
        changed_destinations: 4,
        added_routes: 1,
        removed_routes: 0,
        changed_routes: 2,
    });
    telemetry.emit(BrokerEvent::ConfigRejected);
    telemetry.emit(BrokerEvent::ConfigBlocked);
    telemetry.emit(BrokerEvent::ConfigReloadFailed);
    telemetry.emit(BrokerEvent::LaunchProfileChanged {
        mode: "safe_mode".to_owned(),
        disabled_capture_routes: 1,
        disabled_replay_routes: 2,
        disabled_restart_rehydrate_routes: 3,
    });

    let text = telemetry.render_prometheus();
    assert!(text.contains("rosc_ingress_packets_total{ingress_id=\"udp_localhost_in\"} 1"));
    assert!(text.contains(
        "rosc_ingress_drops_total{ingress_id=\"udp_localhost_in\",reason=\"queue_full\"} 1"
    ));
    assert!(text.contains(
        "rosc_dispatch_failures_total{route_id=\"camera_fov\",destination_id=\"udp_renderer\",reason=\"breaker_open\"} 1"
    ));
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
    assert!(text.contains(
        "rosc_destination_drops_total{destination_id=\"udp_renderer\",reason=\"queue_overflow\"} 1"
    ));
    assert!(text.contains("rosc_destination_breaker_state{destination_id=\"udp_renderer\"} 2"));
    assert!(text.contains("rosc_route_isolated{route_id=\"camera_fov\"} 1"));
    assert!(text.contains("rosc_operator_actions_total{action=\"freeze_traffic\"} 1"));
    assert!(text.contains("rosc_traffic_frozen 1"));
    assert!(text.contains("rosc_config_revision 4"));
    assert!(text.contains("rosc_config_added_ingresses_total 1"));
    assert!(text.contains("rosc_config_changed_ingresses_total 2"));
    assert!(text.contains("rosc_config_added_destinations_total 3"));
    assert!(text.contains("rosc_config_changed_destinations_total 4"));
    assert!(text.contains("rosc_config_rejections_total 1"));
    assert!(text.contains("rosc_config_blocked_total 1"));
    assert!(text.contains("rosc_config_reload_failures_total 1"));
    assert!(text.contains("rosc_launch_profile_mode{mode=\"safe_mode\"} 1"));
    assert!(text.contains("rosc_launch_profile_disabled_capture_routes 1"));
    assert!(text.contains("rosc_launch_profile_disabled_replay_routes 2"));
    assert!(text.contains("rosc_launch_profile_disabled_restart_rehydrate_routes 3"));

    let snapshot = telemetry.snapshot();
    assert_eq!(snapshot.recent_operator_actions.len(), 1);
    assert_eq!(snapshot.recent_operator_actions[0].sequence, 1);
    assert_eq!(snapshot.recent_operator_actions[0].action, "freeze_traffic");
    assert_eq!(snapshot.recent_config_events.len(), 5);
    assert_eq!(
        snapshot.recent_config_events[0].kind,
        RecentConfigEventKind::Applied
    );
    assert_eq!(snapshot.recent_config_events[0].revision, Some(4));
    assert_eq!(snapshot.recent_config_events[0].changed_routes, 2);
    assert_eq!(
        snapshot.recent_config_events[4].kind,
        RecentConfigEventKind::LaunchProfileChanged
    );
    assert_eq!(
        snapshot.recent_config_events[4]
            .launch_profile_mode
            .as_deref(),
        Some("safe_mode")
    );
}

#[test]
fn in_memory_telemetry_keeps_recent_history_bounded() {
    let telemetry = InMemoryTelemetry::default();
    for index in 0..40 {
        telemetry.emit(BrokerEvent::OperatorAction {
            action: format!("action_{index}"),
        });
    }

    let snapshot = telemetry.snapshot();
    assert_eq!(snapshot.recent_operator_actions.len(), 32);
    assert_eq!(snapshot.recent_operator_actions[0].action, "action_8");
    assert_eq!(snapshot.recent_operator_actions[31].action, "action_39");
    assert_eq!(snapshot.recent_operator_actions[0].sequence, 9);
    assert_eq!(snapshot.recent_operator_actions[31].sequence, 40);
}
