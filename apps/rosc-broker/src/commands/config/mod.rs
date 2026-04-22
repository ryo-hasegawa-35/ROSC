mod inspect;
mod maintenance;
mod services;

pub(crate) use inspect::{
    check_config, proxy_assert_ready, proxy_attention, proxy_board, proxy_brief, proxy_casebook,
    proxy_cockpit, proxy_diagnostics, proxy_dossier, proxy_focus, proxy_handoff, proxy_incidents,
    proxy_lens, proxy_mission, proxy_overview, proxy_readiness, proxy_runbook, proxy_snapshot,
    proxy_status, proxy_timeline, proxy_triage, proxy_workspace,
};
pub(crate) use maintenance::{diff_config, watch_config};
pub(crate) use services::serve_health;
