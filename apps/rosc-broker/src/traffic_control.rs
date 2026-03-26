use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::watch;

#[derive(Clone)]
pub struct TrafficControlState {
    frozen: Arc<AtomicBool>,
    state_tx: watch::Sender<bool>,
}

impl Default for TrafficControlState {
    fn default() -> Self {
        let (state_tx, _) = watch::channel(false);
        Self {
            frozen: Arc::new(AtomicBool::new(false)),
            state_tx,
        }
    }
}

impl TrafficControlState {
    pub fn is_frozen(&self) -> bool {
        self.frozen.load(Ordering::Relaxed)
    }

    pub fn freeze(&self) {
        self.state_tx.send_replace(true);
        self.frozen.store(true, Ordering::Relaxed);
    }

    pub fn thaw(&self) {
        self.frozen.store(false, Ordering::Relaxed);
        self.state_tx.send_replace(false);
    }

    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.state_tx.subscribe()
    }

    #[cfg(test)]
    pub async fn wait_until_thawed(&self) {
        if !self.is_frozen() {
            return;
        }

        let mut rx = self.state_tx.subscribe();
        while *rx.borrow_and_update() {
            if rx.changed().await.is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TrafficControlState;

    #[tokio::test]
    async fn wait_until_thawed_returns_after_thaw() {
        let control = TrafficControlState::default();
        control.freeze();

        let waiter = tokio::spawn({
            let control = control.clone();
            async move {
                control.wait_until_thawed().await;
            }
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        control.thaw();

        tokio::time::timeout(Duration::from_secs(1), waiter)
            .await
            .expect("waiter should finish after thaw")
            .expect("waiter task should succeed");
    }

    #[tokio::test]
    async fn wait_until_thawed_returns_immediately_when_already_thawed() {
        let control = TrafficControlState::default();

        tokio::time::timeout(Duration::from_secs(1), control.wait_until_thawed())
            .await
            .expect("already-thawed wait should finish immediately");
    }
}
