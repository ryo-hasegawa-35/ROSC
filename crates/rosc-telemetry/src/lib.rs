use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BrokerEvent {
    PacketAccepted { ingress_id: String },
    PacketDropped { ingress_id: String, reason: String },
    RouteMatched { route_id: String },
    QueueDepthChanged { queue_id: String, depth: usize },
}

pub trait TelemetrySink: Send + Sync {
    fn emit(&self, event: BrokerEvent);
}

#[derive(Default)]
pub struct NoopTelemetry;

impl TelemetrySink for NoopTelemetry {
    fn emit(&self, _event: BrokerEvent) {}
}
