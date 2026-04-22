use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
}

fn command_for(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("rosc-broker")
        .unwrap()
        .current_dir(workspace_root())
        .args(args)
        .assert()
}

fn json_stdout_for(args: &[&str]) -> serde_json::Value {
    let assert = command_for(args).success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&output).unwrap_or_else(|error| {
        panic!("stdout must be valid JSON, got error {error}: {output}");
    })
}

fn temp_config_path(name: &str, body: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rosc-{name}-{unique}.toml"));
    fs::write(&path, body).unwrap();
    path
}

#[test]
fn proxy_overview_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-overview",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
    ]);

    assert!(value.get("status").is_some());
    assert!(value.get("report").is_some());
    assert!(value.get("runtime_summary").is_some());
}

#[test]
fn proxy_readiness_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-readiness",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
    ]);

    assert!(value.get("level").is_some());
    assert!(value.get("flags").is_some());
    assert!(value.get("counts").is_some());
}

#[test]
fn proxy_assert_ready_fails_on_blocked_config_and_keeps_json_stdout() {
    let config = temp_config_path(
        "blocked",
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
        ingress_ids = []
        address_patterns = []
        protocols = ["osc_udp"]

        [[routes.destinations]]
        target = "udp_renderer"
        transport = "osc_udp"
        "#,
    );

    let config_string = config.to_string_lossy().to_string();
    let assert = command_for(&[
        "proxy-assert-ready",
        &config_string,
        "--fail-on-warnings",
        "--require-fallback-ready",
    ])
    .failure();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["level"], "blocked");

    let _ = fs::remove_file(config);
}

#[test]
fn proxy_snapshot_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-snapshot",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
    ]);

    assert!(value.get("overview").is_some());
    assert!(value.get("readiness").is_some());
    assert!(value.get("diagnostics").is_some());
    assert!(value.get("attention").is_some());
    assert!(value.get("incidents").is_some());
}

#[test]
fn proxy_diagnostics_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-diagnostics",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
    ]);

    assert!(value.get("overview").is_some());
    assert!(value.get("recent_operator_actions").is_some());
    assert!(value.get("recent_config_events").is_some());
}

#[test]
fn proxy_attention_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-attention",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("problematic_route_ids").is_some());
    assert!(value.get("problematic_destination_ids").is_some());
}

#[test]
fn proxy_incidents_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-incidents",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("open_blockers").is_some());
    assert!(value.get("recent_operator_actions").is_some());
    assert!(value.get("recent_config_issues").is_some());
}

#[test]
fn proxy_handoff_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-handoff",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("route_handoffs").is_some());
    assert!(value.get("destination_handoffs").is_some());
}

#[test]
fn proxy_timeline_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-timeline",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("global").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_triage_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-triage",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("global").is_some());
    assert!(value.get("route_triage").is_some());
    assert!(value.get("destination_triage").is_some());
    assert_eq!(value["route_triage"].as_array().unwrap().len(), 1);
    assert_eq!(value["route_triage"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_casebook_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-casebook",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("route_casebooks").is_some());
    assert!(value.get("destination_casebooks").is_some());
    assert_eq!(value["route_casebooks"].as_array().unwrap().len(), 1);
    assert_eq!(value["route_casebooks"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_board_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-board",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("blocked_items").is_some());
    assert!(value.get("degraded_items").is_some());
    assert!(value.get("watch_items").is_some());
}

#[test]
fn proxy_focus_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-focus",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_lens_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-lens",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global_blockers").is_some());
    assert!(value.get("global_overrides").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_brief_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-brief",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global_blockers").is_some());
    assert!(value.get("global_overrides").is_some());
    assert!(value.get("global_next_steps").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
}

#[test]
fn proxy_dossier_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-dossier",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global_blockers").is_some());
    assert!(value.get("global_overrides").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
    assert!(value["routes"][0]["brief"].is_object());
    assert!(value["routes"][0]["lens"].is_object());
}

#[test]
fn proxy_runbook_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-runbook",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
    assert!(value["routes"][0]["dossier"].is_object());
}

#[test]
fn proxy_mission_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-mission",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
    assert!(value["routes"][0]["brief"].is_object());
    assert!(value["routes"][0]["dossier"].is_object());
    assert!(value["routes"][0]["runbook"].is_object());
}

#[test]
fn proxy_workspace_stdout_is_json_only() {
    let value = json_stdout_for(&[
        "proxy-workspace",
        "examples/phase-01-basic.toml",
        "--fail-on-warnings",
        "--require-fallback-ready",
        "--history-limit",
        "5",
        "--route-id",
        "ue5_camera_fov",
    ]);

    assert!(value.get("state").is_some());
    assert!(value.get("global").is_some());
    assert!(value.get("routes").is_some());
    assert!(value.get("destinations").is_some());
    assert_eq!(value["routes"].as_array().unwrap().len(), 1);
    assert_eq!(value["routes"][0]["route_id"], "ue5_camera_fov");
    assert!(value["routes"][0]["mission"].is_object());
    assert!(value["routes"][0]["board_items"].is_array());
    assert!(value["routes"][0]["work_items"].is_array());
}
