use std::io;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::control_plane::ProxyControlPlane;

use super::payloads::ResponseBody;

pub(crate) async fn serve_control_connection(
    stream: TcpStream,
    control: Arc<dyn ProxyControlPlane>,
) -> io::Result<()> {
    super::super::serve_control_connection_impl(stream, control).await
}

pub(crate) async fn validate_control_listen_target(listen: &str) -> anyhow::Result<()> {
    super::super::validate_control_listen_target_impl(listen).await
}

pub(crate) async fn write_json_response(
    stream: &mut TcpStream,
    status: &str,
    body: &ResponseBody,
) -> io::Result<()> {
    let payload = body.to_json()?;
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(headers.as_bytes()).await?;
    stream.write_all(&payload).await
}
