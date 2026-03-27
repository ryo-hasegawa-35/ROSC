use std::collections::BTreeMap;

use rosc_config::DropPolicyConfig;
use rosc_route::{CachePolicy, CapturePolicy, TrafficClass};
use rosc_telemetry::{BreakerStateSnapshot, RecentConfigEvent, RecentOperatorAction};
use serde::Serialize;

use crate::ProxyLaunchProfileStatus;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyStatusSnapshot {
    pub launch_profile: ProxyLaunchProfileStatus,
    pub summary: UdpProxySummary,
    pub runtime: Option<UdpProxyRuntimeStatus>,
    pub ingresses: Vec<UdpProxyIngressStatus>,
    pub destinations: Vec<UdpProxyDestinationStatus>,
    pub routes: Vec<UdpProxyRouteStatus>,
    pub fallback_routes: Vec<UdpProxyFallbackStatus>,
    pub route_assessments: Vec<UdpProxyRouteAssessment>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxySummary {
    pub total_routes: usize,
    pub active_routes: usize,
    pub disabled_routes: usize,
    pub active_ingresses: usize,
    pub active_destinations: usize,
    pub fallback_ready_routes: usize,
    pub fallback_missing_routes: usize,
    pub warning_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRuntimeStatus {
    pub traffic_frozen: bool,
    pub isolated_route_ids: Vec<String>,
    pub operator_actions_total: BTreeMap<String, u64>,
    pub recent_operator_actions: Vec<RecentOperatorAction>,
    pub recent_config_events: Vec<RecentConfigEvent>,
    pub config_revision: u64,
    pub config_rejections_total: u64,
    pub config_blocked_total: u64,
    pub config_reload_failures_total: u64,
    pub ingress_packets_total: BTreeMap<String, u64>,
    pub ingress_drops_total: BTreeMap<String, u64>,
    pub dispatch_failures_total: BTreeMap<String, u64>,
    pub route_matches_total: BTreeMap<String, u64>,
    pub route_transform_failures_total: BTreeMap<String, u64>,
    pub destination_drops_total: BTreeMap<String, u64>,
    pub destinations: Vec<UdpProxyDestinationRuntimeStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyDestinationRuntimeStatus {
    pub destination_id: String,
    pub queue_depth: usize,
    pub send_total: u64,
    pub send_failures_total: u64,
    pub breaker_state: Option<BreakerStateSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyIngressStatus {
    pub id: String,
    pub configured_bind: String,
    pub bound_local_addr: Option<String>,
    pub route_ids: Vec<String>,
    pub max_packet_size: usize,
    pub mode: rosc_osc::CompatibilityMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyDestinationStatus {
    pub id: String,
    pub bind: String,
    pub target: String,
    pub route_ids: Vec<String>,
    pub queue_depth: usize,
    pub drop_policy: DropPolicyConfig,
    pub breaker_open_after_consecutive_failures: u32,
    pub breaker_open_after_consecutive_queue_overflows: u32,
    pub breaker_cooldown_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRouteStatus {
    pub id: String,
    pub enabled: bool,
    pub mode: rosc_osc::CompatibilityMode,
    pub traffic_class: TrafficClass,
    pub ingress_ids: Vec<String>,
    pub address_patterns: Vec<String>,
    pub destination_ids: Vec<String>,
    pub rename_address: Option<String>,
    pub cache_policy: CachePolicy,
    pub capture_policy: CapturePolicy,
    pub rehydrate_on_connect: bool,
    pub replay_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyFallbackStatus {
    pub route_id: String,
    pub direct_udp_targets: Vec<String>,
    pub available: bool,
    pub note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyRouteAssessment {
    pub route_id: String,
    pub active: bool,
    pub direct_udp_fallback_available: bool,
    pub warning_count: usize,
    pub warnings: Vec<String>,
}
