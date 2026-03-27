use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    ManagedProxyFileSupervisor, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverview, ProxyOperatorReport, UdpProxyStatusSnapshot,
};

use super::shared::{
    dispatch_result, ensure_destination_exists, ensure_route_exists, status_result,
};
use super::types::{ControlPlaneActionResult, ControlPlaneError, ProxyControlPlane};

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

    async fn operator_diagnostics(&self, history_limit: Option<usize>) -> ProxyOperatorDiagnostics {
        self.inner.lock().await.operator_diagnostics(history_limit)
    }

    async fn operator_incidents(&self, history_limit: Option<usize>) -> ProxyOperatorIncidents {
        self.inner.lock().await.operator_incidents(history_limit)
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        status_result(supervisor.freeze_traffic(), supervisor.status_snapshot())
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        status_result(supervisor.thaw_traffic(), supervisor.status_snapshot())
    }

    async fn isolate_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        ensure_route_exists(supervisor.has_route(route_id), route_id)?;
        Ok(status_result(
            supervisor.isolate_route(route_id),
            supervisor.status_snapshot(),
        ))
    }

    async fn restore_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        ensure_route_exists(supervisor.has_route(route_id), route_id)?;
        Ok(status_result(
            supervisor.restore_route(route_id),
            supervisor.status_snapshot(),
        ))
    }

    async fn rehydrate_destination(
        &self,
        destination_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        ensure_destination_exists(supervisor.has_destination(destination_id), destination_id)?;
        let dispatch_count = supervisor
            .rehydrate_destination(destination_id)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        Ok(dispatch_result(
            dispatch_count,
            supervisor.status_snapshot(),
        ))
    }

    async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let supervisor = self.inner.lock().await;
        ensure_route_exists(supervisor.has_route(route_id), route_id)?;
        ensure_destination_exists(
            supervisor.has_destination(sandbox_destination_id),
            sandbox_destination_id,
        )?;
        let dispatch_count = supervisor
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        Ok(dispatch_result(
            dispatch_count,
            supervisor.status_snapshot(),
        ))
    }

    async fn restore_all_routes(&self) -> ControlPlaneActionResult {
        let supervisor = self.inner.lock().await;
        dispatch_result(
            supervisor.restore_all_routes(),
            supervisor.status_snapshot(),
        )
    }
}
