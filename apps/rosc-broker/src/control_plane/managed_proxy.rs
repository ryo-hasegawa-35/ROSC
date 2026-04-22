use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    ManagedUdpProxy, ProxyOperatorDashboard, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverview, ProxyOperatorReport, ProxyOperatorSnapshot, UdpProxyStatusSnapshot,
};

use super::shared::{
    dispatch_result, ensure_destination_exists, ensure_route_exists, status_result,
};
use super::types::{ControlPlaneActionResult, ControlPlaneError, ProxyControlPlane};

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

    async fn operator_snapshot(&self, history_limit: Option<usize>) -> ProxyOperatorSnapshot {
        self.inner.lock().await.operator_snapshot(history_limit)
    }

    async fn operator_dashboard(&self, history_limit: Option<usize>) -> ProxyOperatorDashboard {
        self.inner.lock().await.operator_dashboard(history_limit)
    }

    async fn operator_diagnostics(&self, history_limit: Option<usize>) -> ProxyOperatorDiagnostics {
        self.inner.lock().await.operator_diagnostics(history_limit)
    }

    async fn operator_incidents(&self, history_limit: Option<usize>) -> ProxyOperatorIncidents {
        self.inner.lock().await.operator_incidents(history_limit)
    }

    async fn freeze_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        status_result(proxy.freeze_traffic(), proxy.status_snapshot())
    }

    async fn thaw_traffic(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        status_result(proxy.thaw_traffic(), proxy.status_snapshot())
    }

    async fn isolate_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        ensure_route_exists(proxy.has_route(route_id), route_id)?;
        Ok(status_result(
            proxy.isolate_route(route_id),
            proxy.status_snapshot(),
        ))
    }

    async fn restore_route(
        &self,
        route_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        ensure_route_exists(proxy.has_route(route_id), route_id)?;
        Ok(status_result(
            proxy.restore_route(route_id),
            proxy.status_snapshot(),
        ))
    }

    async fn rehydrate_destination(
        &self,
        destination_id: &str,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        ensure_destination_exists(proxy.has_destination(destination_id), destination_id)?;
        let dispatch_count = proxy
            .rehydrate_destination(destination_id)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        Ok(dispatch_result(dispatch_count, proxy.status_snapshot()))
    }

    async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<ControlPlaneActionResult, ControlPlaneError> {
        let proxy = self.inner.lock().await;
        ensure_route_exists(proxy.has_route(route_id), route_id)?;
        ensure_destination_exists(
            proxy.has_destination(sandbox_destination_id),
            sandbox_destination_id,
        )?;
        let dispatch_count = proxy
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
            .map_err(|error| ControlPlaneError::ActionFailed(error.to_string()))?;
        Ok(dispatch_result(dispatch_count, proxy.status_snapshot()))
    }

    async fn restore_all_routes(&self) -> ControlPlaneActionResult {
        let proxy = self.inner.lock().await;
        dispatch_result(proxy.restore_all_routes(), proxy.status_snapshot())
    }
}
