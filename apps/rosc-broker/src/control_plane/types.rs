use async_trait::async_trait;
use serde::Serialize;

use crate::{
    ProxyOperatorDashboard, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverview, ProxyOperatorReport, ProxyOperatorSnapshot, UdpProxyStatusSnapshot,
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
    async fn operator_snapshot(&self, history_limit: Option<usize>) -> ProxyOperatorSnapshot;
    async fn operator_dashboard(&self, history_limit: Option<usize>) -> ProxyOperatorDashboard;
    async fn operator_diagnostics(&self, history_limit: Option<usize>) -> ProxyOperatorDiagnostics;
    async fn operator_incidents(&self, history_limit: Option<usize>) -> ProxyOperatorIncidents;
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
