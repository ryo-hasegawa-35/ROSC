mod inspect;
mod maintenance;
mod services;

pub(crate) use inspect::{
    check_config, proxy_assert_ready, proxy_attention, proxy_board, proxy_brief, proxy_casebook,
    proxy_diagnostics, proxy_dossier, proxy_focus, proxy_handoff, proxy_incidents, proxy_lens,
    proxy_overview, proxy_readiness, proxy_snapshot, proxy_status, proxy_timeline, proxy_triage,
};
pub(crate) use maintenance::{diff_config, watch_config};
pub(crate) use services::serve_health;
