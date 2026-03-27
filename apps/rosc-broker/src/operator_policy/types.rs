use rosc_telemetry::BreakerStateSnapshot;
use rosc_telemetry::{RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::{UdpProxyStatusSnapshot, startup_blockers};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyRuntimeSafetyPolicy {
    pub fail_on_warnings: bool,
    pub require_fallback_ready: bool,
}

impl ProxyRuntimeSafetyPolicy {
    pub fn blockers(self, status: &UdpProxyStatusSnapshot) -> Vec<String> {
        startup_blockers(status, self.fail_on_warnings, self.require_fallback_ready)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorReport {
    pub state: ProxyOperatorState,
    pub policy: ProxyRuntimeSafetyPolicy,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub overrides: ProxyOperatorOverrides,
    pub runtime_signals: ProxyOperatorRuntimeSignals,
    pub route_signals: Vec<ProxyOperatorRouteSignal>,
    pub destination_signals: Vec<ProxyOperatorDestinationSignal>,
    pub highlights: ProxyOperatorHighlights,
    pub report_lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorSignalScope {
    #[default]
    All,
    Problematic,
}

impl ProxyOperatorSignalScope {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "problematic" => Some(Self::Problematic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperatorState {
    Healthy,
    Warning,
    Blocked,
}

impl ProxyOperatorState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorHighlights {
    pub latest_operator_action: Option<RecentOperatorAction>,
    pub latest_config_issue: Option<RecentConfigEvent>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorOverrides {
    pub launch_profile_mode: String,
    pub traffic_frozen: bool,
    pub isolated_route_ids: Vec<String>,
    pub disabled_capture_routes: Vec<String>,
    pub disabled_replay_routes: Vec<String>,
    pub disabled_restart_rehydrate_routes: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRuntimeSignals {
    pub ingresses_with_drops: Vec<String>,
    pub routes_with_dispatch_failures: Vec<String>,
    pub routes_with_transform_failures: Vec<String>,
    pub destinations_with_drops: Vec<String>,
    pub destinations_with_send_failures: Vec<String>,
    pub destinations_with_open_breakers: Vec<String>,
    pub destinations_with_half_open_breakers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorRouteSignal {
    pub route_id: String,
    pub active: bool,
    pub isolated: bool,
    pub direct_udp_fallback_available: bool,
    pub config_warnings: Vec<String>,
    pub dispatch_failures_total: u64,
    pub transform_failures_total: u64,
}

impl ProxyOperatorRouteSignal {
    pub fn is_problematic(&self) -> bool {
        !self.active
            || self.isolated
            || !self.direct_udp_fallback_available
            || !self.config_warnings.is_empty()
            || self.dispatch_failures_total > 0
            || self.transform_failures_total > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorDestinationSignal {
    pub destination_id: String,
    pub queue_depth: usize,
    pub send_total: u64,
    pub send_failures_total: u64,
    pub drops_total: u64,
    pub breaker_state: Option<BreakerStateSnapshot>,
}

impl ProxyOperatorDestinationSignal {
    pub fn is_problematic(&self) -> bool {
        self.queue_depth > 0
            || self.send_failures_total > 0
            || self.drops_total > 0
            || matches!(
                self.breaker_state,
                Some(BreakerStateSnapshot::Open | BreakerStateSnapshot::HalfOpen)
            )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProxyOperatorSignalsView {
    pub scope: ProxyOperatorSignalScope,
    pub runtime_signals: ProxyOperatorRuntimeSignals,
    pub route_signals: Vec<ProxyOperatorRouteSignal>,
    pub destination_signals: Vec<ProxyOperatorDestinationSignal>,
}
