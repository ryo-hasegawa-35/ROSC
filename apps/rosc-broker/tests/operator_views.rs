mod common;

use common::broad_scope_config;
use rosc_broker::{
    ProxyOperatorSignalScope, ProxyOperatorState, ProxyOperatorTimelineCategory,
    ProxyRuntimeSafetyPolicy, attach_runtime_status, proxy_operator_attention,
    proxy_operator_board, proxy_operator_casebook, proxy_operator_dashboard,
    proxy_operator_diagnostics, proxy_operator_focus_from_dashboard, proxy_operator_handoff,
    proxy_operator_incidents_from_histories, proxy_operator_overview, proxy_operator_readiness,
    proxy_operator_recovery, proxy_operator_report, proxy_operator_signals_view,
    proxy_operator_snapshot, proxy_operator_timeline, proxy_operator_trace,
    proxy_status_from_config,
};
use rosc_telemetry::{
    HealthSnapshot, RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction,
};

#[test]
fn operator_report_surfaces_runtime_failure_signals() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            ingress_drops_total: [(("udp_localhost_in".to_owned(), "queue_full".to_owned()), 2)]
                .into_iter()
                .collect(),
            dispatch_failures_total: [(
                (
                    "camera".to_owned(),
                    "udp_renderer".to_owned(),
                    "breaker_open".to_owned(),
                ),
                3,
            )]
            .into_iter()
            .collect(),
            route_transform_failures_total: [("camera".to_owned(), 1)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            destination_send_failures_total: [(
                ("udp_renderer".to_owned(), "socket_error".to_owned()),
                5,
            )]
            .into_iter()
            .collect(),
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::Open,
            )]
            .into_iter()
            .collect(),
            ..HealthSnapshot::default()
        },
    );

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(
        report.runtime_signals.ingresses_with_drops,
        vec!["udp_localhost_in"]
    );
    assert_eq!(
        report.runtime_signals.routes_with_dispatch_failures,
        vec!["camera"]
    );
    assert_eq!(
        report.runtime_signals.routes_with_transform_failures,
        vec!["camera"]
    );
    assert_eq!(
        report.runtime_signals.destinations_with_drops,
        vec!["udp_renderer"]
    );
    assert_eq!(
        report.runtime_signals.destinations_with_send_failures,
        vec!["udp_renderer"]
    );
    assert_eq!(
        report.runtime_signals.destinations_with_open_breakers,
        vec!["udp_renderer"]
    );
    assert!(report.destination_signals.iter().any(|destination| {
        destination.destination_id == "udp_renderer"
            && destination.drops_total == 4
            && destination.send_failures_total == 5
            && destination.breaker_state == Some(rosc_telemetry::BreakerStateSnapshot::Open)
    }));
    assert!(report.route_signals.iter().any(|route| {
        route.route_id == "camera"
            && route.dispatch_failures_total == 3
            && route.transform_failures_total == 1
    }));
}

#[test]
fn operator_signals_view_can_filter_to_problematic_entries() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::HalfOpen,
            )]
            .into_iter()
            .collect(),
            ..HealthSnapshot::default()
        },
    );

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());
    let filtered = proxy_operator_signals_view(&report, ProxyOperatorSignalScope::Problematic);

    assert_eq!(filtered.scope, ProxyOperatorSignalScope::Problematic);
    assert!(
        filtered
            .route_signals
            .iter()
            .all(|signal| signal.is_problematic())
    );
    assert!(
        filtered
            .route_signals
            .iter()
            .any(|signal| signal.route_id == "camera" && signal.isolated)
    );
    assert!(
        filtered
            .destination_signals
            .iter()
            .all(|signal| signal.is_problematic())
    );
    assert!(
        filtered
            .destination_signals
            .iter()
            .any(|signal| signal.destination_id == "udp_renderer" && signal.drops_total == 4)
    );
}

#[test]
fn operator_overview_embeds_problematic_signals() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            ..HealthSnapshot::default()
        },
    );

    let overview = proxy_operator_overview(&status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(overview.status, status);
    assert_eq!(overview.report.state, ProxyOperatorState::Warning);
    assert_eq!(
        overview.problematic_signals.scope,
        ProxyOperatorSignalScope::Problematic
    );
    assert!(
        overview
            .problematic_signals
            .route_signals
            .iter()
            .any(|signal| signal.route_id == "camera" && signal.isolated)
    );
    assert!(overview.runtime_summary.has_runtime_status);
    assert!(overview.runtime_summary.traffic_frozen);
    assert_eq!(overview.runtime_summary.isolated_route_count, 1);
}

