use assert_cmd::Command;

fn workspace_root() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
}

fn json_stdout_for(args: &[&str]) -> serde_json::Value {
    let assert = Command::cargo_bin("rosc-broker")
        .unwrap()
        .current_dir(workspace_root())
        .args(args)
        .assert()
        .success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&output).unwrap_or_else(|error| {
        panic!("stdout must be valid JSON, got error {error}: {output}");
    })
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
