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

pub async fn run(command: Command) -> Result<()> {
    match command {
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