#[test]
fn operator_readiness_distinguishes_ready_and_degraded_states() {
    let ready_config = rosc_config::BrokerConfig::from_toml_str(
        r#"
        [[udp_ingresses]]
        id = "udp_localhost_in"
        bind = "127.0.0.1:0"
        mode = "osc1_0_strict"

        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "127.0.0.1:9001"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"

        [routes.match]
        ingress_ids = ["udp_localhost_in"]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]

        [routes.fallback]
        direct_udp_target = "127.0.0.1:9002"

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .expect("config should parse");
    let ready_status = proxy_status_from_config(&ready_config).expect("status should build");
    let ready = proxy_operator_readiness(&ready_status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(ready.level, rosc_broker::ProxyOperatorReadinessLevel::Ready);
    assert!(ready.ready);
    assert!(ready.flags.control_plane_ready);
    assert!(ready.flags.traffic_flow_ready);
    assert!(ready.flags.fallback_complete);
    assert_eq!(
        ready.reasons,
        vec!["no active readiness blockers or warnings"]
    );

    let degraded_config = broad_scope_config();
    let degraded_status = attach_runtime_status(
        proxy_status_from_config(&degraded_config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            ..HealthSnapshot::default()
        },
    );
    let degraded = proxy_operator_readiness(&degraded_status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(
        degraded.level,
        rosc_broker::ProxyOperatorReadinessLevel::Degraded
    );
    assert!(!degraded.ready);
    assert!(degraded.flags.control_plane_ready);
    assert!(!degraded.flags.traffic_flow_ready);
    assert!(degraded.flags.operator_intervention_required);
    assert!(
        degraded
            .reasons
            .iter()
            .any(|reason| reason.contains("traffic is currently frozen"))
    );
    assert!(
        degraded
            .reasons
            .iter()
            .any(|reason| reason.contains("currently isolated"))
    );
    assert!(degraded.counts.problematic_destinations > 0);
    assert!(degraded.counts.problematic_routes > 0);
}

#[test]
fn operator_snapshot_bundles_readiness_diagnostics_attention_and_incidents() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 9,
                recorded_at_unix_ms: 900,
                action: "freeze_traffic".to_owned(),
                details: vec!["applied=true".to_owned()],
            }],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 10,
                recorded_at_unix_ms: 1000,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(4),
                details: vec!["unsafe wildcard route".to_owned()],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 0,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: None,
                disabled_capture_routes: 0,
                disabled_replay_routes: 0,
                disabled_restart_rehydrate_routes: 0,
            }],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(1));

    assert_eq!(
        snapshot.readiness.level,
        rosc_broker::ProxyOperatorReadinessLevel::Degraded
    );
    assert_eq!(snapshot.diagnostics.recent_operator_actions.len(), 1);
    assert_eq!(snapshot.attention.state, ProxyOperatorState::Warning);
    assert_eq!(snapshot.incidents.recent_config_issues.len(), 1);
    assert_eq!(snapshot.overview.report.state, ProxyOperatorState::Warning);
    assert_eq!(snapshot.incident_digest.state, "warning");
    assert!(
        snapshot
            .incident_digest
            .clusters
            .iter()
            .any(|cluster| cluster.id == "traffic-frozen")
    );
    assert_eq!(snapshot.recovery.cached_routes, 0);
    assert!(
        snapshot
            .worklist
            .items
            .iter()
            .any(|item| item.id == "traffic-frozen")
    );
    assert!(
        snapshot
            .worklist
            .items
            .iter()
            .any(|item| item.id == "route:camera:restore")
    );
    assert!(
        snapshot
            .triage
            .global
            .next_steps
            .iter()
            .any(|step| step.contains("Thaw traffic"))
    );
    assert!(
        snapshot
            .casebook
            .route_casebooks
            .iter()
            .any(|casebook| casebook.route_id == "camera")
    );
    assert!(
        snapshot
            .board
            .degraded_items
            .iter()
            .any(|item| item.route_id.as_deref() == Some("camera"))
    );
}

