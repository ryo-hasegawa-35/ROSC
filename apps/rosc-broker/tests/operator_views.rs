mod common;

use common::broad_scope_config;
use rosc_broker::{
    ProxyOperatorSignalScope, ProxyOperatorState, ProxyOperatorTimelineCategory,
    ProxyRuntimeSafetyPolicy, attach_runtime_status, proxy_operator_attention,
    proxy_operator_dashboard, proxy_operator_diagnostics, proxy_operator_incidents_from_histories,
    proxy_operator_overview, proxy_operator_readiness, proxy_operator_report,
    proxy_operator_signals_view, proxy_operator_snapshot, proxy_status_from_config,
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
