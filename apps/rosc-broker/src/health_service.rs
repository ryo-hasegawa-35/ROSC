use std::io;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_telemetry::HealthReporter;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub struct HealthService {
    listen_addr: std::net::SocketAddr,
    shutdown: Option<watch::Sender<bool>>,
    task: Option<JoinHandle<io::Result<()>>>,
}

impl HealthService {
    pub async fn spawn(listen: &str, reporter: Arc<dyn HealthReporter>) -> Result<Self> {
        let listener = TcpListener::bind(listen)
            .await
            .with_context(|| format!("failed to bind health listener on {listen}"))?;
        let listen_addr = listener.local_addr()?;
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    result = rosc_runtime::serve_health_http_once(&listener, Arc::clone(&reporter)) => {
                        result?;
                    }
                }
            }
            Ok(())
        });

        Ok(Self {
            listen_addr,
            shutdown: Some(shutdown_tx),
            task: Some(task),
        })
    }

    pub fn listen_addr(&self) -> std::net::SocketAddr {
        self.listen_addr
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }

        if let Some(task) = self.task.take() {
            task.await
                .context("health service task join failed")?
                .context("health service loop failed")?;
        }

        Ok(())
    }
}

impl Drop for HealthService {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
    }
}