#[test]
fn operator_diagnostics_bounds_recent_history_without_changing_overview() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            recent_operator_actions: vec![
                RecentOperatorAction {
                    sequence: 1,
                    recorded_at_unix_ms: 100,
                    action: "freeze_traffic".to_owned(),
                    details: vec!["applied=true".to_owned()],
                },
                RecentOperatorAction {
                    sequence: 2,
                    recorded_at_unix_ms: 200,
                    action: "thaw_traffic".to_owned(),
                    details: vec!["applied=true".to_owned()],
                },
            ],
            recent_config_events: vec![
                RecentConfigEvent {
                    sequence: 3,
                    recorded_at_unix_ms: 300,
                    kind: RecentConfigEventKind::Applied,
                    revision: Some(1),
                    details: Vec::new(),
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 0,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 0,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
                RecentConfigEvent {
                    sequence: 4,
                    recorded_at_unix_ms: 400,
                    kind: RecentConfigEventKind::Blocked,
                    revision: Some(2),
                    details: vec!["unsafe wildcard route".to_owned()],
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 0,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 1,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
            ],
            ..HealthSnapshot::default()
        },
    );

    let diagnostics =
        proxy_operator_diagnostics(&status, ProxyRuntimeSafetyPolicy::default(), Some(1));

    assert_eq!(diagnostics.overview.status, status);
    assert_eq!(diagnostics.recent_operator_actions.len(), 1);
    assert_eq!(
        diagnostics.recent_operator_actions[0].action,
        "thaw_traffic"
    );
    assert_eq!(diagnostics.recent_config_events.len(), 1);
    assert_eq!(
        diagnostics.recent_config_events[0].kind,
        RecentConfigEventKind::Blocked
    );
    assert_eq!(
        diagnostics
            .overview
            .runtime_summary
            .recent_operator_action_count,
        2
    );
    assert_eq!(
        diagnostics
            .overview
            .runtime_summary
            .recent_config_event_count,
        2
    );
}

