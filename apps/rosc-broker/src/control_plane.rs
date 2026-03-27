use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    ManagedProxyFileSupervisor, ManagedUdpProxy, ProxyOperatorOverview, ProxyOperatorReport,
    UdpProxyStatusSnapshot,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlPlaneActionResult {
    pub applied: bool,
    pub dispatch_count: Option<usize>,
    pub status: UdpProxyStatusSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlPlaneError {
    UnknownRoute(String),
    UnknownDestination(String),
    ActionFailed(String),
}

impl std::fmt::Display for ControlPlaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRoute(route_id) => write!(f, "unknown route `{route_id}`"),
            Self::UnknownDestination(destination_id) => {
                write!(f, "unknown destination `{destination_id}`")
            }
            Self::ActionFailed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ControlPlaneError {}

#[async_trait]
pub trait ProxyControlPlane: Send + Sync + 'static {
    async fn status_snapshot(&self) -> UdpProxyStatusSnapshot;
    async fn operator_report(&self) -> ProxyOperatorReport;
    async fn operator_overview(&self) -> ProxyOperatorOverview;
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
    async fn rehydrate_destination(
        &self,
        destination_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError>;
    async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError>;
    async fn restore_all_routes(&self) -> ControlPlaneActionResult;
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

    async fn operator_report(&self) -> ProxyOperatorReport {
        self.inner.lock().await.operator_report()
    }

    async fn operator_overview(&self) -> ProxyOperatorOverview {
        self.inner.lock().await.operator_overview()
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        let applied = proxy.freeze_traffic();
        let status = proxy.status_snapshot();
        ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        }
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        let applied = proxy.thaw_traffic();
        let status = proxy.status_snapshot();
        ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        }
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
        Ok(ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        })
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
        Ok(ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        })
    }

    async fn rehydrate_destination(
        &self,
        destination_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        if !proxy.has_destination(destination_id) {
            return Err(ControlPlaneError::UnknownDestination(
                destination_id.to_owned(),
            ));
        }
        let dispatch_count = proxy
            .rehydrate_destination(destination_id)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        let status = proxy.status_snapshot();
        Ok(ControlPlaneActionResult {
            applied: dispatch_count > 0,
            dispatch_count: Some(dispatch_count),
            status,
        })
    }

    async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        if !proxy.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        if !proxy.has_destination(sandbox_destination_id) {
            return Err(ControlPlaneError::UnknownDestination(
                sandbox_destination_id.to_owned(),
            ));
        }
        let dispatch_count = proxy
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        let status = proxy.status_snapshot();
        Ok(ControlPlaneActionResult {
            applied: dispatch_count > 0,
            dispatch_count: Some(dispatch_count),
            status,
        })
    }

    async fn restore_all_routes(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        let restored = proxy.restore_all_routes();
        let status = proxy.status_snapshot();
        ControlPlaneActionResult {
            applied: restored > 0,
            dispatch_count: Some(restored),
            status,
        }
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

    async fn operator_report(&self) -> ProxyOperatorReport {
        self.inner.lock().await.operator_report()
    }

    async fn operator_overview(&self) -> ProxyOperatorOverview {
        self.inner.lock().await.operator_overview()
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        let applied = supervisor.freeze_traffic();
        let status = supervisor.status_snapshot();
        ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        }
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        let applied = supervisor.thaw_traffic();
        let status = supervisor.status_snapshot();
        ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        }
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
        Ok(ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        })
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
        Ok(ControlPlaneActionResult {
            applied,
            dispatch_count: None,
            status,
        })
    }

    async fn rehydrate_destination(
        &self,
        destination_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        if !supervisor.has_destination(destination_id) {
            return Err(ControlPlaneError::UnknownDestination(
                destination_id.to_owned(),
            ));
        }
        let dispatch_count = supervisor
            .rehydrate_destination(destination_id)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        let status = supervisor.status_snapshot();
        Ok(ControlPlaneActionResult {
            applied: dispatch_count > 0,
            dispatch_count: Some(dispatch_count),
            status,
        })
    }

    async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        if !supervisor.has_route(route_id) {
            return Err(ControlPlaneError::UnknownRoute(route_id.to_owned()));
        }
        if !supervisor.has_destination(sandbox_destination_id) {
            return Err(ControlPlaneError::UnknownDestination(
                sandbox_destination_id.to_owned(),
            ));
        }
        let dispatch_count = supervisor
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        let status = supervisor.status_snapshot();
        Ok(ControlPlaneActionResult {
            applied: dispatch_count > 0,
            dispatch_count: Some(dispatch_count),
            status,
        })
    }

    async fn restore_all_routes(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        let restored = supervisor.restore_all_routes();
        let status = supervisor.status_snapshot();
        ControlPlaneActionResult {
            applied: restored > 0,
            dispatch_count: Some(restored),
            status,
        }
    }
}
