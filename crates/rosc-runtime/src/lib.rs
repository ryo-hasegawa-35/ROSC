use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use rosc_osc::CompatibilityMode;
use rosc_packet::{IngressMetadata, PacketBuildError, PacketEnvelope, TransportKind};
use rosc_route::{RouteDispatch, RoutingEngine, RoutingOutcome};
use rosc_telemetry::{BreakerStateSnapshot, BrokerEvent, HealthReporter, TelemetrySink};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Notify, mpsc};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropPolicy {
    DropNewest,
    DropOldest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakerPolicy {
    pub open_after_consecutive_failures: u32,
    pub open_after_consecutive_queue_overflows: u32,
    pub cooldown: Duration,
}

impl Default for BreakerPolicy {
    fn default() -> Self {
        Self {
            open_after_consecutive_failures: 3,
            open_after_consecutive_queue_overflows: 3,
            cooldown: Duration::from_millis(250),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestinationPolicy {
    pub queue_depth: usize,
    pub drop_policy: DropPolicy,
    pub breaker: BreakerPolicy,
}

impl Default for DestinationPolicy {
    fn default() -> Self {
        Self {
            queue_depth: 16,
            drop_policy: DropPolicy::DropOldest,
            breaker: BreakerPolicy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestinationStatus {
    pub destination_id: String,
    pub breaker_state: BreakerState,
    pub queue_depth: usize,
    pub consecutive_failures: u32,
    pub consecutive_queue_overflows: u32,
    pub sent_total: u64,
    pub send_failures_total: u64,
    pub dropped_total: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnqueueOutcome {
    Enqueued,
    DroppedNewest,
    DroppedOldest,
}

#[derive(Debug, Error)]
pub enum DestinationDispatchError {
    #[error("destination `{destination_id}` breaker is open")]
    BreakerOpen { destination_id: String },
}

#[derive(Debug, Error)]
pub enum DestinationSendError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Error)]
pub enum RuntimeDispatchError {
    #[error("destination `{0}` is not registered")]
    MissingDestination(String),
    #[error(transparent)]
    Destination(#[from] DestinationDispatchError),
}

pub struct IngressQueue {
    tx: mpsc::Sender<PacketEnvelope>,
}

pub type IngressReceiver = mpsc::Receiver<PacketEnvelope>;

#[async_trait]
pub trait EgressSink: Send + Sync + 'static {
    async fn send(&self, packet: &PacketEnvelope) -> Result<(), DestinationSendError>;
}

pub struct Runtime<TTelemetry> {
    pub routing: RoutingEngine,
    pub telemetry: TTelemetry,
}

pub struct UdpIngressBinding {
    socket: UdpSocket,
    config: UdpIngressConfig,
}

pub struct UdpEgressSink {
    socket: UdpSocket,
    target: SocketAddr,
}

#[derive(Default)]
pub struct DestinationRegistry {
    destinations: BTreeMap<String, DestinationWorkerHandle>,
}

#[derive(Clone)]
pub struct DestinationWorkerHandle {
    destination_id: String,
    queue: Arc<DestinationQueue>,
    state: Arc<Mutex<DestinationState>>,
    policy: DestinationPolicy,
    telemetry: Arc<dyn TelemetrySink>,
}

#[derive(Clone, Debug)]
struct DestinationState {
    status: DestinationStatus,
    open_until: Option<Instant>,
}

struct DestinationQueue {
    capacity: usize,
    inner: tokio::sync::Mutex<VecDeque<PacketEnvelope>>,
    notify: Notify,
}

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

impl<TTelemetry> Runtime<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn route_packet(&self, packet: &PacketEnvelope) -> usize {
        let outcome = self.routing.route(packet);
        self.emit_routing_events(&outcome);
        outcome.dispatches.len()
    }

    pub async fn dispatch_packet(
        &self,
        packet: &PacketEnvelope,
        destinations: &DestinationRegistry,
    ) -> Result<usize, RuntimeDispatchError> {
        let outcome = self.routing.route(packet);
        self.emit_routing_events(&outcome);

        for dispatch in &outcome.dispatches {
            destinations.dispatch(dispatch.clone()).await?;
        }
        Ok(outcome.dispatches.len())
    }

    fn emit_routing_events(&self, outcome: &RoutingOutcome) {
        for dispatch in &outcome.dispatches {
            self.telemetry.emit(BrokerEvent::RouteMatched {
                route_id: dispatch.route_id.clone(),
            });
        }
        for failure in &outcome.failures {
            self.telemetry.emit(BrokerEvent::RouteTransformFailed {
                route_id: failure.route_id.clone(),
            });
        }
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

impl UdpEgressSink {
    pub async fn bind(bind_address: &str, target: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(bind_address).await?;
        Ok(Self { socket, target })
    }
}

pub async fn serve_health_http_once(
    listener: &TcpListener,
    reporter: Arc<dyn HealthReporter>,
) -> Result<(), io::Error> {
    let (stream, _) = listener.accept().await?;
    handle_health_http_connection(stream, reporter).await
}

#[async_trait]
impl EgressSink for UdpEgressSink {
    async fn send(&self, packet: &PacketEnvelope) -> Result<(), DestinationSendError> {
        self.socket.send_to(&packet.raw_bytes, self.target).await?;
        Ok(())
    }
}

impl DestinationRegistry {
    pub fn register(&mut self, handle: DestinationWorkerHandle) {
        self.destinations
            .insert(handle.destination_id.clone(), handle);
    }

    pub fn status(&self, destination_id: &str) -> Option<DestinationStatus> {
        self.destinations
            .get(destination_id)
            .map(DestinationWorkerHandle::status)
    }

    pub async fn dispatch(
        &self,
        dispatch: RouteDispatch,
    ) -> Result<EnqueueOutcome, RuntimeDispatchError> {
        let destination_id = dispatch.destination.destination_id().to_owned();
        let Some(handle) = self.destinations.get(&destination_id) else {
            return Err(RuntimeDispatchError::MissingDestination(destination_id));
        };
        handle
            .enqueue(dispatch.packet)
            .await
            .map_err(RuntimeDispatchError::from)
    }
}

impl DestinationWorkerHandle {
    pub fn spawn(
        destination_id: impl Into<String>,
        policy: DestinationPolicy,
        sink: Arc<dyn EgressSink>,
        telemetry: Arc<dyn TelemetrySink>,
    ) -> Self {
        let destination_id = destination_id.into();
        let queue = Arc::new(DestinationQueue::new(policy.queue_depth));
        let state = Arc::new(Mutex::new(DestinationState {
            status: DestinationStatus {
                destination_id: destination_id.clone(),
                breaker_state: BreakerState::Closed,
                queue_depth: 0,
                consecutive_failures: 0,
                consecutive_queue_overflows: 0,
                sent_total: 0,
                send_failures_total: 0,
                dropped_total: 0,
            },
            open_until: None,
        }));

        let task_queue = Arc::clone(&queue);
        let task_state = Arc::clone(&state);
        let task_policy = policy.clone();
        let task_destination_id = destination_id.clone();
        let task_telemetry = Arc::clone(&telemetry);
        tokio::spawn(async move {
            worker_loop(
                task_destination_id,
                task_policy,
                task_queue,
                task_state,
                sink,
                task_telemetry,
            )
            .await;
        });

        Self {
            destination_id,
            queue,
            state,
            policy,
            telemetry,
        }
    }

    pub fn status(&self) -> DestinationStatus {
        self.state
            .lock()
            .expect("destination status mutex poisoned")
            .status
            .clone()
    }

    pub async fn enqueue(
        &self,
        packet: PacketEnvelope,
    ) -> Result<EnqueueOutcome, DestinationDispatchError> {
        self.ensure_breaker_allows_enqueue()?;

        let queue_result = self.queue.enqueue(packet, self.policy.drop_policy).await;
        let mut events = Vec::new();

        {
            let mut state = self
                .state
                .lock()
                .expect("destination status mutex poisoned");
            state.status.queue_depth = queue_result.depth;
            events.push(BrokerEvent::QueueDepthChanged {
                queue_id: self.destination_id.clone(),
                depth: queue_result.depth,
            });

            match queue_result.outcome {
                EnqueueOutcome::Enqueued => {
                    state.status.consecutive_queue_overflows = 0;
                }
                EnqueueOutcome::DroppedNewest | EnqueueOutcome::DroppedOldest => {
                    state.status.dropped_total += 1;
                    state.status.consecutive_queue_overflows += 1;
                    events.push(BrokerEvent::PacketDropped {
                        ingress_id: self.destination_id.clone(),
                        reason: "destination_queue_overflow".to_owned(),
                    });

                    if state.status.consecutive_queue_overflows
                        >= self.policy.breaker.open_after_consecutive_queue_overflows
                        && self.policy.breaker.open_after_consecutive_queue_overflows > 0
                    {
                        open_breaker(
                            &mut state,
                            &self.destination_id,
                            &self.policy.breaker,
                            &mut events,
                            "queue_overflow",
                        );
                    }
                }
            }
        }

        for event in events {
            self.telemetry.emit(event);
        }

        Ok(queue_result.outcome)
    }

    fn ensure_breaker_allows_enqueue(&self) -> Result<(), DestinationDispatchError> {
        let mut events = Vec::new();
        let mut state = self
            .state
            .lock()
            .expect("destination status mutex poisoned");

        if state.status.breaker_state == BreakerState::Open {
            if let Some(open_until) = state.open_until
                && Instant::now() < open_until
            {
                return Err(DestinationDispatchError::BreakerOpen {
                    destination_id: self.destination_id.clone(),
                });
            }

            state.status.breaker_state = BreakerState::HalfOpen;
            state.open_until = None;
            events.push(BrokerEvent::DestinationBreakerChanged {
                destination_id: self.destination_id.clone(),
                state: BreakerStateSnapshot::HalfOpen,
                reason: "cooldown_elapsed".to_owned(),
            });
        }

        drop(state);
        for event in events {
            self.telemetry.emit(event);
        }
        Ok(())
    }
}

impl DestinationQueue {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: tokio::sync::Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    async fn enqueue(&self, packet: PacketEnvelope, policy: DropPolicy) -> QueueInsertResult {
        let mut queue = self.inner.lock().await;

        let outcome = if queue.len() < self.capacity {
            queue.push_back(packet);
            EnqueueOutcome::Enqueued
        } else {
            match policy {
                DropPolicy::DropNewest => EnqueueOutcome::DroppedNewest,
                DropPolicy::DropOldest => {
                    let _ = queue.pop_front();
                    queue.push_back(packet);
                    EnqueueOutcome::DroppedOldest
                }
            }
        };

        let depth = queue.len();
        drop(queue);
        self.notify.notify_one();
        QueueInsertResult { outcome, depth }
    }

    async fn recv(&self) -> PacketEnvelope {
        loop {
            if let Some(packet) = self.inner.lock().await.pop_front() {
                return packet;
            }
            self.notify.notified().await;
        }
    }

    async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }
}

struct QueueInsertResult {
    outcome: EnqueueOutcome,
    depth: usize,
}

async fn worker_loop(
    destination_id: String,
    policy: DestinationPolicy,
    queue: Arc<DestinationQueue>,
    state: Arc<Mutex<DestinationState>>,
    sink: Arc<dyn EgressSink>,
    telemetry: Arc<dyn TelemetrySink>,
) {
    loop {
        wait_for_breaker(&destination_id, &policy.breaker, &state, &telemetry).await;
        let packet = queue.recv().await;

        telemetry.emit(BrokerEvent::QueueDepthChanged {
            queue_id: destination_id.clone(),
            depth: queue.len().await,
        });

        match sink.send(&packet).await {
            Ok(()) => {
                let mut events = Vec::new();
                {
                    let mut state = state.lock().expect("destination status mutex poisoned");
                    state.status.sent_total += 1;
                    state.status.consecutive_failures = 0;

                    if state.status.breaker_state == BreakerState::HalfOpen {
                        state.status.breaker_state = BreakerState::Closed;
                        events.push(BrokerEvent::DestinationBreakerChanged {
                            destination_id: destination_id.clone(),
                            state: BreakerStateSnapshot::Closed,
                            reason: "probe_success".to_owned(),
                        });
                    }
                }

                events.push(BrokerEvent::DestinationSent {
                    destination_id: destination_id.clone(),
                });
                for event in events {
                    telemetry.emit(event);
                }
            }
            Err(error) => {
                let mut events = Vec::new();
                {
                    let mut state = state.lock().expect("destination status mutex poisoned");
                    state.status.send_failures_total += 1;
                    state.status.consecutive_failures += 1;

                    events.push(BrokerEvent::DestinationSendFailed {
                        destination_id: destination_id.clone(),
                        reason: error.to_string(),
                    });

                    if state.status.consecutive_failures
                        >= policy.breaker.open_after_consecutive_failures
                        && policy.breaker.open_after_consecutive_failures > 0
                    {
                        open_breaker(
                            &mut state,
                            &destination_id,
                            &policy.breaker,
                            &mut events,
                            "send_failure",
                        );
                    }
                }

                for event in events {
                    telemetry.emit(event);
                }
            }
        }
    }
}

async fn wait_for_breaker(
    destination_id: &str,
    breaker_policy: &BreakerPolicy,
    state: &Arc<Mutex<DestinationState>>,
    telemetry: &Arc<dyn TelemetrySink>,
) {
    loop {
        let maybe_wait = {
            let mut state = state.lock().expect("destination status mutex poisoned");
            if state.status.breaker_state != BreakerState::Open {
                None
            } else if let Some(open_until) = state.open_until {
                if Instant::now() < open_until {
                    Some(open_until.saturating_duration_since(Instant::now()))
                } else {
                    state.status.breaker_state = BreakerState::HalfOpen;
                    state.open_until = None;
                    telemetry.emit(BrokerEvent::DestinationBreakerChanged {
                        destination_id: destination_id.to_owned(),
                        state: BreakerStateSnapshot::HalfOpen,
                        reason: "cooldown_elapsed".to_owned(),
                    });
                    None
                }
            } else {
                state.open_until = Some(Instant::now() + breaker_policy.cooldown);
                Some(breaker_policy.cooldown)
            }
        };

        match maybe_wait {
            Some(duration) => tokio::time::sleep(duration).await,
            None => break,
        }
    }
}

fn open_breaker(
    state: &mut DestinationState,
    destination_id: &str,
    breaker_policy: &BreakerPolicy,
    events: &mut Vec<BrokerEvent>,
    reason: &str,
) {
    state.status.breaker_state = BreakerState::Open;
    state.open_until = Some(Instant::now() + breaker_policy.cooldown);
    events.push(BrokerEvent::DestinationBreakerChanged {
        destination_id: destination_id.to_owned(),
        state: BreakerStateSnapshot::Open,
        reason: reason.to_owned(),
    });
}

async fn handle_health_http_connection(
    mut stream: TcpStream,
    reporter: Arc<dyn HealthReporter>,
) -> Result<(), io::Error> {
    let mut buffer = [0u8; 2048];
    let size = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (status, content_type, body) = match path {
        "/healthz" => ("200 OK", "text/plain; charset=utf-8", "ok\n".to_owned()),
        "/metrics" => (
            "200 OK",
            "text/plain; version=0.0.4; charset=utf-8",
            reporter.render_prometheus(),
        ),
        _ => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found\n".to_owned(),
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await
}