#[test]
fn operator_dashboard_bundles_snapshot_traffic_and_timeline() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            ingress_packets_total: [("udp_localhost_in".to_owned(), 120)].into_iter().collect(),
            ingress_drops_total: [(("udp_localhost_in".to_owned(), "queue_full".to_owned()), 3)]
                .into_iter()
                .collect(),
            route_matches_total: [("camera".to_owned(), 90)].into_iter().collect(),
            dispatch_failures_total: [(
                (
                    "camera".to_owned(),
                    "udp_renderer".to_owned(),
                    "breaker_open".to_owned(),
                ),
                2,
            )]
            .into_iter()
            .collect(),
            route_transform_failures_total: [("camera".to_owned(), 1)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            destination_sent_total: [("udp_renderer".to_owned(), 75)].into_iter().collect(),
            destination_send_failures_total: [(
                ("udp_renderer".to_owned(), "socket_error".to_owned()),
                5,
            )]
            .into_iter()
            .collect(),
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::Open,
            )]
            .into_iter()
            .collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 10,
                recorded_at_unix_ms: 1_000,
                action: "freeze_traffic".to_owned(),
                details: vec!["applied=true".to_owned()],
            }],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 11,
                recorded_at_unix_ms: 1_100,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(4),
                details: vec!["unsafe wildcard route".to_owned()],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 0,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: Some("safe_mode".to_owned()),
                disabled_capture_routes: 1,
                disabled_replay_routes: 1,
                disabled_restart_rehydrate_routes: 1,
            }],
            ..HealthSnapshot::default()
        },
    );

    let dashboard = proxy_operator_dashboard(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));

    assert_eq!(dashboard.refresh_interval_ms, 2_500);
    assert_eq!(dashboard.snapshot.overview.status, status);
    assert!(dashboard.traffic.has_runtime_status);
    assert_eq!(dashboard.traffic.ingress_packets_total, 120);
    assert_eq!(dashboard.traffic.ingress_drops_total, 3);
    assert_eq!(dashboard.traffic.route_matches_total, 90);
    assert_eq!(dashboard.traffic.route_dispatch_failures_total, 2);
    assert_eq!(dashboard.traffic.route_transform_failures_total, 1);
    assert_eq!(dashboard.traffic.destination_send_total, 75);
    assert_eq!(dashboard.traffic.destination_send_failures_total, 5);
    assert_eq!(dashboard.traffic.destination_drops_total, 4);
    assert_eq!(
        dashboard.traffic.busiest_ingresses[0].id,
        "udp_localhost_in"
    );
    assert_eq!(dashboard.traffic.busiest_routes[0].id, "camera");
    assert_eq!(
        dashboard.traffic.noisiest_destinations[0].id,
        "udp_renderer"
    );
    assert_eq!(dashboard.focus.state, "warning");
    let camera_detail = dashboard
        .route_details
        .iter()
        .find(|detail| detail.route_id == "camera")
        .expect("camera detail should exist");
    assert_eq!(camera_detail.dispatch_failures_total, 2);
    assert_eq!(camera_detail.transform_failures_total, 1);
    assert_eq!(camera_detail.direct_udp_targets, vec!["127.0.0.1:9001"]);

    let renderer_detail = dashboard
        .destination_details
        .iter()
        .find(|detail| detail.destination_id == "udp_renderer")
        .expect("renderer detail should exist");
    assert_eq!(renderer_detail.send_failures_total, 5);
    assert_eq!(renderer_detail.drops_total, 4);
    assert!(
        dashboard
            .snapshot
            .incident_digest
            .clusters
            .iter()
            .any(|cluster| cluster.id == "route:camera")
    );
    assert!(
        dashboard
            .snapshot
            .incident_digest
            .clusters
            .iter()
            .any(|cluster| cluster.id == "destination:udp_renderer")
    );
    assert_eq!(dashboard.snapshot.recovery.cached_routes, 0);
    assert_eq!(dashboard.snapshot.recovery.rehydrate_ready_destinations, 1);
    assert_eq!(dashboard.timeline_catalog.global.len(), 2);
    assert!(
        dashboard
            .trace
            .routes
            .iter()
            .any(|trace| trace.route_id == "camera")
    );
    assert!(
        dashboard
            .trace
            .destinations
            .iter()
            .any(|trace| trace.destination_id == "udp_renderer")
    );
    assert!(
        dashboard
            .snapshot
            .handoff
            .destination_handoffs
            .iter()
            .any(|handoff| handoff.destination_id == "udp_renderer")
    );
    assert!(dashboard.snapshot.worklist.immediate_actions >= 1);
    assert!(
        dashboard
            .snapshot
            .worklist
            .items
            .iter()
            .any(|item| item.id == "destination:udp_renderer:rehydrate")
    );
    assert_eq!(dashboard.timeline.len(), 2);
    assert_eq!(
        dashboard.timeline[0].category,
        ProxyOperatorTimelineCategory::ConfigEvent
    );
    assert_eq!(dashboard.timeline[0].label, "config_blocked");
    assert_eq!(dashboard.timeline[0].recorded_at_unix_ms, 1_100);
    assert!(
        dashboard.timeline[0]
            .details
            .iter()
            .any(|detail| detail == "revision=4")
    );
    assert_eq!(
        dashboard.timeline[1].category,
        ProxyOperatorTimelineCategory::OperatorAction
    );
    assert_eq!(dashboard.timeline[1].label, "freeze_traffic");
    assert!(
        dashboard
            .timeline_catalog
            .routes
            .iter()
            .any(|timeline| timeline.route_id == "camera")
    );
    assert!(
        dashboard
            .timeline_catalog
            .destinations
            .iter()
            .any(|timeline| timeline.destination_id == "udp_renderer")
    );
    assert!(
        dashboard
            .focus
            .routes
            .iter()
            .any(|packet| packet.route_id == "camera"
                && packet.trace.is_some()
                && packet.timeline.is_some()
                && packet.handoff.is_some()
                && packet.triage.is_some()
                && packet.casebook.is_some())
    );
    assert!(
        dashboard
            .focus
            .destinations
            .iter()
            .any(|packet| packet.destination_id == "udp_renderer"
                && packet.trace.is_some()
                && packet.timeline.is_some()
                && packet.handoff.is_some()
                && packet.triage.is_some()
                && packet.casebook.is_some())
    );
}

