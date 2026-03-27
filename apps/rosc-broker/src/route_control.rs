use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct RouteControlState {
    isolated_route_ids: Arc<RwLock<BTreeSet<String>>>,
}

impl RouteControlState {
    pub fn is_isolated(&self, route_id: &str) -> bool {
        self.isolated_route_ids
            .read()
            .expect("route control lock poisoned")
            .contains(route_id)
    }

    pub fn isolate(&self, route_id: impl Into<String>) -> bool {
        self.isolated_route_ids
            .write()
            .expect("route control lock poisoned")
            .insert(route_id.into())
    }

    pub fn restore(&self, route_id: &str) -> bool {
        self.isolated_route_ids
            .write()
            .expect("route control lock poisoned")
            .remove(route_id)
    }

    pub fn snapshot(&self) -> Vec<String> {
        self.isolated_route_ids
            .read()
            .expect("route control lock poisoned")
            .iter()
            .cloned()
            .collect()
    }
}
