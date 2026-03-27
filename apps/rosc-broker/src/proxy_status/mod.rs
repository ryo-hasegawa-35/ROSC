mod build;
mod runtime;
mod types;

pub use build::proxy_status_from_config;
pub use runtime::{attach_runtime_status, operator_warnings, startup_blockers};
pub use types::{
    UdpProxyDestinationRuntimeStatus, UdpProxyDestinationStatus, UdpProxyFallbackStatus,
    UdpProxyIngressStatus, UdpProxyRouteAssessment, UdpProxyRouteStatus, UdpProxyRuntimeStatus,
    UdpProxyStatusSnapshot, UdpProxySummary,
};