#[test]
fn operator_focus_catalog_bundles_linked_route_and_destination_context() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            queue_depth: [("udp_renderer".to_owned(), 3)].into_iter().collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 21,
                recorded_at_unix_ms: 2_100,
                action: "isolate_route".to_owned(),
                details: vec!["route_id=camera".to_owned()],
            }],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 22,
                recorded_at_unix_ms: 2_200,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(8),
                details: vec![
                    "route_id=camera".to_owned(),
                    "destination_id=udp_renderer".to_owned(),
                ],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 1,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: None,
                disabled_capture_routes: 0,
                disabled_replay_routes: 0,
                disabled_restart_rehydrate_routes: 0,
            }],
            ..HealthSnapshot::default()
        },
    );

    let dashboard = proxy_operator_dashboard(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));
    let focus = proxy_operator_focus_from_dashboard(&dashboard);

    let route_packet = focus
        .routes
        .iter()
        .find(|packet| packet.route_id == "camera")
        .expect("route packet should exist");
    assert!(route_packet.trace.is_some());
    assert!(route_packet.timeline.is_some());
    assert!(route_packet.handoff.is_some());
    assert!(route_packet.triage.is_some());
    assert!(route_packet.casebook.is_some());
    assert!(
        route_packet
            .board_items
            .iter()
            .any(|item| item.route_id.as_deref() == Some("camera"))
    );

    let destination_packet = focus
        .destinations
        .iter()
        .find(|packet| packet.destination_id == "udp_renderer")
        .expect("destination packet should exist");
    assert!(destination_packet.trace.is_some());
    assert!(destination_packet.timeline.is_some());
    assert!(destination_packet.handoff.is_some());
    assert!(destination_packet.triage.is_some());
    assert!(destination_packet.casebook.is_some());
    assert!(
        destination_packet
            .board_items
            .iter()
            .any(|item| item.destination_id.as_deref() == Some("udp_renderer"))
    );
}

#[test]
fn operator_trace_links_runtime_actions_and_config_events_to_entities() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            dispatch_failures_total: [(
                (
                    "camera".to_owned(),
                    "udp_renderer".to_owned(),
                    "breaker_open".to_owned(),
                ),
                3,
            )]
            .into_iter()
            .collect(),
            destination_send_failures_total: [(
                ("udp_renderer".to_owned(), "socket_error".to_owned()),
                5,
            )]
            .into_iter()
            .collect(),
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::Open,
            )]
            .into_iter()
            .collect(),
            recent_operator_actions: vec![
                RecentOperatorAction {
                    sequence: 1,
                    recorded_at_unix_ms: 100,
                    action: "freeze_traffic".to_owned(),
                    details: vec!["applied=true".to_owned()],
                },
                RecentOperatorAction {
                    sequence: 2,
                    recorded_at_unix_ms: 200,
                    action: "rehydrate_destination".to_owned(),
                    details: vec![
                        "destination_id=udp_renderer".to_owned(),
                        "applied=true".to_owned(),
                    ],
                },
            ],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 3,
                recorded_at_unix_ms: 300,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(4),
                details: vec![
                    "route_id=camera".to_owned(),
                    "unsafe wildcard route".to_owned(),
                ],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 0,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: Some("safe_mode".to_owned()),
                disabled_capture_routes: 1,
                disabled_replay_routes: 1,
                disabled_restart_rehydrate_routes: 1,
            }],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));
    let trace = proxy_operator_trace(&snapshot);

    let route_trace = trace
        .routes
        .iter()
        .find(|route| route.route_id == "camera")
        .expect("route trace should exist");
    assert!(
        route_trace
            .open_reasons
            .iter()
            .any(|reason| reason.contains("operator isolation"))
    );
    assert!(route_trace.recent_events.iter().any(|event| event.kind
        == rosc_broker::ProxyOperatorTraceEventKind::OperatorAction
        && event.title == "freeze traffic"));
    assert!(route_trace.recent_events.iter().any(|event| event.kind
        == rosc_broker::ProxyOperatorTraceEventKind::ConfigEvent
        && event.details.iter().any(|detail| detail == "revision=4")));

    let destination_trace = trace
        .destinations
        .iter()
        .find(|destination| destination.destination_id == "udp_renderer")
        .expect("destination trace should exist");
    assert_eq!(
        destination_trace.level,
        rosc_broker::ProxyOperatorTraceEventLevel::Blocked
    );
    assert!(destination_trace.recent_events.iter().any(|event| {
        event.kind == rosc_broker::ProxyOperatorTraceEventKind::OperatorAction
            && event
                .details
                .iter()
                .any(|detail| detail == "destination_id=udp_renderer")
    }));
}

