use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rosc_config::{ConfigApplyResult, ConfigError, ConfigManager};
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};

use crate::{ManagedUdpProxy, ProxyRuntimeSafetyPolicy, UdpProxyStatusSnapshot};

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
        )
        .await?;
        let applied = manager.apply_preview(&raw_toml, preview);
        emit_config_applied(&telemetry, &applied);

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
        self.proxy.app().status_snapshot()
    }

    pub fn proxy(&self) -> &ManagedUdpProxy {
        &self.proxy
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
                self.telemetry.emit(BrokerEvent::ConfigRejected);
                return Ok(ProxyReloadOutcome::Rejected(error));
            }
        };

        match self.proxy.reload(preview.config.clone()).await {
            Ok(()) => {
                let applied = self.manager.apply_preview(&raw_toml, preview);
                emit_config_applied(&self.telemetry, &applied);
                Ok(ProxyReloadOutcome::Applied(applied))
            }
            Err(error) => {
                self.telemetry.emit(BrokerEvent::ConfigRejected);
                let message = format!(
                    "failed to reload managed proxy from {}: {error:#}",
                    self.path.display()
                );
                Ok(match self.classify_reload_error(&message) {
                    ReloadFailureKind::Blocked(reasons) => ProxyReloadOutcome::Blocked(reasons),
                    ReloadFailureKind::Rejected(message) => {
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

fn emit_config_applied(telemetry: &InMemoryTelemetry, applied: &ConfigApplyResult) {
    telemetry.emit(BrokerEvent::ConfigApplied {
        revision: applied.revision,
        added_ingresses: applied.diff.added_ingresses.len(),
        removed_ingresses: applied.diff.removed_ingresses.len(),
        changed_ingresses: applied.diff.changed_ingresses.len(),
        added_destinations: applied.diff.added_destinations.len(),
        removed_destinations: applied.diff.removed_destinations.len(),
        changed_destinations: applied.diff.changed_destinations.len(),
        added_routes: applied.diff.added_routes.len(),
        removed_routes: applied.diff.removed_routes.len(),
        changed_routes: applied.diff.changed_routes.len(),
    });
}

fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to read config file {}", path.display()))
}
