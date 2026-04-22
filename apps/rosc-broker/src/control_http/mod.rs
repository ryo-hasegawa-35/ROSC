mod dashboard;
mod request;
mod response;
mod routes;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};
use tokio::net::TcpStream;

use crate::control_plane::ProxyControlPlane;

#[cfg(test)]
const CONTROL_REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(100);
#[cfg(not(test))]
const CONTROL_REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) use response::{serve_control_connection, validate_control_listen_target};

async fn serve_control_connection_impl(
    mut stream: TcpStream,
    control: Arc<dyn ProxyControlPlane>,
) -> io::Result<()> {
    let request = match tokio::time::timeout(
        CONTROL_REQUEST_READ_TIMEOUT,
        request::read_http_request(&mut stream),
    )
    .await
    {
        Ok(Ok(request)) => request,
        Ok(Err(error)) => {
            response::write_json_response(
                &mut stream,
                "400 Bad Request",
                &response::ResponseBody::error(error.to_string()),
            )
            .await?;
            return Ok(());
        }
        Err(_) => {
            response::write_json_response(
                &mut stream,
                "408 Request Timeout",
                &response::ResponseBody::error(format!(
                    "request headers not received within {} ms",
                    CONTROL_REQUEST_READ_TIMEOUT.as_millis()
                )),
            )
            .await?;
            return Ok(());
        }
    };

    let response = routes::route_request(request, control).await;
    response::write_response(&mut stream, response.status, &response.body).await?;
    Ok(())
}

async fn validate_control_listen_target_impl(listen: &str) -> Result<()> {
    let mut resolved_any = false;
    for addr in tokio::net::lookup_host(listen)
        .await
        .with_context(|| format!("failed to resolve control listener on {listen}"))?
    {
        resolved_any = true;
        ensure!(
            addr.ip().is_loopback(),
            "control listener must bind to a loopback address, got {addr}"
        );
    }

    if !resolved_any {
        return Err(anyhow!(
            "failed to resolve control listener on {listen}: no socket addresses"
        ));
    }

    Ok(())
}
