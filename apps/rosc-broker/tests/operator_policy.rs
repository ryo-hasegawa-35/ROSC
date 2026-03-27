use rosc_broker::{
    ProxyLaunchProfileMode, ProxyOperatorSignalScope, ProxyOperatorState, ProxyRuntimeSafetyPolicy,
    attach_runtime_status, evaluate_proxy_runtime_policy, proxy_operator_report,
    proxy_operator_signals_view, proxy_startup_report_lines, proxy_status_from_config,
};
use rosc_config::BrokerConfig;
use rosc_telemetry::{
    HealthSnapshot, RecentConfigEvent, RecentConfigEventKind, RecentOperatorAction,
};

fn broad_scope_config() -> BrokerConfig {
    BrokerConfig::from_toml_str(
        r#"
        [[udp_destinations]]
        id = "udp_renderer"
        bind = "0.0.0.0:0"
        target = "127.0.0.1:9001"

        [[routes]]
        id = "camera"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/ue5/camera/fov"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"

        [[routes]]
        id = "unsafe"
        enabled = true
        mode = "osc1_0_strict"
        class = "SensorStream"
        [routes.match]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "shadow"
        transport = "internal"
        "#,
    )
    .expect("config should parse")
}

#[test]
fn runtime_policy_blocks_broad_scope_routes_when_requested() {
    let config = broad_scope_config();
    let policy = ProxyRuntimeSafetyPolicy {
        fail_on_warnings: true,
        require_fallback_ready: true,
    };

    let blockers = evaluate_proxy_runtime_policy(&config, policy)
        .expect_err("policy should block broad-scope config");
    assert!(
        blockers
            .iter()
            .any(|reason| reason.contains("matches all ingresses"))
    );
    assert!(
        blockers
            .iter()
            .any(|reason| reason.contains("direct UDP fallback"))
    );
}

#[test]
fn startup_report_lines_include_summary_and_warning_lines() {
    let config = broad_scope_config();
    let status = proxy_status_from_config(&config).expect("status should build");
    let report = proxy_startup_report_lines(&status);

    assert!(report.iter().any(|line| line.starts_with("proxy summary:")));
    assert!(
        report
            .iter()
            .any(|line| line.contains("proxy launch profile: mode=normal"))
    );
    assert!(report.iter().any(|line| line.starts_with("proxy warning:")));
}

#[test]
fn startup_report_lines_include_runtime_config_state_when_available() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            config_revision: 7,
            config_rejections_total: 2,
            config_blocked_total: 3,
            config_reload_failures_total: 1,
            ..HealthSnapshot::default()
        },
    );
    let report = proxy_startup_report_lines(&status);

    assert!(
        report
            .iter()
            .any(|line| line.contains("traffic_frozen=true"))
    );
    assert!(report.iter().any(|line| line.contains("isolated_routes=1")));
    assert!(report.iter().any(|line| line.contains("config_revision=7")));
    assert!(
        report
            .iter()
            .any(|line| line.contains("config_rejections_total=2"))
    );
    assert!(
        report
            .iter()
            .any(|line| line.contains("config_blocked_total=3"))
    );
    assert!(
        report
            .iter()
            .any(|line| line.contains("config_reload_failures_total=1"))
    );
}

#[test]
fn startup_report_lines_include_safe_mode_launch_profile_when_present() {
    let config = broad_scope_config();
    let mut status = proxy_status_from_config(&config).expect("status should build");
    status.launch_profile.mode = ProxyLaunchProfileMode::SafeMode;
    status.launch_profile.disabled_capture_routes = vec!["camera".to_owned()];
    let report = proxy_startup_report_lines(&status);

    assert!(
        report
            .iter()
            .any(|line| line.contains("proxy launch profile: mode=safe_mode"))
    );
    assert!(
        report
            .iter()
            .any(|line| line.contains("disabled_capture_routes=1"))
    );
}

#[test]
fn operator_report_includes_policy_and_blockers() {
    let config = broad_scope_config();
    let status = proxy_status_from_config(&config).expect("status should build");
    let policy = ProxyRuntimeSafetyPolicy {
        fail_on_warnings: true,
        require_fallback_ready: true,
    };

    let report = proxy_operator_report(&status, policy);

    assert_eq!(report.policy, policy);
    assert!(!report.warnings.is_empty());
    assert!(
        report
            .blockers
            .iter()
            .any(|line| line.contains("matches all ingresses"))
    );
    assert!(report.report_lines.iter().any(|line| {
        line.contains("proxy safety policy: fail_on_warnings=true require_fallback_ready=true")
    }));
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.starts_with("proxy blocker:"))
    );
    assert_eq!(report.state, ProxyOperatorState::Blocked);
}

