mod common;
mod config;
mod runtime;

use anyhow::Result;

use crate::cli::Command;

#[derive(Clone, Copy)]
pub(crate) struct ProxyCommandOptions {
    pub fail_on_warnings: bool,
    pub require_fallback_ready: bool,
    pub safe_mode: bool,
    pub start_frozen: bool,
}

pub async fn run(command: Box<Command>) -> Result<()> {
    match *command {
        Command::CheckConfig { path } => config::check_config(&path).await,
        Command::ProxyStatus {
            config,
            resolve_bindings,
            safe_mode,
        } => config::proxy_status(&config, resolve_bindings, safe_mode).await,
        Command::ProxyOverview {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
        } => {
            config::proxy_overview(
                &config,
                resolve_bindings,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyReadiness {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
        } => {
            config::proxy_readiness(
                &config,
                resolve_bindings,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyAssertReady {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            allow_degraded,
        } => {
            config::proxy_assert_ready(
                &config,
                resolve_bindings,
                allow_degraded,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxySnapshot {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
        } => {
            config::proxy_snapshot(
                &config,
                resolve_bindings,
                history_limit,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyDiagnostics {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
        } => {
            config::proxy_diagnostics(
                &config,
                resolve_bindings,
                history_limit,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyAttention {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
        } => {
            config::proxy_attention(
                &config,
                resolve_bindings,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyIncidents {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
        } => {
            config::proxy_incidents(
                &config,
                resolve_bindings,
                history_limit,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyHandoff {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
        } => {
            config::proxy_handoff(
                &config,
                resolve_bindings,
                history_limit,
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyTimeline {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_timeline(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyTriage {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_triage(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyCasebook {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_casebook(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyBoard {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_board(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyFocus {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_focus(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyLens {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_lens(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyBrief {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_brief(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyDossier {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_dossier(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::ProxyRunbook {
            config,
            resolve_bindings,
            safe_mode,
            fail_on_warnings,
            require_fallback_ready,
            history_limit,
            route_id,
            destination_id,
        } => {
            config::proxy_runbook(
                &config,
                resolve_bindings,
                history_limit,
                route_id.as_deref(),
                destination_id.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen: false,
                },
            )
            .await
        }
        Command::WatchConfig {
            path,
            poll_ms,
            fail_on_warnings,
            require_fallback_ready,
        } => config::watch_config(&path, poll_ms, fail_on_warnings, require_fallback_ready).await,
        Command::DiffConfig { current, candidate } => {
            config::diff_config(&current, &candidate).await
        }
        Command::ServeHealth { listen, config } => {
            config::serve_health(&listen, config.as_deref()).await
        }
        Command::WatchUdpProxy {
            config,
            poll_ms,
            ingress_queue_depth,
            health_listen,
            control_listen,
            fail_on_warnings,
            require_fallback_ready,
            safe_mode,
            start_frozen,
        } => {
            runtime::watch_udp_proxy(
                &config,
                poll_ms,
                ingress_queue_depth,
                health_listen.as_deref(),
                control_listen.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen,
                },
            )
            .await
        }
        Command::RunUdpProxy {
            config,
            ingress_queue_depth,
            health_listen,
            control_listen,
            fail_on_warnings,
            require_fallback_ready,
            safe_mode,
            start_frozen,
        } => {
            runtime::run_udp_proxy(
                &config,
                ingress_queue_depth,
                health_listen.as_deref(),
                control_listen.as_deref(),
                ProxyCommandOptions {
                    fail_on_warnings,
                    require_fallback_ready,
                    safe_mode,
                    start_frozen,
                },
            )
            .await
        }
    }
}
