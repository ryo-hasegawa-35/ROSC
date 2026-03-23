use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_config::{BrokerConfig, ConfigApplyResult, ConfigError, ConfigManager, DropPolicyConfig};
use rosc_recovery::{RecoveryEngine, RehydrateRequest, SandboxReplayRequest};
use rosc_route::{CachePolicy, CapturePolicy, TrafficClass, TransportSelector};
use rosc_runtime::{
    BreakerPolicy, DestinationPolicy, DestinationRegistry, DestinationWorkerHandle, DropPolicy,
    IngressQueue, QueuePolicy, Runtime, UdpEgressSink, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};
use serde::Serialize;

pub struct UdpProxyApp {
    runtime: Arc<Runtime<InMemoryTelemetry>>,
    recovery: Arc<RecoveryEngine<InMemoryTelemetry>>,
    destinations: Arc<DestinationRegistry>,
    ingresses: BTreeMap<String, UdpIngressBinding>,
    status: UdpProxyStatusSnapshot,
}

#[derive(Debug)]
pub enum ConfigReloadOutcome {
    Unchanged,
    Applied(ConfigApplyResult),
    Rejected(ConfigError),
}

pub struct ConfigFileSupervisor<TTelemetry> {
    path: PathBuf,
    manager: ConfigManager,
    telemetry: TTelemetry,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UdpProxyStatusSnapshot {
    pub ingresses: Vec<UdpProxyIngressStatus>,
    pub destinations: Vec<UdpProxyDestinationStatus>,
    pub routes: Vec<UdpProxyRouteStatus>,
    pub fallback_routes: Vec<UdpProxyFallbackStatus>,
    pub warnings: Vec<String>,
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

impl UdpProxyApp {
    pub async fn from_config(config: &BrokerConfig, telemetry: InMemoryTelemetry) -> Result<Self> {
        config.validate_runtime_references()?;

        let routing = rosc_route::RoutingEngine::new(config.routes.clone())?;
        let runtime = Arc::new(Runtime {
            routing,
            telemetry: telemetry.clone(),
        });
        let recovery = Arc::new(RecoveryEngine::new(telemetry.clone()));

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

        let ingress_addrs = ingresses
            .iter()
            .map(|(ingress_id, binding)| {
                binding
                    .local_addr()
                    .map(|addr| (ingress_id.clone(), addr))
                    .with_context(|| {
                        format!("failed to inspect local address for ingress `{ingress_id}`")
                    })
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut status = proxy_status_from_config(config)?;
        for ingress in &mut status.ingresses {
            ingress.bound_local_addr = ingress_addrs.get(&ingress.id).map(ToString::to_string);
        }

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
                        cooldown: std::time::Duration::from_millis(
                            destination.policy.breaker.cooldown_ms,
                        ),
                    },
                },
                sink,
                Arc::new(telemetry.clone()),
            ));
        }

