use rosc_broker::{
    ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy, attach_runtime_status,
    evaluate_proxy_runtime_policy, proxy_startup_report_lines, proxy_status_from_config,
};
use rosc_config::BrokerConfig;
use rosc_telemetry::HealthSnapshot;

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