#[test]
fn operator_trace_and_timeline_only_link_explicit_config_events_to_matching_entities() {
    let config = rosc_config::BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "udp_renderer"
        bind = "127.0.0.1:0"
        target = "127.0.0.1:9001"

        [[udp_destinations]]
        id = "udp_lights"
        bind = "127.0.0.1:0"
        target = "127.0.0.1:9002"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"

        [[routes]]
        id = "lights"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/lights/intensity"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_lights"
        transport = "osc_udp"
        "#,
    )
    .expect("config should parse");
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            recent_config_events: vec![
                RecentConfigEvent {
                    sequence: 1,
                    recorded_at_unix_ms: 100,
                    kind: RecentConfigEventKind::Blocked,
                    revision: Some(4),
                    details: vec![
                        "route_id=camera".to_owned(),
                        "destination_id=udp_renderer".to_owned(),
                        "unsafe wildcard route".to_owned(),
                    ],
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 1,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 1,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
                RecentConfigEvent {
                    sequence: 2,
                    recorded_at_unix_ms: 200,
                    kind: RecentConfigEventKind::Rejected,
                    revision: Some(5),
                    details: vec![
                        "destination_id=udp_lights".to_owned(),
                        "queue policy mismatch".to_owned(),
                    ],
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 1,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 0,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
            ],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));
    let trace = proxy_operator_trace(&snapshot);
    let timeline = proxy_operator_timeline(&snapshot);

    let camera_trace = trace
        .routes
        .iter()
        .find(|route| route.route_id == "camera")
        .expect("camera trace should exist");
    assert!(camera_trace.recent_events.iter().any(|event| {
        event.kind == rosc_broker::ProxyOperatorTraceEventKind::ConfigEvent
            && event.details.iter().any(|detail| detail == "revision=4")
    }));

    let lights_trace = trace
        .routes
        .iter()
        .find(|route| route.route_id == "lights")
        .expect("lights trace should exist");
    assert!(!lights_trace.recent_events.iter().any(|event| {
        event.kind == rosc_broker::ProxyOperatorTraceEventKind::ConfigEvent
            && event.details.iter().any(|detail| detail == "revision=4")
    }));
    assert!(lights_trace.recent_events.iter().any(|event| {
        event.kind == rosc_broker::ProxyOperatorTraceEventKind::ConfigEvent
            && event.details.iter().any(|detail| detail == "revision=5")
    }));

    let renderer_timeline = timeline
        .destinations
        .iter()
        .find(|entry| entry.destination_id == "udp_renderer")
        .expect("renderer timeline should exist");
    assert!(renderer_timeline.entries.iter().any(|entry| {
        entry.category == ProxyOperatorTimelineCategory::ConfigEvent
            && entry.details.iter().any(|detail| detail == "revision=4")
    }));
    assert!(!renderer_timeline.entries.iter().any(|entry| {
        entry.category == ProxyOperatorTimelineCategory::ConfigEvent
            && entry.details.iter().any(|detail| detail == "revision=5")
    }));
}

#[test]
fn operator_handoff_derives_next_steps_from_trace_and_snapshot() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::HalfOpen,
            )]
            .into_iter()
            .collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 7,
                recorded_at_unix_ms: 700,
                action: "freeze_traffic".to_owned(),
                details: vec!["applied=true".to_owned()],
            }],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));
    let handoff = proxy_operator_handoff(&snapshot);

    let route_handoff = handoff
        .route_handoffs
        .iter()
        .find(|handoff| handoff.route_id == "camera")
        .expect("route handoff should exist");
    assert!(
        route_handoff
            .next_steps
            .iter()
            .any(|step| step.contains("Thaw traffic"))
    );

    let destination_handoff = handoff
        .destination_handoffs
        .iter()
        .find(|handoff| handoff.destination_id == "udp_renderer")
        .expect("destination handoff should exist");
    assert!(
        destination_handoff
            .next_steps
            .iter()
            .any(|step| step.contains("Thaw traffic"))
    );
}

