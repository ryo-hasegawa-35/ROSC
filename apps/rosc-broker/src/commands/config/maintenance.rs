use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rosc_telemetry::InMemoryTelemetry;

use super::super::common::print_applied_config;

pub(crate) async fn watch_config(
    path: &Path,
    poll_ms: u64,
    fail_on_warnings: bool,
    require_fallback_ready: bool,
) -> Result<()> {
    let telemetry = InMemoryTelemetry::default();
    let mut supervisor = rosc_broker::ConfigFileSupervisor::new(path, telemetry);
    let safety_policy = rosc_broker::ProxyRuntimeSafetyPolicy {
        fail_on_warnings,
        require_fallback_ready,
    };
    let applied = supervisor.load_initial_with_guard(|config| {
        rosc_broker::evaluate_proxy_runtime_policy(config, safety_policy)
    })?;
    println!(
        "loaded initial config: revision={} added_ingresses={} added_destinations={} added_routes={}",
        applied.revision,
        applied.diff.added_ingresses.join(","),
        applied.diff.added_destinations.join(","),
        applied.diff.added_routes.join(",")
    );

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(poll_ms));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match supervisor.poll_once_with_guard(|config| {
                    rosc_broker::evaluate_proxy_runtime_policy(config, safety_policy)
                })? {
                    rosc_broker::ConfigReloadOutcome::Unchanged => {}
                    rosc_broker::ConfigReloadOutcome::Applied(applied) => {
                        print_applied_config(&applied);
                    }
                    rosc_broker::ConfigReloadOutcome::Rejected(error) => {
                        let revision = supervisor.current_revision().unwrap_or_default();
                        println!(
                            "rejected config change; keeping revision={} reason={}",
                            revision,
                            error
                        );
                    }
                    rosc_broker::ConfigReloadOutcome::Blocked(reasons) => {
                        let revision = supervisor.current_revision().unwrap_or_default();
                        println!(
                            "blocked config change; keeping revision={} reasons={}",
                            revision,
                            reasons.join(" | ")
                        );
                    }
                }
            }
            result = tokio::signal::ctrl_c() => {
                result.context("failed to listen for ctrl-c")?;
                break;
            }
        }
    }
    println!("config watcher stopped");
    Ok(())
}

pub(crate) async fn diff_config(current: &Path, candidate: &Path) -> Result<()> {
    let current_content = fs::read_to_string(current)
        .with_context(|| format!("failed to read config file {}", current.display()))?;
    let candidate_content = fs::read_to_string(candidate)
        .with_context(|| format!("failed to read config file {}", candidate.display()))?;

    let mut manager = rosc_config::ConfigManager::default();
    let applied = manager.apply_toml_str(&current_content)?;
    let diff = manager.preview_toml_diff(&candidate_content)?;

    println!("current_revision={}", applied.revision);
    println!("added_ingresses={}", diff.added_ingresses.join(","));
    println!("removed_ingresses={}", diff.removed_ingresses.join(","));
    println!("changed_ingresses={}", diff.changed_ingresses.join(","));
    println!("added_destinations={}", diff.added_destinations.join(","));
    println!(
        "removed_destinations={}",
        diff.removed_destinations.join(",")
    );
    println!(
        "changed_destinations={}",
        diff.changed_destinations.join(",")
    );
    println!("added_routes={}", diff.added_routes.join(","));
    println!("removed_routes={}", diff.removed_routes.join(","));
    println!("changed_routes={}", diff.changed_routes.join(","));
    Ok(())
}
