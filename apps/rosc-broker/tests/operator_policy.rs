use rosc_broker::{
    ProxyRuntimeSafetyPolicy, evaluate_proxy_runtime_policy, proxy_startup_report_lines,
    proxy_status_from_config,
};
use rosc_config::BrokerConfig;

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
    assert!(report.iter().any(|line| line.starts_with("proxy warning:")));
}
