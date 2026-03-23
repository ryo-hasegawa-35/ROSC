use std::collections::{BTreeMap, BTreeSet};

use rosc_route::{
    CachePolicy, CapturePolicy, RouteBuildError, RouteSpec, RoutingEngine, TransportSelector,
};
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
    #[serde(default)]
    pub policy: UdpDestinationPolicyConfig,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropPolicyConfig {
    DropNewest,
    #[default]
    DropOldest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BreakerPolicyConfig {
    #[serde(default = "default_breaker_failure_threshold")]
    pub open_after_consecutive_failures: u32,
    #[serde(default = "default_breaker_overflow_threshold")]
    pub open_after_consecutive_queue_overflows: u32,
    #[serde(default = "default_breaker_cooldown_ms")]
    pub cooldown_ms: u64,
}

impl Default for BreakerPolicyConfig {
    fn default() -> Self {
        Self {
            open_after_consecutive_failures: default_breaker_failure_threshold(),
            open_after_consecutive_queue_overflows: default_breaker_overflow_threshold(),
            cooldown_ms: default_breaker_cooldown_ms(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UdpDestinationPolicyConfig {
    #[serde(default = "default_destination_queue_depth")]
    pub queue_depth: usize,
    #[serde(default)]
    pub drop_policy: DropPolicyConfig,
    #[serde(default)]
    pub breaker: BreakerPolicyConfig,
}

impl Default for UdpDestinationPolicyConfig {
    fn default() -> Self {
        Self {
            queue_depth: default_destination_queue_depth(),
            drop_policy: DropPolicyConfig::default(),
            breaker: BreakerPolicyConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigDiff {
    pub added_ingresses: Vec<String>,
    pub removed_ingresses: Vec<String>,
    pub changed_ingresses: Vec<String>,
    pub added_destinations: Vec<String>,
    pub removed_destinations: Vec<String>,
    pub changed_destinations: Vec<String>,
    pub added_routes: Vec<String>,
    pub removed_routes: Vec<String>,
    pub changed_routes: Vec<String>,
}

impl ConfigDiff {
    pub fn is_empty(&self) -> bool {
        self.added_ingresses.is_empty()
            && self.removed_ingresses.is_empty()
            && self.changed_ingresses.is_empty()
            && self.added_destinations.is_empty()
            && self.removed_destinations.is_empty()
            && self.changed_destinations.is_empty()
            && self.added_routes.is_empty()
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
    #[error("udp destination `{destination_id}` must have queue_depth >= 1")]
    InvalidUdpDestinationQueueDepth { destination_id: String },
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
    #[error("route `{route_id}` enables rehydrate without a cache policy")]
    RecoveryWithoutCache { route_id: String },
    #[error("route `{route_id}` enables replay without bounded capture")]
    ReplayWithoutCapture { route_id: String },
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
            if destination.policy.queue_depth == 0 {
                return Err(ConfigError::InvalidUdpDestinationQueueDepth {
                    destination_id: destination.id.clone(),
                });
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
            if route.cache.policy == CachePolicy::NoCache
                && (route.recovery.rehydrate_on_connect
                    || route.recovery.rehydrate_on_restart
                    || route.recovery.late_joiner.is_enabled())
            {
                return Err(ConfigError::RecoveryWithoutCache {
                    route_id: route.id.clone(),
                });
            }
            if route.recovery.replay_allowed && route.observability.capture == CapturePolicy::Off {
                return Err(ConfigError::ReplayWithoutCapture {
                    route_id: route.id.clone(),
                });
            }

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
                added_ingresses: candidate
                    .udp_ingresses
                    .iter()
                    .map(|ingress| ingress.id.clone())
                    .collect(),
                removed_ingresses: Vec::new(),
                changed_ingresses: Vec::new(),
                added_destinations: candidate
                    .udp_destinations
                    .iter()
                    .map(|destination| destination.id.clone())
                    .collect(),
                removed_destinations: Vec::new(),
                changed_destinations: Vec::new(),
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
                added_ingresses: candidate
                    .udp_ingresses
                    .iter()
                    .map(|ingress| ingress.id.clone())
                    .collect(),
                removed_ingresses: Vec::new(),
                changed_ingresses: Vec::new(),
                added_destinations: candidate
                    .udp_destinations
                    .iter()
                    .map(|destination| destination.id.clone())
                    .collect(),
                removed_destinations: Vec::new(),
                changed_destinations: Vec::new(),
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
    let (added_ingresses, removed_ingresses, changed_ingresses) = diff_named(
        current
            .udp_ingresses
            .iter()
            .map(|ingress| (ingress.id.as_str(), ingress)),
        candidate
            .udp_ingresses
            .iter()
            .map(|ingress| (ingress.id.as_str(), ingress)),
    );
    let (added_destinations, removed_destinations, changed_destinations) = diff_named(
        current
            .udp_destinations
            .iter()
            .map(|destination| (destination.id.as_str(), destination)),
        candidate
            .udp_destinations
            .iter()
            .map(|destination| (destination.id.as_str(), destination)),
    );
    let (added_routes, removed_routes, changed_routes) = diff_named(
        current
            .routes
            .iter()
            .map(|route| (route.id.as_str(), route)),
        candidate
            .routes
            .iter()
            .map(|route| (route.id.as_str(), route)),
    );

    ConfigDiff {
        added_ingresses,
        removed_ingresses,
        changed_ingresses,
        added_destinations,
        removed_destinations,
        changed_destinations,
        added_routes,
        removed_routes,
        changed_routes,
    }
}

fn diff_named<'a, T: Eq + 'a>(
    current: impl IntoIterator<Item = (&'a str, &'a T)>,
    candidate: impl IntoIterator<Item = (&'a str, &'a T)>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let current: BTreeMap<&str, &T> = current.into_iter().collect();
    let candidate: BTreeMap<&str, &T> = candidate.into_iter().collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for id in candidate.keys() {
        match current.get(id) {
            None => added.push((*id).to_owned()),
            Some(current_value) if *current_value != candidate[id] => {
                changed.push((*id).to_owned())
            }
            Some(_) => {}
        }
    }

    for id in current.keys() {
        if !candidate.contains_key(id) {
            removed.push((*id).to_owned());
        }
    }

    (added, removed, changed)
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

const fn default_destination_queue_depth() -> usize {
    16
}

const fn default_breaker_failure_threshold() -> u32 {
    3
}

const fn default_breaker_overflow_threshold() -> u32 {
    3
}

const fn default_breaker_cooldown_ms() -> u64 {
    250
}
