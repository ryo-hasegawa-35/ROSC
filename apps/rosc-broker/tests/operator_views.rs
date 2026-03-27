mod common;

use common::broad_scope_config;
use rosc_broker::{
    ProxyOperatorSignalScope, ProxyOperatorState, ProxyRuntimeSafetyPolicy, attach_runtime_status,
    proxy_operator_attention, proxy_operator_diagnostics, proxy_operator_incidents_from_histories,
    proxy_operator_overview, proxy_operator_report, proxy_operator_signals_view,
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
