use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_config::{BrokerConfig, DropPolicyConfig};
use rosc_recovery::{RecoveryEngine, RehydrateRequest, SandboxReplayRequest};
use rosc_runtime::{
    BreakerPolicy, DestinationPolicy, DestinationRegistry, DestinationWorkerHandle, DropPolicy,
    IngressQueue, QueuePolicy, Runtime, UdpEgressSink, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::{BrokerEvent, HealthSnapshot, InMemoryTelemetry, TelemetrySink};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::proxy_status::{
    UdpProxyStatusSnapshot, attach_runtime_status, proxy_status_from_config,
};

pub struct UdpProxyApp {
    runtime: Arc<Runtime<InMemoryTelemetry>>,
    recovery: Arc<RecoveryEngine<InMemoryTelemetry>>,
    destinations: Arc<DestinationRegistry>,
    ingress_specs: BTreeMap<String, IngressBindingSpec>,
    ingress_addrs: BTreeMap<String, SocketAddr>,
    ingresses: BTreeMap<String, UdpIngressBinding>,
    status: UdpProxyStatusSnapshot,
    tasks: ProxyRuntimeTasks,
}

#[derive(Clone)]
struct IngressBindingSpec {
    bind_address: String,
    config: UdpIngressConfig,
}

#[derive(Default)]
struct ProxyRuntimeTasks {
    shutdown: Option<watch::Sender<bool>>,
    dispatcher: Option<JoinHandle<()>>,
    ingresses: Vec<JoinHandle<()>>,
}

impl ProxyRuntimeTasks {
    fn is_running(&self) -> bool {
        self.shutdown.is_some() || self.dispatcher.is_some() || !self.ingresses.is_empty()
    }

    async fn shutdown(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }

        if let Some(dispatcher) = self.dispatcher.take() {
            let _ = dispatcher.await;
        }

        for handle in self.ingresses.drain(..) {
            let _ = handle.await;
        }
    }
}

impl Drop for ProxyRuntimeTasks {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
    }
}

impl UdpProxyApp {
    pub async fn from_config(config: &BrokerConfig, telemetry: InMemoryTelemetry) -> Result<Self> {
        config.validate_runtime_references()?;

        let routing = rosc_route::RoutingEngine::new(config.routes.clone())?;
        let runtime = Arc::new(Runtime {
            routing,
            telemetry: telemetry.clone(),
        });
        let recovery = Arc::new(RecoveryEngine::new(telemetry.clone()));

        let ingresses = bind_ingresses(config).await?;
        let ingress_addrs = ingress_addresses(&ingresses)?;
        let ingress_specs = ingress_specs(&config.udp_ingresses, &ingress_addrs);
        let mut status = proxy_status_from_config(config)?;
        for ingress in &mut status.ingresses {
            ingress.bound_local_addr = ingress_addrs.get(&ingress.id).map(ToString::to_string);
        }

        let destinations = build_destinations(config, &ingress_addrs, telemetry.clone()).await?;

        Ok(Self {
            runtime,
            recovery,
            destinations: Arc::new(destinations),
            ingress_specs,
            ingress_addrs,
            ingresses,
            status,
            tasks: ProxyRuntimeTasks::default(),
        })
    }

