use std::io;
use std::net::SocketAddr;
use std::time::SystemTime;

use rosc_osc::CompatibilityMode;
use rosc_packet::{IngressMetadata, PacketBuildError, PacketEnvelope, TransportKind};
use rosc_route::RoutingEngine;
use rosc_telemetry::{BrokerEvent, TelemetrySink};
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuePolicy {
    pub max_depth: usize,
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("ingress queue is full")]
    QueueFull,
    #[error("ingress queue is closed")]
    QueueClosed,
}

#[derive(Clone, Debug)]
pub struct UdpIngressConfig {
    pub ingress_id: String,
    pub compatibility_mode: CompatibilityMode,
    pub max_packet_size: usize,
}

#[derive(Debug, Error)]
pub enum UdpIngressError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] PacketBuildError),
}

pub struct IngressQueue {
    tx: mpsc::Sender<PacketEnvelope>,
}

pub type IngressReceiver = mpsc::Receiver<PacketEnvelope>;

impl IngressQueue {
    pub fn new(policy: QueuePolicy) -> (Self, IngressReceiver) {
        let (tx, rx) = mpsc::channel(policy.max_depth);
        (Self { tx }, rx)
    }

    pub fn try_send(&self, packet: PacketEnvelope) -> Result<(), QueueError> {
        self.tx.try_send(packet).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => QueueError::QueueFull,
            mpsc::error::TrySendError::Closed(_) => QueueError::QueueClosed,
        })
    }
}

pub struct Runtime<TTelemetry> {
    pub routing: RoutingEngine,
    pub telemetry: TTelemetry,
}

pub struct UdpIngressBinding {
    socket: UdpSocket,
    config: UdpIngressConfig,
}

impl<TTelemetry> Runtime<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn route_packet(&self, packet: &PacketEnvelope) -> Result<usize, rosc_route::RoutingError> {
        let dispatches = self.routing.route(packet)?;
        for dispatch in &dispatches {
            self.telemetry.emit(BrokerEvent::RouteMatched {
                route_id: dispatch.route_id.clone(),
            });
        }
        Ok(dispatches.len())
    }
}

impl UdpIngressBinding {
    pub async fn bind(address: &str, config: UdpIngressConfig) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(address).await?;
        Ok(Self { socket, config })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.socket.local_addr()
    }

    pub async fn recv_next(&self) -> Result<PacketEnvelope, UdpIngressError> {
        let mut buffer = vec![0u8; self.config.max_packet_size];
        let (size, source) = self.socket.recv_from(&mut buffer).await?;
        buffer.truncate(size);
        Ok(PacketEnvelope::parse_osc(
            buffer,
            IngressMetadata {
                ingress_id: self.config.ingress_id.clone(),
                transport: TransportKind::OscUdp,
                source_endpoint: Some(source.to_string()),
                compatibility_mode: self.config.compatibility_mode,
                received_at: SystemTime::now(),
            },
        )?)
    }
}
