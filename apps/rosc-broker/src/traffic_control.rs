use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct TrafficControlState {
    frozen: Arc<AtomicBool>,
}

impl TrafficControlState {
    pub fn is_frozen(&self) -> bool {
        self.frozen.load(Ordering::Relaxed)
    }

    pub fn freeze(&self) {
        self.frozen.store(true, Ordering::Relaxed);
    }

    pub fn thaw(&self) {
        self.frozen.store(false, Ordering::Relaxed);
    }
}
