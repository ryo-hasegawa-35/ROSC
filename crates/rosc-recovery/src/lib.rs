use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use rosc_packet::TransportKind;
use rosc_route::{CachePolicy, CapturePolicy, DestinationRef, RouteDispatch};
use rosc_telemetry::{BrokerEvent, TelemetrySink};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CacheKey {
    route_id: String,
    destination_id: String,
    address: String,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    dispatch: RouteDispatch,
    expires_at: Option<SystemTime>,
}

#[derive(Clone, Debug)]
struct CaptureRecord {
    dispatch: RouteDispatch,
    captured_at: SystemTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    Rehydrate,
    SandboxReplay,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryAuditRecord {
    pub action: RecoveryAction,
    pub route_id: String,
    pub source_destination_id: Option<String>,
    pub target_destination_id: String,
    pub count: usize,
    pub recorded_at: SystemTime,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RehydrateRequest {
    pub route_id: Option<String>,
    pub destination_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SandboxReplayRequest {
    pub route_id: String,
    pub source_destination_id: Option<String>,
    pub sandbox_destination_id: String,
    pub limit: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RehydrateOutcome {
    pub dispatches: Vec<RouteDispatch>,
    pub stale_evictions: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SandboxReplayOutcome {
    pub dispatches: Vec<RouteDispatch>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RecoveryError {
    #[error("rehydrate requests must specify at least one selector")]
    MissingSelector,
    #[error("sandbox replay requests must set a positive limit")]
    ZeroReplayLimit,
}

pub struct RecoveryEngine<TTelemetry> {
    telemetry: TTelemetry,
    entries: Arc<Mutex<BTreeMap<CacheKey, CacheEntry>>>,
    captures: Arc<Mutex<VecDeque<CaptureRecord>>>,
    audit: Arc<Mutex<VecDeque<RecoveryAuditRecord>>>,
    max_capture_entries: usize,
    max_audit_entries: usize,
}

impl<TTelemetry> RecoveryEngine<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn new(telemetry: TTelemetry) -> Self {
        Self::with_limits(telemetry, 1024, 256)
    }

    pub fn with_limits(
        telemetry: TTelemetry,
        max_capture_entries: usize,
        max_audit_entries: usize,
    ) -> Self {
        Self {
            telemetry,
            entries: Arc::new(Mutex::new(BTreeMap::new())),
            captures: Arc::new(Mutex::new(VecDeque::new())),
            audit: Arc::new(Mutex::new(VecDeque::new())),
            max_capture_entries,
            max_audit_entries,
        }
    }

    pub fn observe_dispatches(&self, dispatches: &[RouteDispatch]) {
        let mut entries = self.entries.lock().expect("recovery cache mutex poisoned");
        let mut captures = self.captures.lock().expect("capture mutex poisoned");
        let mut changed_cache_routes = BTreeSet::new();
        let mut changed_capture_routes = BTreeSet::new();

        for dispatch in dispatches {
            if dispatch.cache.policy == CachePolicy::LastValuePerAddress
                && dispatch
                    .packet
                    .capabilities
                    .contains(rosc_packet::PacketCapabilities::CACHEABLE_CANDIDATE)
            {
                let Some(address) = dispatch.packet.address() else {
                    continue;
                };

                let cached_at = SystemTime::now();
                let expires_at = dispatch
                    .cache
                    .ttl_ms
                    .map(Duration::from_millis)
                    .and_then(|ttl| cached_at.checked_add(ttl));
                let key = CacheKey {
                    route_id: dispatch.route_id.clone(),
                    destination_id: dispatch.destination.destination_id().to_owned(),
                    address: address.to_owned(),
                };

                entries.insert(
                    key,
                    CacheEntry {
                        dispatch: dispatch.clone(),
                        expires_at,
                    },
                );
                changed_cache_routes.insert(dispatch.route_id.clone());
                self.telemetry.emit(BrokerEvent::CacheStored {
                    route_id: dispatch.route_id.clone(),
                });
            }

            if dispatch.observability.capture == CapturePolicy::AlwaysBounded {
                captures.push_back(CaptureRecord {
                    dispatch: dispatch.clone(),
                    captured_at: SystemTime::now(),
                });
                changed_capture_routes.insert(dispatch.route_id.clone());
                self.telemetry.emit(BrokerEvent::CaptureStored {
                    route_id: dispatch.route_id.clone(),
                });

                while captures.len() > self.max_capture_entries {
                    if let Some(evicted) = captures.pop_front() {
                        changed_capture_routes.insert(evicted.dispatch.route_id);
                    }
                }
            }
        }

        self.emit_cache_entry_metrics(&entries, &changed_cache_routes);
        self.emit_capture_entry_metrics(&captures, &changed_capture_routes);
    }

    pub fn rehydrate(&self, request: RehydrateRequest) -> Result<RehydrateOutcome, RecoveryError> {
        if request.route_id.is_none() && request.destination_id.is_none() {
            return Err(RecoveryError::MissingSelector);
        }

        let now = SystemTime::now();
        let mut entries = self.entries.lock().expect("recovery cache mutex poisoned");
        let keys = entries.keys().cloned().collect::<Vec<_>>();

        let mut route_counts = BTreeMap::<(String, String), usize>::new();
        let mut changed_cache_routes = BTreeSet::new();
        let mut stale_keys = Vec::new();
        let mut dispatches = Vec::new();

        for key in keys {
            let Some(entry) = entries.get(&key) else {
                continue;
            };
            if !matches_request(&key, &request) {
                continue;
            }
            if entry.dispatch.recovery.rehydrate_on_connect
                && entry.dispatch.recovery.late_joiner.is_enabled()
            {
                if is_stale(entry, now) {
                    changed_cache_routes.insert(key.route_id.clone());
                    stale_keys.push(key.clone());
                    continue;
                }

                dispatches.push(mark_dispatch_lineage(
                    &entry.dispatch,
                    format!("rehydrate:{}", entry.dispatch.route_id),
                    Some("recovery:rehydrate".to_owned()),
                ));
                *route_counts
                    .entry((key.route_id.clone(), key.destination_id.clone()))
                    .or_default() += 1;
            }
        }

        for key in &stale_keys {
            if let Some(entry) = entries.remove(key) {
                self.telemetry.emit(BrokerEvent::CacheEvicted {
                    route_id: entry.dispatch.route_id.clone(),
                    reason: "stale".to_owned(),
                });
            }
        }

        for ((route_id, destination_id), count) in &route_counts {
            self.telemetry.emit(BrokerEvent::RecoveryRehydrate {
                route_id: route_id.clone(),
                destination_id: destination_id.clone(),
                count: *count,
            });
            self.record_audit(RecoveryAuditRecord {
                action: RecoveryAction::Rehydrate,
                route_id: route_id.clone(),
                source_destination_id: Some(destination_id.clone()),
                target_destination_id: destination_id.clone(),
                count: *count,
                recorded_at: SystemTime::now(),
            });
        }

        self.emit_cache_entry_metrics(&entries, &changed_cache_routes);

        Ok(RehydrateOutcome {
            dispatches,
            stale_evictions: stale_keys.len(),
        })
    }

    pub fn sandbox_replay(
        &self,
        request: SandboxReplayRequest,
    ) -> Result<SandboxReplayOutcome, RecoveryError> {
        if request.limit == 0 {
            return Err(RecoveryError::ZeroReplayLimit);
        }

        let captures = self.captures.lock().expect("capture mutex poisoned");
        let mut selected = captures
            .iter()
            .rev()
            .filter(|record| record.dispatch.route_id == request.route_id)
            .filter(|record| {
                request
                    .source_destination_id
                    .as_ref()
                    .is_none_or(|destination_id| {
                        record.dispatch.destination.destination_id() == destination_id
                    })
            })
            .filter(|record| record.dispatch.recovery.replay_allowed)
            .take(request.limit)
            .cloned()
            .collect::<Vec<_>>();
        selected.reverse();

        let dispatches = selected
            .iter()
            .map(|record| {
                let mut replay_dispatch = mark_dispatch_lineage(
                    &record.dispatch,
                    format!("replay:{}", record.dispatch.route_id),
                    Some(format!(
                        "recovery:sandbox_replay:{}",
                        record
                            .captured_at
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis()
                    )),
                );
                replay_dispatch.destination = DestinationRef {
                    target: request.sandbox_destination_id.clone(),
                    ..record.dispatch.destination.clone()
                };
                replay_dispatch
            })
            .collect::<Vec<_>>();

        if !dispatches.is_empty() {
            self.telemetry.emit(BrokerEvent::RecoveryReplay {
                route_id: request.route_id.clone(),
                destination_id: request.sandbox_destination_id.clone(),
                count: dispatches.len(),
            });
            self.record_audit(RecoveryAuditRecord {
                action: RecoveryAction::SandboxReplay,
                route_id: request.route_id,
                source_destination_id: request.source_destination_id,
                target_destination_id: request.sandbox_destination_id,
                count: dispatches.len(),
                recorded_at: SystemTime::now(),
            });
        }

        Ok(SandboxReplayOutcome { dispatches })
    }

    pub fn audit_records(&self) -> Vec<RecoveryAuditRecord> {
        self.audit
            .lock()
            .expect("audit mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn cached_routes(&self) -> BTreeSet<String> {
        self.entries
            .lock()
            .expect("recovery cache mutex poisoned")
            .keys()
            .map(|key| key.route_id.clone())
            .collect()
    }

    pub fn captured_routes(&self) -> BTreeSet<String> {
        self.captures
            .lock()
            .expect("capture mutex poisoned")
            .iter()
            .map(|record| record.dispatch.route_id.clone())
            .collect()
    }

    fn record_audit(&self, record: RecoveryAuditRecord) {
        let mut audit = self.audit.lock().expect("audit mutex poisoned");
        audit.push_back(record);
        while audit.len() > self.max_audit_entries {
            let _ = audit.pop_front();
        }
    }

    fn emit_cache_entry_metrics(
        &self,
        entries: &BTreeMap<CacheKey, CacheEntry>,
        additional_routes: &BTreeSet<String>,
    ) {
        let mut by_route = BTreeMap::<String, usize>::new();
        for key in entries.keys() {
            *by_route.entry(key.route_id.clone()).or_default() += 1;
        }
        for route_id in additional_routes {
            by_route.entry(route_id.clone()).or_insert(0);
        }
        for (route_id, entries) in by_route {
            self.telemetry
                .emit(BrokerEvent::CacheEntriesChanged { route_id, entries });
        }
    }

    fn emit_capture_entry_metrics(
        &self,
        captures: &VecDeque<CaptureRecord>,
        additional_routes: &BTreeSet<String>,
    ) {
        let mut by_route = BTreeMap::<String, usize>::new();
        for record in captures {
            *by_route
                .entry(record.dispatch.route_id.clone())
                .or_default() += 1;
        }
        for route_id in additional_routes {
            by_route.entry(route_id.clone()).or_insert(0);
        }
        for (route_id, entries) in by_route {
            self.telemetry
                .emit(BrokerEvent::CaptureEntriesChanged { route_id, entries });
        }
    }
}

fn is_stale(entry: &CacheEntry, now: SystemTime) -> bool {
    entry.expires_at.is_some_and(|expires_at| expires_at <= now)
}

fn matches_request(key: &CacheKey, request: &RehydrateRequest) -> bool {
    request
        .route_id
        .as_ref()
        .is_none_or(|route_id| route_id == &key.route_id)
        && request
            .destination_id
            .as_ref()
            .is_none_or(|destination_id| destination_id == &key.destination_id)
}

fn mark_dispatch_lineage(
    dispatch: &RouteDispatch,
    ingress_id: String,
    source_endpoint: Option<String>,
) -> RouteDispatch {
    let metadata = rosc_packet::IngressMetadata {
        ingress_id,
        transport: TransportKind::Internal,
        source_endpoint,
        compatibility_mode: dispatch.packet.metadata.compatibility_mode,
        received_at: SystemTime::now(),
    };

    RouteDispatch {
        packet: dispatch.packet.clone_with_metadata(metadata),
        ..dispatch.clone()
    }
}