    pub fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        attach_runtime_status(self.status.clone(), &self.telemetry_snapshot())
    }

    pub fn telemetry_snapshot(&self) -> HealthSnapshot {
        self.runtime.telemetry.snapshot()
    }

    pub fn ingress_local_addr(&self, ingress_id: &str) -> Option<SocketAddr> {
        self.ingress_addrs.get(ingress_id).copied()
    }

    pub async fn relay_once(&self, ingress_id: &str) -> Result<usize> {
        let binding = self
            .ingresses
            .get(ingress_id)
            .with_context(|| format!("unknown ingress id {ingress_id}"))?;
        let packet = binding.recv_next().await?;
        self.runtime.telemetry.emit(BrokerEvent::PacketAccepted {
            ingress_id: packet.metadata.ingress_id.clone(),
        });
        let outcome = self
            .runtime
            .dispatch_packet(&packet, &self.destinations)
            .await;
        self.recovery
            .observe_dispatches(&outcome.successful_dispatches);
        for failure in &outcome.failures {
            self.runtime.telemetry.emit(BrokerEvent::DispatchFailed {
                route_id: failure.route_id.clone(),
                destination_id: failure.destination_id.clone(),
                reason: failure.reason.clone(),
            });
        }
        Ok(outcome.dispatched)
    }

    pub async fn rehydrate_destination(&self, destination_id: &str) -> Result<usize> {
        let outcome = self.recovery.rehydrate(RehydrateRequest {
            route_id: None,
            destination_id: Some(destination_id.to_owned()),
        })?;

        let mut dispatched = 0usize;
        for dispatch in outcome.dispatches {
            if self.destinations.dispatch(dispatch).await.is_ok() {
                dispatched += 1;
            }
        }

        Ok(dispatched)
    }

    pub async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<usize> {
        let outcome = self.recovery.sandbox_replay(SandboxReplayRequest {
            route_id: route_id.to_owned(),
            source_destination_id: None,
            sandbox_destination_id: sandbox_destination_id.to_owned(),
            limit,
        })?;

        let mut dispatched = 0usize;
        for dispatch in outcome.dispatches {
            if self.destinations.dispatch(dispatch).await.is_ok() {
                dispatched += 1;
            }
        }

        Ok(dispatched)
    }

    pub async fn spawn_ingress_tasks(&mut self, ingress_queue_depth: usize) -> Result<()> {
        if self.tasks.is_running() {
            anyhow::bail!("udp proxy ingress tasks are already running");
        }
        self.ensure_ingresses_bound().await?;

        let (queue, mut rx) = IngressQueue::new(QueuePolicy {
            max_depth: ingress_queue_depth,
        });
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let runtime = Arc::clone(&self.runtime);
        let recovery = Arc::clone(&self.recovery);
        let destinations = Arc::clone(&self.destinations);
        let mut dispatcher_shutdown = shutdown_rx.clone();
        self.tasks.dispatcher = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = dispatcher_shutdown.changed() => {
                        break;
                    }
                    packet = rx.recv() => {
                        let Some(packet) = packet else {
                            break;
                        };
                        let outcome = runtime.dispatch_packet(&packet, &destinations).await;
                        recovery.observe_dispatches(&outcome.successful_dispatches);
                        for failure in outcome.failures {
                            runtime.telemetry.emit(BrokerEvent::DispatchFailed {
                                route_id: failure.route_id,
                                destination_id: failure.destination_id,
                                reason: failure.reason,
                            });
                        }
                    }
                }
            }
        }));

        for (ingress_id, binding) in std::mem::take(&mut self.ingresses) {
            let queue = queue.clone();
            let telemetry = self.runtime.telemetry.clone();
            let mut ingress_shutdown = shutdown_rx.clone();
            self.tasks.ingresses.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        biased;
                        _ = ingress_shutdown.changed() => {
                            break;
                        }
                        packet = binding.recv_next() => {
                            match packet {
                                Ok(packet) => {
                                    telemetry.emit(BrokerEvent::PacketAccepted {
                                        ingress_id: packet.metadata.ingress_id.clone(),
                                    });
                                    match queue.try_send(packet) {
                                        Ok(()) => {}
                                        Err(error) => telemetry.emit(BrokerEvent::PacketDropped {
                                            ingress_id: ingress_id.clone(),
                                            reason: error.to_string(),
                                        }),
                                    }
                                }
                                Err(error) => {
                                    telemetry.emit(BrokerEvent::PacketDropped {
                                        ingress_id: ingress_id.clone(),
                                        reason: error.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }));
        }

        self.tasks.shutdown = Some(shutdown_tx);
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.tasks.shutdown().await;
    }

    async fn ensure_ingresses_bound(&mut self) -> Result<()> {
        if !self.ingresses.is_empty() {
            return Ok(());
        }

        self.ingresses = bind_ingresses_from_specs(&self.ingress_specs).await?;
        self.ingress_addrs = ingress_addresses(&self.ingresses)?;
        refresh_ingress_status(&mut self.status, &self.ingress_addrs);
        Ok(())
    }
}

async fn bind_ingresses(config: &BrokerConfig) -> Result<BTreeMap<String, UdpIngressBinding>> {
    let mut ingresses = BTreeMap::new();
    for ingress in &config.udp_ingresses {
        let binding = UdpIngressBinding::bind(
            &ingress.bind,
            UdpIngressConfig {
                ingress_id: ingress.id.clone(),
                compatibility_mode: ingress.mode,
                max_packet_size: ingress.max_packet_size,
            },
        )
        .await?;
        ingresses.insert(ingress.id.clone(), binding);
    }
    Ok(ingresses)
}

