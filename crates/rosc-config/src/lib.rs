use rosc_route::{RouteBuildError, RouteSpec, RoutingEngine};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BrokerConfig {
    #[serde(default)]
    pub routes: Vec<RouteSpec>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Parse(#[from] toml::de::Error),
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
        let mut seen = std::collections::BTreeSet::new();
        for route in &self.routes {
            if !seen.insert(route.id.clone()) {
                return Err(ConfigError::DuplicateRouteId(route.id.clone()));
            }
        }

        let _engine = RoutingEngine::new(self.routes.clone())?;
        Ok(())
    }
}