#[test]
fn operator_report_surfaces_state_and_recent_highlights() {
    let config = broad_scope_config();
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            traffic_frozen: true,
            route_isolated: [("camera".to_owned(), true)].into_iter().collect(),
            recent_operator_actions: vec![RecentOperatorAction {
                sequence: 9,
                recorded_at_unix_ms: 1234,
                action: "freeze_traffic".to_owned(),
                details: Vec::new(),
            }],
            recent_config_events: vec![
                RecentConfigEvent {
                    sequence: 8,
                    recorded_at_unix_ms: 1200,
                    kind: RecentConfigEventKind::Applied,
                    revision: Some(1),
                    details: Vec::new(),
                    added_ingresses: 1,
                    removed_ingresses: 0,
                    changed_ingresses: 0,
                    added_destinations: 1,
                    removed_destinations: 0,
                    changed_destinations: 0,
                    added_routes: 1,
                    removed_routes: 0,
                    changed_routes: 0,
                    launch_profile_mode: None,
                    disabled_capture_routes: 0,
                    disabled_replay_routes: 0,
                    disabled_restart_rehydrate_routes: 0,
                },
                RecentConfigEvent {
                    sequence: 10,
                    recorded_at_unix_ms: 1400,
                    kind: RecentConfigEventKind::ReloadFailed,
                    revision: Some(1),
                    details: vec!["reload rollback happened".to_owned()],
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
            ],
            ..HealthSnapshot::default()
        },
    );

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(report.state, ProxyOperatorState::Warning);
    assert!(report.overrides.traffic_frozen);
    assert_eq!(report.overrides.isolated_route_ids, vec!["camera"]);
    assert_eq!(
        report.runtime_signals.ingresses_with_drops,
        Vec::<String>::new()
    );
    assert_eq!(
        report.runtime_signals.destinations_with_open_breakers,
        Vec::<String>::new()
    );
    assert_eq!(report.route_signals.len(), 2);
    assert!(report.route_signals.iter().any(|route| {
        route.route_id == "camera"
            && route.isolated
            && route.dispatch_failures_total == 0
            && route.transform_failures_total == 0
    }));
    assert!(report.destination_signals.iter().any(|destination| {
        destination.destination_id == "udp_renderer"
            && destination.send_failures_total == 0
            && destination.drops_total == 0
    }));
    assert_eq!(
        report
            .highlights
            .latest_operator_action
            .as_ref()
            .map(|action| action.action.as_str()),
        Some("freeze_traffic")
    );
    assert_eq!(
        report
            .highlights
            .latest_config_issue
            .as_ref()
            .map(|event| &event.kind),
        Some(&RecentConfigEventKind::ReloadFailed)
    );
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.contains("proxy operator state: state=warning"))
    );
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.contains("latest_operator_action=freeze_traffic"))
    );
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.contains("proxy overrides: launch_profile_mode=normal"))
    );
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.contains("proxy runtime signals:"))
    );
    assert!(
        report
            .report_lines
            .iter()
            .any(|line| line.contains("latest_config_issue=reload_failed"))
    );
}

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
fn operator_report_returns_to_healthy_after_later_apply() {
    let config = BrokerConfig::from_toml_str(
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
    let status = attach_runtime_status(
        proxy_status_from_config(&config).expect("status should build"),
        &HealthSnapshot {
            recent_config_events: vec![
                RecentConfigEvent {
                    sequence: 10,
                    recorded_at_unix_ms: 1400,
                    kind: RecentConfigEventKind::ReloadFailed,
                    revision: Some(1),
                    details: vec!["reload rollback happened".to_owned()],
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
                    sequence: 11,
                    recorded_at_unix_ms: 1500,
                    kind: RecentConfigEventKind::Applied,
                    revision: Some(2),
                    details: Vec::new(),
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

    let report = proxy_operator_report(&status, ProxyRuntimeSafetyPolicy::default());

    assert_eq!(report.state, ProxyOperatorState::Healthy);
    assert_eq!(
        report
            .highlights
            .latest_config_issue
            .as_ref()
            .map(|event| &event.kind),
        Some(&RecentConfigEventKind::ReloadFailed)
    );
}