#[test]
fn operator_triage_combines_global_actions_with_focused_history() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 11,
                recorded_at_unix_ms: 1_100,
                action: "freeze_traffic".to_owned(),
                details: vec!["applied=true".to_owned()],
            }],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 12,
                recorded_at_unix_ms: 1_200,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(7),
                details: vec!["route_id=camera".to_owned()],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 0,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: None,
                disabled_capture_routes: 0,
                disabled_replay_routes: 0,
                disabled_restart_rehydrate_routes: 0,
            }],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(8));
    let triage = snapshot.triage;

    assert!(
        triage
            .global
            .actions
            .iter()
            .any(|action| action.kind == rosc_broker::ProxyOperatorSuggestedActionKind::ThawTraffic)
    );
    let route_triage = triage
        .route_triage
        .iter()
        .find(|entry| entry.route_id == "camera")
        .expect("route triage should exist");
    assert!(!route_triage.timeline.is_empty());
    assert!(
        route_triage
            .next_steps
            .iter()
            .any(|step| step.contains("Thaw traffic"))
    );
}

#[test]
fn operator_casebook_bundles_incident_recovery_and_handoff_context() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            queue_depth: [("udp_renderer".to_owned(), 4)].into_iter().collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 7,
                recorded_at_unix_ms: 700,
                action: "freeze_traffic".to_owned(),
                details: vec!["applied=true".to_owned()],
            }],
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(3));
    let casebook = proxy_operator_casebook(&snapshot);

    let route_casebook = casebook
        .route_casebooks
        .iter()
        .find(|entry| entry.route_id == "camera")
        .expect("camera casebook should exist");
    assert!(
        route_casebook
            .next_steps
            .iter()
            .any(|step| step.contains("Thaw traffic"))
    );
    assert!(
        route_casebook
            .incident_titles
            .iter()
            .any(|title| title.contains("Traffic frozen"))
    );
    assert_eq!(route_casebook.linked_destination_ids, vec!["udp_renderer"]);
    assert!(
        route_casebook
            .recommended_actions
            .iter()
            .any(|action| action.route_id.as_deref() == Some("camera"))
    );

    let destination_casebook = casebook
        .destination_casebooks
        .iter()
        .find(|entry| entry.destination_id == "udp_renderer")
        .expect("destination casebook should exist");
    assert!(
        destination_casebook
            .recovery_surface
            .iter()
            .any(|entry| entry.contains("queue_depth=4"))
    );
    assert!(
        destination_casebook
            .recent_events
            .iter()
            .any(|event| event.title.contains("Traffic override"))
    );
}

#[test]
fn operator_board_groups_casebooks_into_triage_lanes() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            queue_depth: [("udp_renderer".to_owned(), 2)].into_iter().collect(),
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(3));
    let board = proxy_operator_board(&snapshot);

    assert!(
        board
            .degraded_items
            .iter()
            .any(|item| item.title.contains("Traffic frozen"))
    );
    assert!(
        board
            .degraded_items
            .iter()
            .any(|item| item.route_id.as_deref() == Some("camera"))
    );
    assert!(
        board
            .degraded_items
            .iter()
            .any(|item| item.destination_id.as_deref() == Some("udp_renderer"))
            || board
                .watch_items
                .iter()
                .any(|item| item.destination_id.as_deref() == Some("udp_renderer"))
    );
}

#[test]
fn operator_recovery_ignores_closed_breakers_without_other_pressure() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            destination_breaker_state: [(
                "udp_renderer".to_owned(),
                rosc_telemetry::BreakerStateSnapshot::Closed,
            )]
            .into_iter()
            .collect(),
            ..HealthSnapshot::default()
        },
    );

    let snapshot = proxy_operator_snapshot(&status, ProxyRuntimeSafetyPolicy::default(), Some(4));
    let recovery = proxy_operator_recovery(&snapshot);

    assert!(
        recovery
            .destination_candidates
            .iter()
            .all(|candidate| candidate.destination_id != "udp_renderer")
    );
}

