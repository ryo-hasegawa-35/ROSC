use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rosc_config::{BrokerConfig, ConfigApplyResult, ConfigError, ConfigManager};
use rosc_telemetry::{BrokerEvent, TelemetrySink};

use crate::emit_applied_config;

#[derive(Debug)]
pub enum ConfigReloadOutcome {
    Unchanged,
    Applied(ConfigApplyResult),
    Rejected(ConfigError),
    Blocked(Vec<String>),
}

pub struct ConfigFileSupervisor<TTelemetry> {
    path: PathBuf,
    manager: ConfigManager,
    telemetry: TTelemetry,
}

impl<TTelemetry> ConfigFileSupervisor<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn new(path: impl Into<PathBuf>, telemetry: TTelemetry) -> Self {
        Self {
            path: path.into(),
            manager: ConfigManager::default(),
            telemetry,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn current_revision(&self) -> Option<u64> {
        self.manager.current().map(|applied| applied.revision)
    }

    pub fn load_initial(&mut self) -> Result<ConfigApplyResult> {
        self.load_initial_with_guard(|_| Ok(()))
    }

    pub fn load_initial_with_guard<F>(&mut self, guard: F) -> Result<ConfigApplyResult>
    where
        F: FnOnce(&BrokerConfig) -> Result<(), Vec<String>>,
    {
        let content = read_config_file(&self.path)?;
        let preview = self.manager.preview_toml_str(&content)?;
        if let Err(reasons) = guard(&preview.config) {
            self.telemetry.emit(BrokerEvent::ConfigRejected);
            anyhow::bail!(format_blocked_reasons(
                "initial config blocked by runtime safety policy",
                reasons
            ));
        }
        let applied = self.manager.apply_preview(&content, preview);
        emit_applied_config(&self.telemetry, &applied);
        Ok(applied)
    }

    pub fn poll_once(&mut self) -> Result<ConfigReloadOutcome> {
        self.poll_once_with_guard(|_| Ok(()))
    }

    pub fn poll_once_with_guard<F>(&mut self, guard: F) -> Result<ConfigReloadOutcome>
    where
        F: FnOnce(&BrokerConfig) -> Result<(), Vec<String>>,
    {
        let content = read_config_file(&self.path)?;
        if self
            .manager
            .current()
            .map(|current| current.raw_toml == content)
            .unwrap_or(false)
        {
            return Ok(ConfigReloadOutcome::Unchanged);
        }

        match self.manager.preview_toml_str(&content) {
            Ok(preview) => {
                if let Err(reasons) = guard(&preview.config) {
                    self.telemetry.emit(BrokerEvent::ConfigRejected);
                    return Ok(ConfigReloadOutcome::Blocked(reasons));
                }
                let applied = self.manager.apply_preview(&content, preview);
                emit_applied_config(&self.telemetry, &applied);
                Ok(ConfigReloadOutcome::Applied(applied))
            }
            Err(error) => {
                self.telemetry.emit(BrokerEvent::ConfigRejected);
                Ok(ConfigReloadOutcome::Rejected(error))
            }
        }
    }
}

fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to read config file {}", path.display()))
}

fn format_blocked_reasons(header: &str, reasons: Vec<String>) -> String {
    if reasons.is_empty() {
        return header.to_owned();
    }

    format!(
        "{header}:\n{}",
        reasons
            .into_iter()
            .map(|reason| format!("- {reason}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
