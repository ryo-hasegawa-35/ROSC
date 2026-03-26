use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rosc_config::{ConfigApplyResult, ConfigError, ConfigManager};
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};

use crate::{
    ManagedProxyStartupOptions, ManagedUdpProxy, ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy,
    UdpProxyStatusSnapshot,
};

#[derive(Debug)]
pub enum ProxyReloadOutcome {
    Unchanged,
    Applied(ConfigApplyResult),
    Blocked(Vec<String>),
    Rejected(ConfigError),
    ReloadFailed(String),
}

pub struct ManagedProxyFileSupervisor {
    path: PathBuf,
    manager: ConfigManager,
    proxy: ManagedUdpProxy,
    telemetry: InMemoryTelemetry,
}

impl ManagedProxyFileSupervisor {
    pub async fn start(
        path: impl Into<PathBuf>,
        telemetry: InMemoryTelemetry,
        ingress_queue_depth: usize,
        safety_policy: ProxyRuntimeSafetyPolicy,
        launch_profile_mode: ProxyLaunchProfileMode,
        startup_options: ManagedProxyStartupOptions,
    ) -> Result<Self> {
        let path = path.into();
        let mut manager = ConfigManager::default();
        let raw_toml = read_config_file(&path)?;
        let preview = manager.preview_toml_str(&raw_toml)?;
        let proxy = ManagedUdpProxy::start(
            preview.config.clone(),
            telemetry.clone(),
            ingress_queue_depth,
            safety_policy,
            launch_profile_mode,
            startup_options,
        )
        .await?;
        manager.apply_preview(&raw_toml, preview);

        Ok(Self {
            path,
            manager,
            proxy,
            telemetry,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn current_revision(&self) -> Option<u64> {
        self.manager.current().map(|current| current.revision)
    }

    pub fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        self.proxy.status_snapshot()
    }

    pub fn operator_report(&self) -> crate::ProxyOperatorReport {
        self.proxy.operator_report()
    }

    pub fn proxy(&self) -> &ManagedUdpProxy {
        &self.proxy
    }

    pub fn freeze_traffic(&self) -> bool {
        self.proxy.freeze_traffic()
    }

    pub fn thaw_traffic(&self) -> bool {
        self.proxy.thaw_traffic()
    }

    pub fn has_route(&self, route_id: &str) -> bool {
        self.proxy.has_route(route_id)
    }

    pub fn has_destination(&self, destination_id: &str) -> bool {
        self.proxy.has_destination(destination_id)
    }

    pub fn isolated_routes(&self) -> Vec<String> {
        self.proxy.isolated_routes()
    }

    pub fn isolate_route(&self, route_id: &str) -> bool {
        self.proxy.isolate_route(route_id)
    }

    pub fn restore_route(&self, route_id: &str) -> bool {
        self.proxy.restore_route(route_id)
    }

    pub async fn rehydrate_destination(&self, destination_id: &str) -> Result<usize> {
        self.proxy.rehydrate_destination(destination_id).await
    }

    pub async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<usize> {
        self.proxy
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
    }

    pub async fn poll_once(&mut self) -> Result<ProxyReloadOutcome> {
        let raw_toml = read_config_file(&self.path)?;
        if self
            .manager
            .current()
            .map(|current| current.raw_toml == raw_toml)
            .unwrap_or(false)
        {
            return Ok(ProxyReloadOutcome::Unchanged);
        }

        let preview = match self.manager.preview_toml_str(&raw_toml) {
            Ok(preview) => preview,
            Err(error) => {
                self.telemetry.emit(BrokerEvent::ConfigRejected {
                    reason: error.to_string(),
                });
                return Ok(ProxyReloadOutcome::Rejected(error));
            }
        };

        match self.proxy.reload(preview.config.clone()).await {
            Ok(()) => {
                let applied = self.manager.apply_preview(&raw_toml, preview);
                Ok(ProxyReloadOutcome::Applied(applied))
            }
            Err(error) => {
                let message = format!(
                    "failed to reload managed proxy from {}: {error:#}",
                    self.path.display()
                );
                Ok(match self.classify_reload_error(&message) {
                    ReloadFailureKind::Blocked(reasons) => {
                        self.telemetry.emit(BrokerEvent::ConfigBlocked {
                            reasons: reasons.clone(),
                        });
                        ProxyReloadOutcome::Blocked(reasons)
                    }
                    ReloadFailureKind::Rejected(message) => {
                        self.telemetry.emit(BrokerEvent::ConfigReloadFailed {
                            reason: message.clone(),
                        });
                        ProxyReloadOutcome::ReloadFailed(message)
                    }
                })
            }
        }
    }

    pub async fn shutdown(&mut self) {
        self.proxy.shutdown().await;
    }

    fn classify_reload_error(&self, message: &str) -> ReloadFailureKind {
        const RELOAD_BLOCKED_PREFIX: &str = "udp proxy reload blocked:";
        if let Some(block) = message.split(RELOAD_BLOCKED_PREFIX).nth(1) {
            let reasons = block
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.trim_start_matches("- ").to_owned())
                .collect::<Vec<_>>();
            return ReloadFailureKind::Blocked(reasons);
        }

        if let Some(block) = message.split("udp proxy startup blocked:").nth(1) {
            let reasons = block
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.trim_start_matches("- ").to_owned())
                .collect::<Vec<_>>();
            return ReloadFailureKind::Blocked(reasons);
        }

        ReloadFailureKind::Rejected(message.to_owned())
    }
}

enum ReloadFailureKind {
    Blocked(Vec<String>),
    Rejected(String),
}

fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to read config file {}", path.display()))
}