#[test]
fn operator_attention_focuses_problematic_routes_destinations_and_overrides() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 7,
                recorded_at_unix_ms: 700,
                action: "isolate_route".to_owned(),
                details: vec!["route_id=camera".to_owned()],
            }],
            recent_config_events: vec![RecentConfigEvent {
                sequence: 8,
                recorded_at_unix_ms: 800,
                kind: RecentConfigEventKind::Blocked,
                revision: Some(3),
                details: vec!["unsafe wildcard route".to_owned()],
                added_ingresses: 0,
                removed_ingresses: 0,
                changed_ingresses: 0,
                added_destinations: 0,
                removed_destinations: 0,
                changed_destinations: 0,
                added_routes: 0,
                removed_routes: 0,
                changed_routes: 1,
                launch_profile_mode: None,
                disabled_capture_routes: 0,
                disabled_replay_routes: 0,
                disabled_restart_rehydrate_routes: 0,
            }],
            ..HealthSnapshot::default()
        },
    );

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());
    let attention = proxy_operator_attention(&report);

    assert_eq!(attention.state, ProxyOperatorState::Warning);
    assert!(attention.traffic_frozen);
    assert_eq!(attention.isolated_route_ids, vec!["camera"]);
    assert!(
        attention
            .problematic_route_ids
            .contains(&"camera".to_owned())
    );
    assert!(
        attention
            .problematic_destination_ids
            .contains(&"udp_renderer".to_owned())
    );
    assert_eq!(
        attention
            .latest_operator_action
            .as_ref()
            .map(|action| action.action.as_str()),
        Some("isolate_route")
    );
    assert_eq!(
        attention
            .latest_config_issue
            .as_ref()
            .map(|event| &event.kind),
        Some(&RecentConfigEventKind::Blocked)
    );
}

#[test]
fn operator_incidents_filter_recent_issue_history_and_problematic_entities() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            recent_operator_actions: vec![
                RecentOperatorAction {
                    sequence: 1,
                    recorded_at_unix_ms: 100,
                    action: "freeze_traffic".to_owned(),
                    details: vec!["applied=true".to_owned()],
                },
                RecentOperatorAction {
                    sequence: 2,
                    recorded_at_unix_ms: 200,
                    action: "isolate_route".to_owned(),
                    details: vec!["route_id=camera".to_owned()],
                },
            ],
            recent_config_events: vec![
                RecentConfigEvent {
                    sequence: 3,
                    recorded_at_unix_ms: 300,
                    kind: RecentConfigEventKind::Applied,
                    revision: Some(1),
                    details: Vec::new(),
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 0,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 0,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
                RecentConfigEvent {
                    sequence: 4,
                    recorded_at_unix_ms: 400,
                    kind: RecentConfigEventKind::Blocked,
                    revision: Some(2),
                    details: vec!["unsafe wildcard route".to_owned()],
                    added_ingresses: 0,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 0,
                    removed_destinations: 0,
                    changed_destinations: 0,
                    added_routes: 0,
                    removed_routes: 0,
                    changed_routes: 1,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
            ],
            destination_drops_total: [(
                ("udp_renderer".to_owned(), "queue_overflow".to_owned()),
                4,
            )]
            .into_iter()
            .collect(),
            ..HealthSnapshot::default()
        },
    );

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());
    let runtime = status.runtime.as_ref().unwrap();
    let incidents = proxy_operator_incidents_from_histories(
        &report,
        runtime.recent_operator_actions.clone(),
        runtime.recent_config_events.clone(),
        Some(1),
    );

    assert_eq!(incidents.state, ProxyOperatorState::Warning);
    assert_eq!(incidents.recent_operator_actions.len(), 1);
    assert_eq!(incidents.recent_operator_actions[0].action, "isolate_route");
    assert_eq!(incidents.recent_config_issues.len(), 1);
    assert_eq!(
        incidents.recent_config_issues[0].kind,
        RecentConfigEventKind::Blocked
    );
    assert!(
        incidents
            .problematic_routes
            .iter()
            .any(|route| route.route_id == "camera")
    );
    assert!(
        incidents
            .problematic_destinations
            .iter()
            .any(|destination| destination.destination_id == "udp_renderer")
    );
}