        Ok(Self {
            runtime,
            recovery,
            destinations: Arc::new(destinations),
            ingresses,
            status,
        })
    }

    pub fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        self.status.clone()
    }

    pub fn ingress_local_addr(&self, ingress_id: &str) -> Option<SocketAddr> {
        self.ingresses
            .get(ingress_id)
            .and_then(|binding| binding.local_addr().ok())
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
            self.runtime.telemetry.emit(BrokerEvent::PacketDropped {
                ingress_id: failure.destination_id.clone(),
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

    pub async fn spawn_ingress_tasks(&mut self, ingress_queue_depth: usize) {
        let (queue, mut rx) = IngressQueue::new(QueuePolicy {
            max_depth: ingress_queue_depth,
        });

        let runtime = Arc::clone(&self.runtime);
        let recovery = Arc::clone(&self.recovery);
        let destinations = Arc::clone(&self.destinations);
        tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                let outcome = runtime.dispatch_packet(&packet, &destinations).await;
                recovery.observe_dispatches(&outcome.successful_dispatches);
                for failure in outcome.failures {
                    runtime.telemetry.emit(BrokerEvent::PacketDropped {
                        ingress_id: failure.destination_id,
                        reason: failure.reason,
                    });
                }
            }
        });

        for (ingress_id, binding) in std::mem::take(&mut self.ingresses) {
            let queue = queue.clone();
            let telemetry = self.runtime.telemetry.clone();
            tokio::spawn(async move {
                loop {
                    match binding.recv_next().await {
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
            });
        }
    }
}

impl<TTelemetry> ConfigFileSupervisor<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn new(path: impl Into<PathBuf>, telemetry: TTelemetry) -> Self {
        Self {
            path: path.into(),
            manager: ConfigManager::default(),
            telemetry,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn current_revision(&self) -> Option<u64> {
        self.manager.current().map(|applied| applied.revision)
    }

    pub fn load_initial(&mut self) -> Result<ConfigApplyResult> {
        let content = read_config_file(&self.path)?;
        let applied = self.manager.apply_toml_str(&content)?;
        self.emit_config_applied(&applied);
        Ok(applied)
    }

    pub fn poll_once(&mut self) -> Result<ConfigReloadOutcome> {
        let content = read_config_file(&self.path)?;
        if self
            .manager
            .current()
            .map(|current| current.raw_toml == content)
            .unwrap_or(false)
        {
            return Ok(ConfigReloadOutcome::Unchanged);
        }

        match self.manager.apply_toml_str(&content) {
            Ok(applied) => {
                self.emit_config_applied(&applied);
                Ok(ConfigReloadOutcome::Applied(applied))
            }
            Err(error) => {
                self.telemetry.emit(BrokerEvent::ConfigRejected);
                Ok(ConfigReloadOutcome::Rejected(error))
            }
        }
    }

    fn emit_config_applied(&self, applied: &ConfigApplyResult) {
        self.telemetry.emit(BrokerEvent::ConfigApplied {
            revision: applied.revision,
            added_ingresses: applied.diff.added_ingresses.len(),
            removed_ingresses: applied.diff.removed_ingresses.len(),
            changed_ingresses: applied.diff.changed_ingresses.len(),
            added_destinations: applied.diff.added_destinations.len(),
            removed_destinations: applied.diff.removed_destinations.len(),
            changed_destinations: applied.diff.changed_destinations.len(),
            added_routes: applied.diff.added_routes.len(),
            removed_routes: applied.diff.removed_routes.len(),
            changed_routes: applied.diff.changed_routes.len(),
        });
    }
}

pub fn proxy_status_from_config(config: &BrokerConfig) -> Result<UdpProxyStatusSnapshot> {
    config.validate_runtime_references()?;

    let route_ids_by_ingress = config
        .udp_ingresses
        .iter()
        .map(|ingress| {
            let route_ids = config
                .routes
                .iter()
                .filter(|route| {
                    route.match_spec.ingress_ids.is_empty()
                        || route
                            .match_spec
                            .ingress_ids
                            .iter()
                            .any(|id| id == &ingress.id)
                })
                .map(|route| route.id.clone())
                .collect::<Vec<_>>();
            (ingress.id.clone(), route_ids)
        })
        .collect::<BTreeMap<_, _>>();

    let route_ids_by_destination = config
        .udp_destinations
        .iter()
        .map(|destination| {
            let route_ids = config
                .routes
                .iter()
                .filter(|route| {
                    route.destinations.iter().any(|route_destination| {
                        route_destination.transport == TransportSelector::OscUdp
                            && route_destination.target == destination.id
                    })
                })
                .map(|route| route.id.clone())
                .collect::<Vec<_>>();
            (destination.id.clone(), route_ids)
        })
        .collect::<BTreeMap<_, _>>();

    let ingresses = config
        .udp_ingresses
        .iter()
        .map(|ingress| UdpProxyIngressStatus {
            id: ingress.id.clone(),
            configured_bind: ingress.bind.clone(),
            bound_local_addr: None,
            route_ids: route_ids_by_ingress
                .get(&ingress.id)
                .cloned()
                .unwrap_or_default(),
            max_packet_size: ingress.max_packet_size,
            mode: ingress.mode,
        })
        .collect::<Vec<_>>();

    let destinations = config
        .udp_destinations
        .iter()
        .map(|destination| UdpProxyDestinationStatus {
            id: destination.id.clone(),
            bind: destination.bind.clone(),
            target: destination.target.clone(),
            route_ids: route_ids_by_destination
                .get(&destination.id)
                .cloned()
                .unwrap_or_default(),
            queue_depth: destination.policy.queue_depth,
            drop_policy: destination.policy.drop_policy,
            breaker_open_after_consecutive_failures: destination
                .policy
                .breaker
                .open_after_consecutive_failures,
            breaker_open_after_consecutive_queue_overflows: destination
                .policy
                .breaker
                .open_after_consecutive_queue_overflows,
            breaker_cooldown_ms: destination.policy.breaker.cooldown_ms,
        })
        .collect::<Vec<_>>();

    let destination_targets = config
        .udp_destinations
        .iter()
        .map(|destination| (destination.id.as_str(), destination.target.as_str()))
        .collect::<BTreeMap<_, _>>();

    let routes = config
        .routes
        .iter()
        .map(|route| UdpProxyRouteStatus {
            id: route.id.clone(),
            enabled: route.enabled,
            mode: route.mode,
            traffic_class: route.class.clone(),
            ingress_ids: route.match_spec.ingress_ids.clone(),
            address_patterns: route.match_spec.address_patterns.clone(),
            destination_ids: route
                .destinations
                .iter()
                .map(|destination| destination.target.clone())
                .collect(),
            rename_address: route.transform.rename_address.clone(),
            cache_policy: route.cache.policy,
            capture_policy: route.observability.capture,
            rehydrate_on_connect: route.recovery.rehydrate_on_connect,
            replay_allowed: route.recovery.replay_allowed,
        })
        .collect::<Vec<_>>();

    let fallback_routes = config
        .routes
        .iter()
        .map(|route| {
            let direct_udp_targets = route
                .destinations
                .iter()
                .filter(|destination| destination.transport == TransportSelector::OscUdp)
                .filter_map(|destination| destination_targets.get(destination.target.as_str()))
                .map(|target| (*target).to_owned())
                .collect::<Vec<_>>();
            let available = !direct_udp_targets.is_empty();
            let note = if available {
                "If ROSC sidecar mode is unavailable, point the sender directly at these UDP targets.".to_owned()
            } else {
                "No direct UDP fallback target is available for this route yet.".to_owned()
            };

            UdpProxyFallbackStatus {
                route_id: route.id.clone(),
                direct_udp_targets,
                available,
                note,
            }
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    for route in &config.routes {
        if route.match_spec.ingress_ids.is_empty() {
            warnings.push(format!(
                "route `{}` matches all ingresses; proxy sidecar scope is broad",
                route.id
            ));
        }
        if route.match_spec.address_patterns.is_empty() {
            warnings.push(format!(
                "route `{}` matches all addresses; fallback analysis is broad",
                route.id
            ));
        }
    }

    Ok(UdpProxyStatusSnapshot {
        ingresses,
        destinations,
        routes,
        fallback_routes,
        warnings,
    })
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

fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to read config file {}", path.display()))
}
