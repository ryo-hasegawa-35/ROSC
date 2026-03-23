use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rosc_broker::{ConfigFileSupervisor, ConfigReloadOutcome};
use rosc_telemetry::InMemoryTelemetry;

fn unique_config_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("rosc-config-reload-{nonce}.toml"))
}

fn base_config() -> &'static str {
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
    "#
}

#[test]
fn config_supervisor_preserves_last_known_good_after_rejection() {
    let path = unique_config_path();
    fs::write(&path, base_config()).unwrap();

    let telemetry = InMemoryTelemetry::default();
    let mut supervisor = ConfigFileSupervisor::new(&path, telemetry.clone());
    let applied = supervisor.load_initial().unwrap();
    assert_eq!(applied.revision, 1);

    fs::write(&path, "schema_version = 99").unwrap();
    let outcome = supervisor.poll_once().unwrap();
    match outcome {
        ConfigReloadOutcome::Rejected(error) => {
            assert!(error.to_string().contains("unsupported schema version"));
        }
        other => panic!("expected rejected config, got {other:?}"),
    }

    assert_eq!(supervisor.current_revision(), Some(1));
    let metrics = telemetry.render_prometheus();
    assert!(metrics.contains("rosc_config_rejections_total 1"));

    let _ = fs::remove_file(path);
}

#[test]
fn config_supervisor_applies_changed_valid_config() {
    let path = unique_config_path();
    fs::write(&path, base_config()).unwrap();

    let mut supervisor = ConfigFileSupervisor::new(&path, InMemoryTelemetry::default());
    supervisor.load_initial().unwrap();

    fs::write(
        &path,
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
        id = "tracking"
        enabled = true
        mode = "osc1_1_extended"
        class = "SensorStream"
        [routes.match]
        address_patterns = ["//tracking"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    )
    .unwrap();

    let outcome = supervisor.poll_once().unwrap();
    match outcome {
        ConfigReloadOutcome::Applied(applied) => {
            assert_eq!(applied.revision, 2);
            assert_eq!(applied.diff.added_routes, vec!["tracking"]);
        }
        other => panic!("expected applied config, got {other:?}"),
    }

    assert_eq!(supervisor.current_revision(), Some(2));

    let _ = fs::remove_file(path);
}