async fn bind_ingresses_from_specs(
    specs: &BTreeMap<String, IngressBindingSpec>,
) -> Result<BTreeMap<String, UdpIngressBinding>> {
    let mut ingresses = BTreeMap::new();
    for (ingress_id, spec) in specs {
        let binding = UdpIngressBinding::bind(&spec.bind_address, spec.config.clone()).await?;
        ingresses.insert(ingress_id.clone(), binding);
    }
    Ok(ingresses)
}

fn ingress_specs(
    ingresses: &[rosc_config::UdpIngressConfig],
    ingress_addrs: &BTreeMap<String, SocketAddr>,
) -> BTreeMap<String, IngressBindingSpec> {
    ingresses
        .iter()
        .map(|ingress| {
            let bind_address = ingress_addrs
                .get(&ingress.id)
                .map(ToString::to_string)
                .unwrap_or_else(|| ingress.bind.clone());
            (
                ingress.id.clone(),
                IngressBindingSpec {
                    bind_address,
                    config: UdpIngressConfig {
                        ingress_id: ingress.id.clone(),
                        compatibility_mode: ingress.mode,
                        max_packet_size: ingress.max_packet_size,
                    },
                },
            )
        })
        .collect()
}

fn ingress_addresses(
    ingresses: &BTreeMap<String, UdpIngressBinding>,
) -> Result<BTreeMap<String, SocketAddr>> {
    ingresses
        .iter()
        .map(|(ingress_id, binding)| {
            binding
                .local_addr()
                .map(|addr| (ingress_id.clone(), addr))
                .with_context(|| {
                    format!("failed to inspect local address for ingress `{ingress_id}`")
                })
        })
        .collect()
}

fn refresh_ingress_status(
    status: &mut UdpProxyStatusSnapshot,
    ingress_addrs: &BTreeMap<String, SocketAddr>,
) {
    for ingress in &mut status.ingresses {
        ingress.bound_local_addr = ingress_addrs.get(&ingress.id).map(ToString::to_string);
    }
}

async fn build_destinations(
    config: &BrokerConfig,
    ingress_addrs: &BTreeMap<String, SocketAddr>,
    telemetry: InMemoryTelemetry,
) -> Result<DestinationRegistry> {
    let mut destinations = DestinationRegistry::default();

    for destination in &config.udp_destinations {
        let target: SocketAddr = destination
            .target
            .parse()
            .with_context(|| format!("invalid udp target address {}", destination.target))?;
        if let Some((ingress_id, ingress_addr)) = ingress_addrs
            .iter()
            .find(|(_, ingress_addr)| ingress_receives_target(**ingress_addr, target))
        {
            anyhow::bail!(
                "udp destination `{}` targets ingress `{}` at {}; refusing proxy self-loop",
                destination.id,
                ingress_id,
                ingress_addr
            );
        }

        let sink = Arc::new(UdpEgressSink::bind(&destination.bind, target).await?);
        destinations.register(DestinationWorkerHandle::spawn(
            destination.id.clone(),
            destination_policy(destination),
            sink,
            Arc::new(telemetry.clone()),
        ));
    }

    Ok(destinations)
}

fn destination_policy(destination: &rosc_config::UdpDestinationConfig) -> DestinationPolicy {
    DestinationPolicy {
        queue_depth: destination.policy.queue_depth,
        drop_policy: match destination.policy.drop_policy {
            DropPolicyConfig::DropNewest => DropPolicy::DropNewest,
            DropPolicyConfig::DropOldest => DropPolicy::DropOldest,
        },
        breaker: BreakerPolicy {
            open_after_consecutive_failures: destination
                .policy
                .breaker
                .open_after_consecutive_failures,
            open_after_consecutive_queue_overflows: destination
                .policy
                .breaker
                .open_after_consecutive_queue_overflows,
            cooldown: std::time::Duration::from_millis(destination.policy.breaker.cooldown_ms),
        },
    }
}

fn ingress_receives_target(ingress_addr: SocketAddr, target: SocketAddr) -> bool {
    if ingress_addr.port() != target.port() {
        return false;
    }

    if ingress_addr.ip() == target.ip() {
        return true;
    }

    match (ingress_addr.ip(), target.ip()) {
        (IpAddr::V4(ingress_ip), IpAddr::V4(target_ip)) => {
            ingress_ip.is_unspecified() && (target_ip.is_loopback() || target_ip.is_unspecified())
        }
        (IpAddr::V6(ingress_ip), IpAddr::V6(target_ip)) => {
            ingress_ip.is_unspecified() && (target_ip.is_loopback() || target_ip.is_unspecified())
        }
        _ => false,
    }
}
