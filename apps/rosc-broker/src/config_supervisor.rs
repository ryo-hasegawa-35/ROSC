use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rosc_config::{ConfigApplyResult, ConfigError, ConfigManager};
use rosc_telemetry::{BrokerEvent, TelemetrySink};

#[derive(Debug)]
pub enum ConfigReloadOutcome {
    Unchanged,
    Applied(ConfigApplyResult),
    Rejected(ConfigError),
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
        let content = read_config_file(&self.path)?;
        let applied = self.manager.apply_toml_str(&content)?;
        self.emit_config_applied(&applied);
        Ok(applied)
    }

    pub fn poll_once(&mut self) -> Result<ConfigReloadOutcome> {
        let content = read_config_file(&self.path)?;
        if self
            .manager
            .current()
            .map(|current| current.raw_toml == content)
            .unwrap_or(false)
        {
            return Ok(ConfigReloadOutcome::Unchanged);
        }

        match self.manager.apply_toml_str(&content) {
            Ok(applied) => {
                self.emit_config_applied(&applied);
                Ok(ConfigReloadOutcome::Applied(applied))
            }
            Err(error) => {
                self.telemetry.emit(BrokerEvent::ConfigRejected);
                Ok(ConfigReloadOutcome::Rejected(error))
            }
        }
    }

    fn emit_config_applied(&self, applied: &ConfigApplyResult) {
        self.telemetry.emit(BrokerEvent::ConfigApplied {
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
}

fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to read config file {}", path.display()))
}
