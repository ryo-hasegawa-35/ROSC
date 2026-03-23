use std::collections::{BTreeMap, BTreeSet};

use rosc_route::{RouteBuildError, RouteSpec, RoutingEngine};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BrokerConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub routes: Vec<RouteSpec>,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            routes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigDiff {
    pub added_routes: Vec<String>,
    pub removed_routes: Vec<String>,
    pub changed_routes: Vec<String>,
}

impl ConfigDiff {
    pub fn is_empty(&self) -> bool {
        self.added_routes.is_empty()
            && self.removed_routes.is_empty()
            && self.changed_routes.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedConfig {
    pub revision: u64,
    pub raw_toml: String,
    pub config: BrokerConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigApplyResult {
    pub revision: u64,
    pub diff: ConfigDiff,
}

#[derive(Default)]
pub struct ConfigManager {
    next_revision: u64,
    current: Option<AppliedConfig>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Parse(#[from] toml::de::Error),
    #[error("unsupported schema version `{0}`")]
    UnsupportedSchemaVersion(u32),
    #[error("route ids must be unique; duplicate `{0}` was found")]
    DuplicateRouteId(String),
    #[error(transparent)]
    RouteBuild(#[from] RouteBuildError),
}

impl BrokerConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, ConfigError> {
        let config = toml::from_str::<Self>(input)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchemaVersion(self.schema_version));
        }

        let mut seen = BTreeSet::new();
        for route in &self.routes {
            if !seen.insert(route.id.clone()) {
                return Err(ConfigError::DuplicateRouteId(route.id.clone()));
            }
        }

        let _engine = RoutingEngine::new(self.routes.clone())?;
        Ok(())
    }
}

impl ConfigManager {
    pub fn current(&self) -> Option<&AppliedConfig> {
        self.current.as_ref()
    }

    pub fn preview_toml_diff(&self, input: &str) -> Result<ConfigDiff, ConfigError> {
        let candidate = BrokerConfig::from_toml_str(input)?;
        Ok(match &self.current {
            Some(current) => diff_configs(&current.config, &candidate),
            None => ConfigDiff {
                added_routes: candidate
                    .routes
                    .iter()
                    .map(|route| route.id.clone())
                    .collect(),
                removed_routes: Vec::new(),
                changed_routes: Vec::new(),
            },
        })
    }

    pub fn apply_toml_str(&mut self, input: &str) -> Result<ConfigApplyResult, ConfigError> {
        let candidate = BrokerConfig::from_toml_str(input)?;
        let diff = match &self.current {
            Some(current) => diff_configs(&current.config, &candidate),
            None => ConfigDiff {
                added_routes: candidate
                    .routes
                    .iter()
                    .map(|route| route.id.clone())
                    .collect(),
                removed_routes: Vec::new(),
                changed_routes: Vec::new(),
            },
        };

        let revision = self.next_revision + 1;
        self.next_revision = revision;
        self.current = Some(AppliedConfig {
            revision,
            raw_toml: input.to_owned(),
            config: candidate,
        });

        Ok(ConfigApplyResult { revision, diff })
    }
}

fn diff_configs(current: &BrokerConfig, candidate: &BrokerConfig) -> ConfigDiff {
    let current_routes: BTreeMap<&str, &RouteSpec> = current
        .routes
        .iter()
        .map(|route| (route.id.as_str(), route))
        .collect();
    let candidate_routes: BTreeMap<&str, &RouteSpec> = candidate
        .routes
        .iter()
        .map(|route| (route.id.as_str(), route))
        .collect();

    let mut added_routes = Vec::new();
    let mut removed_routes = Vec::new();
    let mut changed_routes = Vec::new();

    for route_id in candidate_routes.keys() {
        match current_routes.get(route_id) {
            None => added_routes.push((*route_id).to_owned()),
            Some(current_route) if *current_route != candidate_routes[route_id] => {
                changed_routes.push((*route_id).to_owned())
            }
            Some(_) => {}
        }
    }

    for route_id in current_routes.keys() {
        if !candidate_routes.contains_key(route_id) {
            removed_routes.push((*route_id).to_owned());
        }
    }

    ConfigDiff {
        added_routes,
        removed_routes,
        changed_routes,
    }
}

const fn default_schema_version() -> u32 {
    SUPPORTED_SCHEMA_VERSION
}
