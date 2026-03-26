use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rosc_broker::{ConfigFileSupervisor, ConfigReloadOutcome};
use rosc_telemetry::InMemoryTelemetry;

static UNIQUE_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_config_path() -> PathBuf {
    let nonce = UNIQUE_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("rosc-config-reload-{pid}-{nonce}.toml"))
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
    assert!(metrics.contains("rosc_config_blocked_total 0"));
    let snapshot = telemetry.snapshot();
    assert_eq!(snapshot.recent_config_events.len(), 2);
    assert!(
        snapshot.recent_config_events[1]
            .details
            .iter()
            .any(|detail| detail.contains("unsupported schema version"))
    );

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
            assert!(applied.diff.added_destinations.is_empty());
        }
        other => panic!("expected applied config, got {other:?}"),
    }

    assert_eq!(supervisor.current_revision(), Some(2));

    let _ = fs::remove_file(path);
}

#[test]
fn config_supervisor_reports_destination_policy_only_changes() {
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
        [udp_destinations.policy]
        queue_depth = 32

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
        "#,
    )
    .unwrap();

    let outcome = supervisor.poll_once().unwrap();
    match outcome {
        ConfigReloadOutcome::Applied(applied) => {
            assert_eq!(applied.revision, 2);
            assert_eq!(applied.diff.changed_destinations, vec!["udp_renderer"]);
            assert!(applied.diff.changed_routes.is_empty());
        }
        other => panic!("expected applied config, got {other:?}"),
    }

    let _ = fs::remove_file(path);
}

#[test]
fn config_supervisor_blocks_candidate_when_guard_rejects_it() {
    let path = unique_config_path();
    fs::write(&path, base_config()).unwrap();

    let telemetry = InMemoryTelemetry::default();
    let mut supervisor = ConfigFileSupervisor::new(&path, telemetry.clone());
    let applied = supervisor
        .load_initial_with_guard(|_| Ok(()))
        .expect("initial config should load");
    assert_eq!(applied.revision, 1);

    fs::write(
        &path,
        r#"
        [[routes]]
        id = "unsafe"
        enabled = true
        mode = "osc1_0_strict"
        class = "StatefulControl"
        [routes.match]
        address_patterns = ["/unsafe"]
        protocols = ["osc_udp"]
        [[routes.destinations]]
        target = "shadow"
        transport = "internal"
        "#,
    )
    .unwrap();

    let outcome = supervisor
        .poll_once_with_guard(|config| {
            let status = rosc_broker::proxy_status_from_config(config)
                .map_err(|error| vec![error.to_string()])?;
            let blockers = rosc_broker::startup_blockers(&status, true, true);
            if blockers.is_empty() {
                Ok(())
            } else {
                Err(blockers)
            }
        })
        .unwrap();

    match outcome {
        ConfigReloadOutcome::Blocked(reasons) => {
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("direct UDP fallback"))
            );
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("matches all ingresses"))
            );
        }
        other => panic!("expected blocked config, got {other:?}"),
    }

    assert_eq!(supervisor.current_revision(), Some(1));
    let metrics = telemetry.render_prometheus();
    assert!(metrics.contains("rosc_config_rejections_total 0"));
    assert!(metrics.contains("rosc_config_blocked_total 1"));
    let snapshot = telemetry.snapshot();
    assert_eq!(snapshot.recent_config_events.len(), 2);
    assert!(
        snapshot.recent_config_events[1]
            .details
            .iter()
            .any(|detail| detail.contains("direct UDP fallback"))
    );
    assert!(
        snapshot.recent_config_events[1]
            .details
            .iter()
            .any(|detail| detail.contains("matches all ingresses"))
    );

    let _ = fs::remove_file(path);
}
