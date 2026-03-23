use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BreakerStateSnapshot {
    Closed,
    Open,
    HalfOpen,
}

impl BreakerStateSnapshot {
    fn as_metric_value(&self) -> u8 {
        match self {
            Self::Closed => 0,
            Self::Open => 1,
            Self::HalfOpen => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BrokerEvent {
    PacketAccepted {
        ingress_id: String,
    },
    PacketDropped {
        ingress_id: String,
        reason: String,
    },
    RouteMatched {
        route_id: String,
    },
    QueueDepthChanged {
        queue_id: String,
        depth: usize,
    },
    DestinationSent {
        destination_id: String,
    },
    DestinationSendFailed {
        destination_id: String,
        reason: String,
    },
    DestinationBreakerChanged {
        destination_id: String,
        state: BreakerStateSnapshot,
        reason: String,
    },
    ConfigApplied {
        revision: u64,
        added_routes: usize,
        removed_routes: usize,
        changed_routes: usize,
    },
}

pub trait TelemetrySink: Send + Sync {
    fn emit(&self, event: BrokerEvent);
}

#[derive(Default)]
pub struct NoopTelemetry;

impl TelemetrySink for NoopTelemetry {
    fn emit(&self, _event: BrokerEvent) {}
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub ingress_packets_total: BTreeMap<String, u64>,
    pub ingress_drops_total: BTreeMap<(String, String), u64>,
    pub route_matches_total: BTreeMap<String, u64>,
    pub queue_depth: BTreeMap<String, usize>,
    pub destination_sent_total: BTreeMap<String, u64>,
    pub destination_send_failures_total: BTreeMap<(String, String), u64>,
    pub destination_breaker_state: BTreeMap<String, BreakerStateSnapshot>,
    pub config_revision: u64,
    pub config_added_routes_total: u64,
    pub config_removed_routes_total: u64,
    pub config_changed_routes_total: u64,
}

#[derive(Clone, Default)]
pub struct InMemoryTelemetry {
    inner: Arc<Mutex<HealthSnapshot>>,
}

impl InMemoryTelemetry {
    pub fn snapshot(&self) -> HealthSnapshot {
        self.inner.lock().expect("telemetry mutex poisoned").clone()
    }

    pub fn render_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        let mut output = String::new();

        for (ingress_id, count) in snapshot.ingress_packets_total {
            let _ = writeln!(
                output,
                "rosc_ingress_packets_total{{ingress_id=\"{ingress_id}\"}} {count}"
            );
        }

        for ((ingress_id, reason), count) in snapshot.ingress_drops_total {
            let _ = writeln!(
                output,
                "rosc_ingress_drops_total{{ingress_id=\"{ingress_id}\",reason=\"{reason}\"}} {count}"
            );
        }

        for (route_id, count) in snapshot.route_matches_total {
            let _ = writeln!(
                output,
                "rosc_route_matches_total{{route_id=\"{route_id}\"}} {count}"
            );
        }

        for (queue_id, depth) in snapshot.queue_depth {
            let _ = writeln!(
                output,
                "rosc_queue_depth{{queue_id=\"{queue_id}\"}} {depth}"
            );
        }

        for (destination_id, count) in snapshot.destination_sent_total {
            let _ = writeln!(
                output,
                "rosc_destination_send_total{{destination_id=\"{destination_id}\"}} {count}"
            );
        }

        for ((destination_id, reason), count) in snapshot.destination_send_failures_total {
            let _ = writeln!(
                output,
                "rosc_destination_send_failures_total{{destination_id=\"{destination_id}\",reason=\"{reason}\"}} {count}"
            );
        }

        for (destination_id, state) in snapshot.destination_breaker_state {
            let _ = writeln!(
                output,
                "rosc_destination_breaker_state{{destination_id=\"{destination_id}\"}} {}",
                state.as_metric_value()
            );
        }

        let _ = writeln!(output, "rosc_config_revision {}", snapshot.config_revision);
        let _ = writeln!(
            output,
            "rosc_config_added_routes_total {}",
            snapshot.config_added_routes_total
        );
        let _ = writeln!(
            output,
            "rosc_config_removed_routes_total {}",
            snapshot.config_removed_routes_total
        );
        let _ = writeln!(
            output,
            "rosc_config_changed_routes_total {}",
            snapshot.config_changed_routes_total
        );

        output
    }
}

impl TelemetrySink for InMemoryTelemetry {
    fn emit(&self, event: BrokerEvent) {
        let mut snapshot = self.inner.lock().expect("telemetry mutex poisoned");
        match event {
            BrokerEvent::PacketAccepted { ingress_id } => {
                *snapshot
                    .ingress_packets_total
                    .entry(ingress_id)
                    .or_default() += 1;
            }
            BrokerEvent::PacketDropped { ingress_id, reason } => {
                *snapshot
                    .ingress_drops_total
                    .entry((ingress_id, reason))
                    .or_default() += 1;
            }
            BrokerEvent::RouteMatched { route_id } => {
                *snapshot.route_matches_total.entry(route_id).or_default() += 1;
            }
            BrokerEvent::QueueDepthChanged { queue_id, depth } => {
                snapshot.queue_depth.insert(queue_id, depth);
            }
            BrokerEvent::DestinationSent { destination_id } => {
                *snapshot
                    .destination_sent_total
                    .entry(destination_id)
                    .or_default() += 1;
            }
            BrokerEvent::DestinationSendFailed {
                destination_id,
                reason,
            } => {
                *snapshot
                    .destination_send_failures_total
                    .entry((destination_id, reason))
                    .or_default() += 1;
            }
            BrokerEvent::DestinationBreakerChanged {
                destination_id,
                state,
                ..
            } => {
                snapshot
                    .destination_breaker_state
                    .insert(destination_id, state);
            }
            BrokerEvent::ConfigApplied {
                revision,
                added_routes,
                removed_routes,
                changed_routes,
            } => {
                snapshot.config_revision = revision;
                snapshot.config_added_routes_total += added_routes as u64;
                snapshot.config_removed_routes_total += removed_routes as u64;
                snapshot.config_changed_routes_total += changed_routes as u64;
            }
        }
    }
}
