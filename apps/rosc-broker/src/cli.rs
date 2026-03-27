use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "ROSC broker bootstrap CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    CheckConfig {
        path: PathBuf,
    },
    ProxyStatus {
        config: PathBuf,
        #[arg(long)]
        resolve_bindings: bool,
        #[arg(long)]
        safe_mode: bool,
    },
    ProxyOverview {
        config: PathBuf,
        #[arg(long)]
        resolve_bindings: bool,
        #[arg(long)]
        safe_mode: bool,
        #[arg(long)]
        fail_on_warnings: bool,
        #[arg(long)]
        require_fallback_ready: bool,
    },
    ProxyDiagnostics {
        config: PathBuf,
        #[arg(long)]
        resolve_bindings: bool,
        #[arg(long)]
        safe_mode: bool,
        #[arg(long)]
        fail_on_warnings: bool,
        #[arg(long)]
        require_fallback_ready: bool,
        #[arg(long)]
        history_limit: Option<usize>,
    },
    WatchConfig {
        path: PathBuf,
        #[arg(long, default_value_t = 1000)]
        poll_ms: u64,
        #[arg(long)]
        fail_on_warnings: bool,
        #[arg(long)]
        require_fallback_ready: bool,
    },
    WatchUdpProxy {
        config: PathBuf,
        #[arg(long, default_value_t = 1000)]
        poll_ms: u64,
        #[arg(long, default_value_t = 1024)]
        ingress_queue_depth: usize,
        #[arg(long)]
        health_listen: Option<String>,
        #[arg(long)]
        control_listen: Option<String>,
        #[arg(long)]
        fail_on_warnings: bool,
        #[arg(long)]
        require_fallback_ready: bool,
        #[arg(long)]
        safe_mode: bool,
        #[arg(long)]
        start_frozen: bool,
    },
    DiffConfig {
        current: PathBuf,
        candidate: PathBuf,
    },
    ServeHealth {
        listen: String,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    RunUdpProxy {
        config: PathBuf,
        #[arg(long, default_value_t = 1024)]
        ingress_queue_depth: usize,
        #[arg(long)]
        health_listen: Option<String>,
        #[arg(long)]
        control_listen: Option<String>,
        #[arg(long)]
        fail_on_warnings: bool,
        #[arg(long)]
        require_fallback_ready: bool,
        #[arg(long)]
        safe_mode: bool,
        #[arg(long)]
        start_frozen: bool,
    },
}
