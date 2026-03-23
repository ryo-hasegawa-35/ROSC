use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use rosc_route::{CachePolicy, RouteDispatch};
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RehydrateRequest {
    pub route_id: Option<String>,
    pub destination_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RehydrateOutcome {
    pub dispatches: Vec<RouteDispatch>,
    pub stale_evictions: usize,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RecoveryError {
    #[error("rehydrate requests must specify at least one selector")]
    MissingSelector,
}

pub struct RecoveryEngine<TTelemetry> {
    telemetry: TTelemetry,
    entries: Arc<Mutex<BTreeMap<CacheKey, CacheEntry>>>,
}

impl<TTelemetry> RecoveryEngine<TTelemetry>
where
    TTelemetry: TelemetrySink,
{
    pub fn new(telemetry: TTelemetry) -> Self {
        Self {
            telemetry,
            entries: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn observe_dispatches(&self, dispatches: &[RouteDispatch]) {
        let mut entries = self.entries.lock().expect("recovery cache mutex poisoned");

        for dispatch in dispatches {
            if dispatch.cache.policy != CachePolicy::LastValuePerAddress {
                continue;
            }
            if !dispatch
                .packet
                .capabilities
                .contains(rosc_packet::PacketCapabilities::CACHEABLE_CANDIDATE)
            {
                continue;
            }

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

            self.telemetry.emit(BrokerEvent::CacheStored {
                route_id: dispatch.route_id.clone(),
            });
        }

        self.emit_entry_metrics(&entries);
    }

    pub fn rehydrate(&self, request: RehydrateRequest) -> Result<RehydrateOutcome, RecoveryError> {
        if request.route_id.is_none() && request.destination_id.is_none() {
            return Err(RecoveryError::MissingSelector);
        }

        let now = SystemTime::now();
        let mut entries = self.entries.lock().expect("recovery cache mutex poisoned");
        let keys = entries.keys().cloned().collect::<Vec<_>>();

        let mut route_counts = BTreeMap::<(String, String), usize>::new();
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
                    stale_keys.push(key.clone());
                    continue;
                }

                dispatches.push(entry.dispatch.clone());
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

        for ((route_id, destination_id), count) in route_counts {
            self.telemetry.emit(BrokerEvent::RecoveryRehydrate {
                route_id,
                destination_id,
                count,
            });
        }

        self.emit_entry_metrics(&entries);

        Ok(RehydrateOutcome {
            dispatches,
            stale_evictions: stale_keys.len(),
        })
    }

    pub fn cached_routes(&self) -> BTreeSet<String> {
        self.entries
            .lock()
            .expect("recovery cache mutex poisoned")
            .keys()
            .map(|key| key.route_id.clone())
            .collect()
    }

    fn emit_entry_metrics(&self, entries: &BTreeMap<CacheKey, CacheEntry>) {
        let mut by_route = BTreeMap::<String, usize>::new();
        for key in entries.keys() {
            *by_route.entry(key.route_id.clone()).or_default() += 1;
        }

        for (route_id, entries) in by_route {
            self.telemetry
                .emit(BrokerEvent::CacheEntriesChanged { route_id, entries });
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
