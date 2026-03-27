use regex::Regex;
use rosc_osc::CompatibilityMode;
use rosc_packet::{PacketEnvelope, TransportKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrafficClass {
    CriticalControl,
    StatefulControl,
    SensorStream,
    Telemetry,
    ForensicCapture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportSelector {
    OscUdp,
    OscTcp,
    OscSlip,
    WsJson,
    Mqtt,
    Ipc,
    Internal,
}

impl TransportSelector {
    fn matches(&self, transport: &TransportKind) -> bool {
        matches!(
            (self, transport),
            (Self::OscUdp, TransportKind::OscUdp)
                | (Self::OscTcp, TransportKind::OscTcp)
                | (Self::OscSlip, TransportKind::OscSlip)
                | (Self::WsJson, TransportKind::WsJson)
                | (Self::Mqtt, TransportKind::Mqtt)
                | (Self::Ipc, TransportKind::Ipc)
                | (Self::Internal, TransportKind::Internal)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DestinationRef {
    pub target: String,
    pub transport: TransportSelector,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl DestinationRef {
    pub fn destination_id(&self) -> &str {
        &self.target
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    #[default]
    NoCache,
    LastValuePerAddress,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistPolicy {
    #[default]
    Ephemeral,
    Warm,
    Durable,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LateJoinerPolicy {
    #[default]
    Disabled,
    Latest,
}

impl LateJoinerPolicy {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteMatchSpec {
    #[serde(default)]
    pub ingress_ids: Vec<String>,
    #[serde(default)]
    pub source_endpoints: Vec<String>,
    #[serde(default)]
    pub address_patterns: Vec<String>,
    #[serde(default)]
    pub protocols: Vec<TransportSelector>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransformSpec {
    pub rename_address: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteCacheSpec {
    #[serde(default)]
    pub policy: CachePolicy,
    pub ttl_ms: Option<u64>,
    #[serde(default)]
    pub persist: PersistPolicy,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteRecoverySpec {
    #[serde(default)]
    pub late_joiner: LateJoinerPolicy,
    #[serde(default)]
    pub rehydrate_on_connect: bool,
    #[serde(default)]
    pub rehydrate_on_restart: bool,
    #[serde(default)]
    pub replay_allowed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapturePolicy {
    #[default]
    Off,
    AlwaysBounded,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteObservabilitySpec {
    #[serde(default)]
    pub capture: CapturePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteSpec {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub mode: CompatibilityMode,
    pub class: TrafficClass,
    #[serde(rename = "match")]
    pub match_spec: RouteMatchSpec,
    #[serde(default)]
    pub transform: TransformSpec,
    #[serde(default)]
    pub cache: RouteCacheSpec,
    #[serde(default)]
    pub recovery: RouteRecoverySpec,
    #[serde(default)]
    pub observability: RouteObservabilitySpec,
    pub destinations: Vec<DestinationRef>,
}

#[derive(Clone, Debug)]
pub struct RouteDispatch {
    pub route_id: String,
    pub destination: DestinationRef,
    pub packet: PacketEnvelope,
    pub transform: TransformSpec,
    pub cache: RouteCacheSpec,
    pub recovery: RouteRecoverySpec,
    pub observability: RouteObservabilitySpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteFailure {
    pub route_id: String,
    pub error: RoutingError,
}

#[derive(Clone, Debug, Default)]
pub struct RoutingOutcome {
    pub dispatches: Vec<RouteDispatch>,
    pub failures: Vec<RouteFailure>,
}

#[derive(Debug, Error)]
pub enum RouteBuildError {
    #[error("route `{route_id}` has no destinations")]
    MissingDestination { route_id: String },
    #[error("route `{route_id}` uses an unsupported address pattern `{pattern}`")]
    InvalidAddressPattern { route_id: String, pattern: String },
}

#[derive(Debug, Error)]
enum PatternCompileError {
    #[error("`//` path traversal requires osc1_1_extended mode")]
    TraversalWildcardRequiresExtendedMode,
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RoutingError {
    #[error("route `{route_id}` requires a transform that cannot be applied safely")]
    TransformNotAllowed { route_id: String },
}

pub struct RoutingEngine {
    routes: Vec<CompiledRoute>,
}

impl RoutingEngine {
    pub fn new(routes: Vec<RouteSpec>) -> Result<Self, RouteBuildError> {
        let routes = routes
            .into_iter()
            .map(CompiledRoute::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { routes })
    }

    pub fn route(&self, packet: &PacketEnvelope) -> RoutingOutcome {
        let mut outcome = RoutingOutcome::default();

        for route in &self.routes {
            if !route.matches(packet) {
                continue;
            }

            let routed_packet = match route.apply_transform(packet) {
                Ok(routed_packet) => routed_packet,
                Err(error) => {
                    outcome.failures.push(RouteFailure {
                        route_id: route.spec.id.clone(),
                        error,
                    });
                    continue;
                }
            };
            for destination in &route.spec.destinations {
                if destination.enabled {
                    outcome.dispatches.push(RouteDispatch {
                        route_id: route.spec.id.clone(),
                        destination: destination.clone(),
                        packet: routed_packet.clone(),
                        transform: route.spec.transform.clone(),
                        cache: route.spec.cache.clone(),
                        recovery: route.spec.recovery.clone(),
                        observability: route.spec.observability.clone(),
                    });
                }
            }
        }

        outcome
    }
}

struct CompiledRoute {
    spec: RouteSpec,
    compiled_patterns: Vec<Regex>,
}

impl CompiledRoute {
    fn new(spec: RouteSpec) -> Result<Self, RouteBuildError> {
        if spec.destinations.is_empty() {
            return Err(RouteBuildError::MissingDestination {
                route_id: spec.id.clone(),
            });
        }

        let mut compiled_patterns = Vec::new();
        for pattern in &spec.match_spec.address_patterns {
            compiled_patterns.push(compile_osc_pattern(pattern, spec.mode).map_err(|_| {
                RouteBuildError::InvalidAddressPattern {
                    route_id: spec.id.clone(),
                    pattern: pattern.clone(),
                }
            })?);
        }

        Ok(Self {
            spec,
            compiled_patterns,
        })
    }

    fn matches(&self, packet: &PacketEnvelope) -> bool {
        if !self.spec.enabled {
            return false;
        }
        if packet.metadata.compatibility_mode != self.spec.mode {
            return false;
        }
        if !self.spec.match_spec.ingress_ids.is_empty()
            && !self
                .spec
                .match_spec
                .ingress_ids
                .iter()
                .any(|ingress_id| ingress_id == &packet.metadata.ingress_id)
        {
            return false;
        }
        if !self.spec.match_spec.source_endpoints.is_empty()
            && !packet
                .metadata
                .source_endpoint
                .as_ref()
                .map(|source| {
                    self.spec
                        .match_spec
                        .source_endpoints
                        .iter()
                        .any(|expected| expected == source)
                })
                .unwrap_or(false)
        {
            return false;
        }
        if !self.spec.match_spec.protocols.is_empty()
            && !self
                .spec
                .match_spec
                .protocols
                .iter()
                .any(|selector| selector.matches(&packet.metadata.transport))
        {
            return false;
        }

        if self.compiled_patterns.is_empty() {
            return true;
        }

        packet
            .address()
            .map(|address| {
                self.compiled_patterns
                    .iter()
                    .any(|pattern| pattern.is_match(address))
            })
            .unwrap_or(false)
    }

    fn apply_transform(&self, packet: &PacketEnvelope) -> Result<PacketEnvelope, RoutingError> {
        if let Some(rename_address) = &self.spec.transform.rename_address {
            packet
                .derive_with_renamed_address(rename_address)
                .map_err(|_| RoutingError::TransformNotAllowed {
                    route_id: self.spec.id.clone(),
                })
        } else {
            Ok(packet.clone())
        }
    }
}

fn compile_osc_pattern(
    pattern: &str,
    mode: CompatibilityMode,
) -> Result<Regex, PatternCompileError> {
    if pattern.contains("//") && mode != CompatibilityMode::Osc1_1Extended {
        return Err(PatternCompileError::TraversalWildcardRequiresExtendedMode);
    }

    let mut regex = String::from("^");
    let bytes = pattern.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'/' => {
                regex.push_str("(?:/(?:[^/]+))*/");
                index += 2;
            }
            b'*' => {
                regex.push_str("[^/]*");
                index += 1;
            }
            b'?' => {
                regex.push_str("[^/]");
                index += 1;
            }
            b'{' => {
                let end = pattern[index + 1..].find('}').ok_or_else(|| {
                    PatternCompileError::Regex(regex::Error::Syntax(
                        "unterminated alternation".to_owned(),
                    ))
                })?;
                let choices = &pattern[index + 1..index + 1 + end];
                regex.push_str("(?:");
                for (choice_index, choice) in choices.split(',').enumerate() {
                    if choice_index > 0 {
                        regex.push('|');
                    }
                    regex.push_str(&regex::escape(choice));
                }
                regex.push(')');
                index += end + 2;
            }
            b'[' => {
                let end = pattern[index + 1..].find(']').ok_or_else(|| {
                    PatternCompileError::Regex(regex::Error::Syntax(
                        "unterminated character class".to_owned(),
                    ))
                })?;
                regex.push('[');
                regex.push_str(&pattern[index + 1..index + 1 + end]);
                regex.push(']');
                index += end + 2;
            }
            other => {
                regex.push_str(&regex::escape(&(other as char).to_string()));
                index += 1;
            }
        }
    }
    regex.push('$');
    Regex::new(&regex).map_err(PatternCompileError::from)
}

const fn default_enabled() -> bool {
    true
}
