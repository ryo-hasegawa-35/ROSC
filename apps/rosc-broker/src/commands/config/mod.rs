mod inspect;
mod maintenance;
mod services;

pub(crate) use inspect::{
    check_config, proxy_assert_ready, proxy_attention, proxy_diagnostics, proxy_handoff,
    proxy_incidents, proxy_overview, proxy_readiness, proxy_snapshot, proxy_status,
};
pub(crate) use maintenance::{diff_config, watch_config};
pub(crate) use services::serve_health;
