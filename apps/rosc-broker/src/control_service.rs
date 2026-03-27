use std::io;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::{JoinHandle, JoinSet};

use crate::control_http::{serve_control_connection, validate_control_listen_target};
use crate::control_plane::ProxyControlPlane;

pub struct ControlService {
    listen_addr: std::net::SocketAddr,
    shutdown: Option<watch::Sender<bool>>,
    task: Option<JoinHandle<io::Result<()>>>,
}

impl ControlService {
    pub async fn spawn(listen: &str, control: Arc<dyn ProxyControlPlane>) -> Result<Self> {
        validate_control_listen_target(listen).await?;
        let listener = TcpListener::bind(listen)
            .await
            .with_context(|| format!("failed to bind control listener on {listen}"))?;
        let listen_addr = listener.local_addr()?;
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    Some(result) = connections.join_next(), if !connections.is_empty() => {
                        process_connection_task_result(result)?;
                    }
                    result = listener.accept() => {
                        let (stream, _) = result?;
                        let control = Arc::clone(&control);
                        connections.spawn(async move { serve_control_connection(stream, control).await });
                    }
                }
            }

            connections.abort_all();
            while let Some(result) = connections.join_next().await {
                process_connection_task_result(result)?;
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
                .context("control service task join failed")?
                .context("control service loop failed")?;
        }

        Ok(())
    }
}

impl Drop for ControlService {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
    }
}

fn process_connection_task_result(
    result: std::result::Result<io::Result<()>, tokio::task::JoinError>,
) -> io::Result<()> {
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) if is_connection_local_error(&error) => Ok(()),
        Ok(Err(error)) => Err(error),
        Err(error) if error.is_cancelled() => Ok(()),
        Err(error) => Err(io::Error::other(format!(
            "control connection task join failed: {error}"
        ))),
    }
}

fn is_connection_local_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::UnexpectedEof
            | io::ErrorKind::TimedOut
            | io::ErrorKind::WriteZero
    )
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::process_connection_task_result;

    #[test]
    fn connection_reset_is_treated_as_connection_local_failure() {
        let result = process_connection_task_result(Ok(Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "connection reset by peer",
        ))));
        assert!(result.is_ok());
    }
}
