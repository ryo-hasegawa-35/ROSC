use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecentOperatorAction {
    pub sequence: u64,
    pub recorded_at_unix_ms: u64,
    pub action: String,
    pub details: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecentConfigEventKind {
    Applied,
    Rejected,
    Blocked,
    ReloadFailed,
    LaunchProfileChanged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecentConfigEvent {
    pub sequence: u64,
    pub recorded_at_unix_ms: u64,
    pub kind: RecentConfigEventKind,
    pub revision: Option<u64>,
    pub details: Vec<String>,
    pub added_ingresses: usize,
    pub removed_ingresses: usize,
    pub changed_ingresses: usize,
    pub added_destinations: usize,
    pub removed_destinations: usize,
    pub changed_destinations: usize,
    pub added_routes: usize,
    pub removed_routes: usize,
    pub changed_routes: usize,
    pub launch_profile_mode: Option<String>,
    pub disabled_capture_routes: usize,
    pub disabled_replay_routes: usize,
    pub disabled_restart_rehydrate_routes: usize,
}
