use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct TrafficControlState {
    frozen: Arc<AtomicBool>,
    thaw_notify: Arc<Notify>,
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
        self.thaw_notify.notify_waiters();
    }

    pub async fn wait_until_thawed(&self) {
        while self.is_frozen() {
            self.thaw_notify.notified().await;
        }
    }
}
