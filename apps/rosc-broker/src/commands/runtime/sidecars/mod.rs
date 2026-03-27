use std::sync::Arc;

use anyhow::Result;
use rosc_telemetry::InMemoryTelemetry;
use tokio::sync::Mutex;

pub(super) async fn spawn_optional_health_service(
    health_listen: Option<&str>,
    telemetry: InMemoryTelemetry,
) -> Result<Option<rosc_broker::HealthService>> {
    match health_listen {
        Some(listen) => {
            let service = rosc_broker::HealthService::spawn(listen, Arc::new(telemetry)).await?;
            println!("health endpoint listening on {}", service.listen_addr());
            Ok(Some(service))
        }
        None => Ok(None),
    }
}

pub(super) async fn shutdown_optional_health_service(
    service: &mut Option<rosc_broker::HealthService>,
) -> Result<()> {
    if let Some(service) = service.as_mut() {
        service.shutdown().await?;
        println!("health endpoint stopped");
    }
    Ok(())
}

pub(super) async fn spawn_optional_control_service(
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<Option<rosc_broker::ControlService>> {
    match control_listen {
        Some(listen) => {
            let service = rosc_broker::ControlService::spawn(listen, control_plane).await?;
            println!("control endpoint listening on {}", service.listen_addr());
            Ok(Some(service))
        }
        None => Ok(None),
    }
}

pub(super) async fn shutdown_optional_control_service(
    service: &mut Option<rosc_broker::ControlService>,
) -> Result<()> {
    if let Some(service) = service.as_mut() {
        service.shutdown().await?;
        println!("control endpoint stopped");
    }
    Ok(())
}

pub(super) async fn start_managed_proxy_sidecars(
    proxy: &Arc<Mutex<rosc_broker::ManagedUdpProxy>>,
    telemetry: InMemoryTelemetry,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<(
    Option<rosc_broker::HealthService>,
    Option<rosc_broker::ControlService>,
)> {
    let mut health_service = match spawn_optional_health_service(health_listen, telemetry).await {
        Ok(service) => service,
        Err(error) => {
            proxy.lock().await.shutdown().await;
            return Err(error);
        }
    };

    let control_service = match spawn_optional_control_service(control_listen, control_plane).await
    {
        Ok(service) => service,
        Err(error) => {
            shutdown_optional_health_service(&mut health_service).await?;
            proxy.lock().await.shutdown().await;
            return Err(error);
        }
    };

    Ok((health_service, control_service))
}

pub(super) async fn start_supervisor_sidecars(
    supervisor: &Arc<Mutex<rosc_broker::ManagedProxyFileSupervisor>>,
    telemetry: InMemoryTelemetry,
    health_listen: Option<&str>,
    control_listen: Option<&str>,
    control_plane: Arc<dyn rosc_broker::ProxyControlPlane>,
) -> Result<(
    Option<rosc_broker::HealthService>,
    Option<rosc_broker::ControlService>,
)> {
    let mut health_service = match spawn_optional_health_service(health_listen, telemetry).await {
        Ok(service) => service,
        Err(error) => {
            supervisor.lock().await.shutdown().await;
            return Err(error);
        }
    };

    let control_service = match spawn_optional_control_service(control_listen, control_plane).await
    {
        Ok(service) => service,
        Err(error) => {
            shutdown_optional_health_service(&mut health_service).await?;
            supervisor.lock().await.shutdown().await;
            return Err(error);
        }
    };

    Ok((health_service, control_service))
}

#[cfg(test)]
mod tests;
