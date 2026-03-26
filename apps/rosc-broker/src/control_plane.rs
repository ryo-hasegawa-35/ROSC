use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{ManagedProxyFileSupervisor, ManagedUdpProxy, UdpProxyStatusSnapshot};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlPlaneActionResult {
    pub applied: bool,
    pub status: UdpProxyStatusSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlPlaneError {
    UnknownRoute(String),
}

impl std::fmt::Display for ControlPlaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRoute(route_id) => write!(f, "unknown route `{route_id}`"),
        }
    }
}

impl std::error::Error for ControlPlaneError {}

#[async_trait]
pub trait ProxyControlPlane: Send + Sync + 'static {
    async fn status_snapshot(&self) -> UdpProxyStatusSnapshot;
    async fn freeze_traffic(&self) -> ControlPlaneActionResult;
    async fn thaw_traffic(&self) -> ControlPlaneActionResult;
    async fn isolate_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError>;
    async fn restore_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError>;
}

#[derive(Clone)]
pub struct ManagedUdpProxyController {
    inner: Arc<Mutex<ManagedUdpProxy>>,
}

impl ManagedUdpProxyController {
    pub fn new(inner: Arc<Mutex<ManagedUdpProxy>>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ProxyControlPlane for ManagedUdpProxyController {
    async fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        self.inner.lock().await.status_snapshot()
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        let applied = proxy.freeze_traffic();
        let status = proxy.status_snapshot();
        ControlPlaneActionResult { applied, status }
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        let applied = proxy.thaw_traffic();
        let status = proxy.status_snapshot();
        ControlPlaneActionResult { applied, status }
    }

    async fn isolate_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        if !proxy.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        let applied = proxy.isolate_route(route_id);
        let status = proxy.status_snapshot();
        Ok(ControlPlaneActionResult { applied, status })
    }

    async fn restore_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        if !proxy.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        let applied = proxy.restore_route(route_id);
        let status = proxy.status_snapshot();
        Ok(ControlPlaneActionResult { applied, status })
    }
}

#[derive(Clone)]
pub struct ManagedProxyFileSupervisorController {
    inner: Arc<Mutex<ManagedProxyFileSupervisor>>,
}

impl ManagedProxyFileSupervisorController {
    pub fn new(inner: Arc<Mutex<ManagedProxyFileSupervisor>>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ProxyControlPlane for ManagedProxyFileSupervisorController {
    async fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        self.inner.lock().await.status_snapshot()
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        let applied = supervisor.freeze_traffic();
        let status = supervisor.status_snapshot();
        ControlPlaneActionResult { applied, status }
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        let applied = supervisor.thaw_traffic();
        let status = supervisor.status_snapshot();
        ControlPlaneActionResult { applied, status }
    }

    async fn isolate_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        if !supervisor.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        let applied = supervisor.isolate_route(route_id);
        let status = supervisor.status_snapshot();
        Ok(ControlPlaneActionResult { applied, status })
    }

    async fn restore_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        if !supervisor.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        let applied = supervisor.restore_route(route_id);
        let status = supervisor.status_snapshot();
        Ok(ControlPlaneActionResult { applied, status })
    }
}
