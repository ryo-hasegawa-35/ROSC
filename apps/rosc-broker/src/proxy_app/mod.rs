mod build;
mod control;
mod dispatch;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use rosc_config::BrokerConfig;
use rosc_packet::PacketEnvelope;
use rosc_recovery::RecoveryEngine;
use rosc_runtime::{DestinationRegistry, Runtime, UdpIngressBinding, UdpIngressConfig};
use rosc_telemetry::{HealthSnapshot, InMemoryTelemetry};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::ProxyLaunchProfileStatus;
use crate::proxy_status::{UdpProxyStatusSnapshot, attach_runtime_status};
use crate::route_control::RouteControlState;
use crate::traffic_control::TrafficControlState;

pub struct UdpProxyApp {
    config: BrokerConfig,
    runtime: Arc<Runtime<InMemoryTelemetry>>,
    recovery: Arc<RecoveryEngine<InMemoryTelemetry>>,
    destinations: Arc<DestinationRegistry>,
    traffic_control: TrafficControlState,
    route_control: RouteControlState,
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
        build::build_proxy_app(config, telemetry).await
    }

    pub fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        attach_runtime_status(self.status.clone(), &self.telemetry_snapshot())
    }

    pub fn apply_launch_profile(&mut self, profile: ProxyLaunchProfileStatus) {
        self.status.launch_profile = profile;
    }

    pub fn emit_launch_profile_event(&self, revision: u64) {
        control::emit_launch_profile_event(self, revision);
    }

    pub fn telemetry_snapshot(&self) -> HealthSnapshot {
        self.runtime.telemetry.snapshot()
    }

    pub fn freeze_traffic(&self) -> bool {
        control::freeze_traffic(self)
    }

    pub fn thaw_traffic(&self) -> bool {
        control::thaw_traffic(self)
    }

    pub fn restore_frozen_traffic(&self) -> bool {
        control::restore_frozen_traffic(self)
    }

    pub fn is_traffic_frozen(&self) -> bool {
        self.traffic_control.is_frozen()
    }

    pub fn has_route(&self, route_id: &str) -> bool {
        self.status.routes.iter().any(|route| route.id == route_id)
    }

    pub fn has_destination(&self, destination_id: &str) -> bool {
        self.status
            .destinations
            .iter()
            .any(|destination| destination.id == destination_id)
    }

    pub fn isolate_route(&self, route_id: &str) -> bool {
        control::isolate_route(self, route_id)
    }

    pub fn restore_route(&self, route_id: &str) -> bool {
        control::restore_route(self, route_id)
    }

    pub fn restore_all_routes(&self) -> usize {
        control::restore_all_routes(self)
    }

    pub fn restore_route_isolation(&self, route_id: &str) -> bool {
        control::restore_route_isolation(self, route_id)
    }

    pub fn isolated_routes(&self) -> Vec<String> {
        self.route_control.snapshot()
    }

    pub fn ingress_local_addr(&self, ingress_id: &str) -> Option<SocketAddr> {
        self.ingress_addrs.get(ingress_id).copied()
    }

    pub async fn relay_once(&self, ingress_id: &str) -> Result<usize> {
        control::relay_once(self, ingress_id).await
    }

    pub async fn rehydrate_destination(&self, destination_id: &str) -> Result<usize> {
        control::rehydrate_destination(self, destination_id).await
    }

    pub async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<usize> {
        control::replay_route_to_sandbox(self, route_id, sandbox_destination_id, limit).await
    }

    pub async fn spawn_ingress_tasks(&mut self, ingress_queue_depth: usize) -> Result<()> {
        dispatch::spawn_ingress_tasks(self, ingress_queue_depth).await
    }

    pub async fn shutdown(&mut self) {
        self.tasks.shutdown().await;
        self.destinations.shutdown().await;
        self.destinations = Arc::new(DestinationRegistry::default());
    }

    async fn ensure_ingresses_bound(&mut self) -> Result<()> {
        dispatch::ensure_ingresses_bound(self).await
    }

    async fn ensure_destinations_bound(&mut self) -> Result<()> {
        dispatch::ensure_destinations_bound(self).await
    }

    async fn dispatch_packet(&self, packet: &PacketEnvelope) -> rosc_runtime::DispatchOutcome {
        control::dispatch_packet(self, packet).await
    }
}
