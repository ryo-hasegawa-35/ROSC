use std::collections::{BTreeMap, BTreeSet};

use rosc_route::{RouteBuildError, RouteSpec, RoutingEngine, TransportSelector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BrokerConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub udp_ingresses: Vec<UdpIngressConfig>,
    #[serde(default)]
    pub udp_destinations: Vec<UdpDestinationConfig>,
    #[serde(default)]
    pub routes: Vec<RouteSpec>,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            udp_ingresses: Vec::new(),
            udp_destinations: Vec::new(),
            routes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UdpIngressConfig {
    pub id: String,
    pub bind: String,
    pub mode: rosc_osc::CompatibilityMode,
    #[serde(default = "default_udp_packet_size")]
    pub max_packet_size: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UdpDestinationConfig {
    pub id: String,
    #[serde(default = "default_udp_bind_address")]
    pub bind: String,
    pub target: String,
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
    #[error("udp ingress ids must be unique; duplicate `{0}` was found")]
    DuplicateUdpIngressId(String),
    #[error("udp destination ids must be unique; duplicate `{0}` was found")]
    DuplicateUdpDestinationId(String),
    #[error("route `{route_id}` references unknown ingress `{ingress_id}`")]
    UnknownIngressReference {
        route_id: String,
        ingress_id: String,
    },
    #[error("route `{route_id}` references unknown udp destination `{destination_id}`")]
    UnknownUdpDestinationReference {
        route_id: String,
        destination_id: String,
    },
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

        let mut ingress_ids = BTreeSet::new();
        for ingress in &self.udp_ingresses {
            if !ingress_ids.insert(ingress.id.clone()) {
                return Err(ConfigError::DuplicateUdpIngressId(ingress.id.clone()));
            }
        }

        let mut destination_ids = BTreeSet::new();
        for destination in &self.udp_destinations {
            if !destination_ids.insert(destination.id.clone()) {
                return Err(ConfigError::DuplicateUdpDestinationId(
                    destination.id.clone(),
                ));
            }
        }

        let _engine = RoutingEngine::new(self.routes.clone())?;
        self.validate_runtime_references()?;
        Ok(())
    }

    pub fn validate_runtime_references(&self) -> Result<(), ConfigError> {
        let ingress_ids: BTreeSet<&str> = self
            .udp_ingresses
            .iter()
            .map(|ingress| ingress.id.as_str())
            .collect();
        let destination_ids: BTreeSet<&str> = self
            .udp_destinations
            .iter()
            .map(|destination| destination.id.as_str())
            .collect();

        for route in &self.routes {
            for ingress_id in &route.match_spec.ingress_ids {
                if !ingress_ids.contains(ingress_id.as_str()) {
                    return Err(ConfigError::UnknownIngressReference {
                        route_id: route.id.clone(),
                        ingress_id: ingress_id.clone(),
                    });
                }
            }

            for destination in &route.destinations {
                if destination.transport == TransportSelector::OscUdp
                    && !destination_ids.contains(destination.target.as_str())
                {
                    return Err(ConfigError::UnknownUdpDestinationReference {
                        route_id: route.id.clone(),
                        destination_id: destination.target.clone(),
                    });
                }
            }
        }

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

const fn default_udp_packet_size() -> usize {
    65_536
}

fn default_udp_bind_address() -> String {
    "0.0.0.0:0".to_owned()
}
