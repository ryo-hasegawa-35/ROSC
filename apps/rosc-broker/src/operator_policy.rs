use anyhow::Result;
use rosc_config::BrokerConfig;

use crate::{
    UdpProxyStatusSnapshot, operator_warnings, proxy_status_from_config, startup_blockers,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProxyRuntimeSafetyPolicy {
    pub fail_on_warnings: bool,
    pub require_fallback_ready: bool,
}

impl ProxyRuntimeSafetyPolicy {
    pub fn blockers(self, status: &UdpProxyStatusSnapshot) -> Vec<String> {
        startup_blockers(status, self.fail_on_warnings, self.require_fallback_ready)
    }
}

pub fn evaluate_proxy_runtime_policy(
    config: &BrokerConfig,
    policy: ProxyRuntimeSafetyPolicy,
) -> Result<(), Vec<String>> {
    let status = match proxy_status_from_config(config) {
        Ok(status) => status,
        Err(error) => return Err(vec![error.to_string()]),
    };
    let blockers = policy.blockers(&status);
    if blockers.is_empty() {
        Ok(())
    } else {
        Err(blockers)
    }
}

pub fn proxy_startup_report_lines(status: &UdpProxyStatusSnapshot) -> Vec<String> {
    let mut lines = vec![format!(
        "proxy summary: active_routes={} disabled_routes={} active_ingresses={} active_destinations={} fallback_ready_routes={} fallback_missing_routes={} warnings={}",
        status.summary.active_routes,
        status.summary.disabled_routes,
        status.summary.active_ingresses,
        status.summary.active_destinations,
        status.summary.fallback_ready_routes,
        status.summary.fallback_missing_routes,
        status.summary.warning_count
    )];

    lines.push(format!(
        "proxy launch profile: mode={} disabled_capture_routes={} disabled_replay_routes={} disabled_restart_rehydrate_routes={}",
        status.launch_profile.mode.as_str(),
        status.launch_profile.disabled_capture_routes.len(),
        status.launch_profile.disabled_replay_routes.len(),
        status
            .launch_profile
            .disabled_restart_rehydrate_routes
            .len()
    ));

    if let Some(runtime) = &status.runtime {
        lines.push(format!(
            "proxy runtime: traffic_frozen={} config_revision={} config_rejections_total={} config_blocked_total={} config_reload_failures_total={}",
            runtime.traffic_frozen,
            runtime.config_revision,
            runtime.config_rejections_total,
            runtime.config_blocked_total,
            runtime.config_reload_failures_total
        ));
    }

    lines.extend(
        operator_warnings(status)
            .into_iter()
            .map(|warning| format!("proxy warning: {warning}")),
    );
    lines
}
