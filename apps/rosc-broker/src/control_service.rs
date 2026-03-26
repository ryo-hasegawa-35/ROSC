use std::io;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::UdpProxyStatusSnapshot;
use crate::control_plane::{ControlPlaneActionResult, ControlPlaneError, ProxyControlPlane};

pub struct ControlService {
    listen_addr: std::net::SocketAddr,
    shutdown: Option<watch::Sender<bool>>,
    task: Option<JoinHandle<io::Result<()>>>,
}

impl ControlService {
    pub async fn spawn(listen: &str, control: Arc<dyn ProxyControlPlane>) -> Result<Self> {
        let listener = TcpListener::bind(listen)
            .await
            .with_context(|| format!("failed to bind control listener on {listen}"))?;
        let listen_addr = listener.local_addr()?;
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    result = serve_control_http_once(&listener, Arc::clone(&control)) => {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct HttpRequest {
    method: String,
    path: String,
}

#[derive(Serialize)]
struct StatusResponse {
    ok: bool,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
struct ActionResponse {
    ok: bool,
    action: &'static str,
    applied: bool,
    status: UdpProxyStatusSnapshot,
}

#[derive(Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

async fn serve_control_http_once(
    listener: &TcpListener,
    control: Arc<dyn ProxyControlPlane>,
) -> io::Result<()> {
    let (mut stream, _) = listener.accept().await?;
    let request = match read_http_request(&mut stream).await {
        Ok(request) => request,
        Err(error) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                &ResponseBody::Error(ErrorResponse {
                    ok: false,
                    error: error.to_string(),
                }),
            )
            .await?;
            return Ok(());
        }
    };

    let response = route_request(request, control).await;
    write_json_response(&mut stream, response.status, &response.body).await?;

    Ok(())
}

async fn route_request(request: HttpRequest, control: Arc<dyn ProxyControlPlane>) -> HttpResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/status") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Status(StatusResponse {
                ok: true,
                status: control.status_snapshot().await,
            }),
        },
        ("POST", "/freeze") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(
                "freeze_traffic",
                control.freeze_traffic().await,
            )),
        },
        ("POST", "/thaw") => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(
                "thaw_traffic",
                control.thaw_traffic().await,
            )),
        },
        _ => route_route_request(&request, control).await,
    }
}

async fn route_route_request(
    request: &HttpRequest,
    control: Arc<dyn ProxyControlPlane>,
) -> HttpResponse {
    let Some(route_id) = request.path.strip_prefix("/routes/") else {
        return HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unsupported control route {}", request.path),
            }),
        };
    };

    if let Some(route_id) = route_id.strip_suffix("/isolate") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        return map_route_result("isolate_route", control.isolate_route(route_id).await);
    }

    if let Some(route_id) = route_id.strip_suffix("/restore") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        return map_route_result("restore_route", control.restore_route(route_id).await);
    }

    HttpResponse {
        status: "404 Not Found",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("unsupported control route {}", request.path),
        }),
    }
}

fn method_or_path_error(request: &HttpRequest) -> HttpResponse {
    HttpResponse {
        status: "404 Not Found",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("unsupported control route {}", request.path),
        }),
    }
}

fn map_route_result(
    action: &'static str,
    result: Result<ControlPlaneActionResult, ControlPlaneError>,
) -> HttpResponse {
    match result {
        Ok(result) => HttpResponse {
            status: "200 OK",
            body: ResponseBody::Action(action_response(action, result)),
        },
        Err(ControlPlaneError::UnknownRoute(route_id)) => HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unknown route `{route_id}`"),
            }),
        },
    }
}

fn action_response(action: &'static str, result: ControlPlaneActionResult) -> ActionResponse {
    ActionResponse {
        ok: true,
        action,
        applied: result.applied,
        status: result.status,
    }
}

async fn read_http_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 8192 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request headers exceed 8192 bytes",
            ));
        }
    }

    let request = String::from_utf8_lossy(&buffer);
    let Some(request_line) = request.lines().next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request line",
        ));
    };

    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request method",
        ));
    };
    let Some(path) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request path",
        ));
    };

    Ok(HttpRequest {
        method: method.to_owned(),
        path: path.to_owned(),
    })
}

async fn write_json_response(
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

enum ResponseBody {
    Status(StatusResponse),
    Action(ActionResponse),
    Error(ErrorResponse),
}

struct HttpResponse {
    status: &'static str,
    body: ResponseBody,
}

impl ResponseBody {
    fn to_json(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Status(body) => serde_json::to_vec(body),
            Self::Action(body) => serde_json::to_vec(body),
            Self::Error(body) => serde_json::to_vec(body),
        }
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use rosc_config::BrokerConfig;
    use rosc_osc::{
        OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet, parse_packet,
    };
    use rosc_telemetry::InMemoryTelemetry;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpStream, UdpSocket};
    use tokio::sync::Mutex;

    use super::ControlService;
    use crate::{
        ManagedProxyStartupOptions, ManagedUdpProxy, ProxyLaunchProfileMode,
        ProxyRuntimeSafetyPolicy, control_plane::ManagedUdpProxyController,
    };

    fn proxy_config(ingress_bind: &str, destination_addr: &str) -> BrokerConfig {
        BrokerConfig::from_toml_str(&format!(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "{ingress_bind}"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "udp_renderer"
            bind = "127.0.0.1:0"
            target = "{destination_addr}"

            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["udp_localhost_in"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [routes.transform]
            rename_address = "/render/camera/fov"

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
        ))
        .unwrap()
    }

    async fn send_packet(target: std::net::SocketAddr) {
        let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let payload = encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: "/ue5/camera/fov".to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(80.0)],
        }))
        .unwrap();
        source.send_to(&payload, target).await.unwrap();
    }

    async fn request(addr: std::net::SocketAddr, raw_request: &str) -> String {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(raw_request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    #[tokio::test]
    async fn control_service_freezes_and_thaws_live_proxy() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let ingress_addr = reserved.local_addr().unwrap();
        drop(reserved);

        let proxy = Arc::new(Mutex::new(
            ManagedUdpProxy::start(
                proxy_config(
                    &ingress_addr.to_string(),
                    &destination_listener.local_addr().unwrap().to_string(),
                ),
                InMemoryTelemetry::default(),
                32,
                ProxyRuntimeSafetyPolicy::default(),
                ProxyLaunchProfileMode::Normal,
                ManagedProxyStartupOptions::default(),
            )
            .await
            .unwrap(),
        ));
        let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
        let mut service = ControlService::spawn("127.0.0.1:0", controller)
            .await
            .unwrap();

        let freeze_response = request(
            service.listen_addr(),
            "POST /freeze HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(freeze_response.contains("HTTP/1.1 200 OK"));
        assert!(freeze_response.contains("\"action\":\"freeze_traffic\""));
        assert!(freeze_response.contains("\"applied\":true"));

        send_packet(ingress_addr).await;
        let mut buffer = [0u8; 2048];
        let frozen = tokio::time::timeout(
            Duration::from_millis(200),
            destination_listener.recv_from(&mut buffer),
        )
        .await;
        assert!(frozen.is_err(), "frozen control should stop egress");

        let thaw_response = request(
            service.listen_addr(),
            "POST /thaw HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(thaw_response.contains("HTTP/1.1 200 OK"));
        assert!(thaw_response.contains("\"action\":\"thaw_traffic\""));

        send_packet(ingress_addr).await;
        let (size, _) = tokio::time::timeout(
            Duration::from_secs(1),
            destination_listener.recv_from(&mut buffer),
        )
        .await
        .unwrap()
        .unwrap();
        let parsed =
            parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
        let ParsedOscPacket::Message(message) = parsed else {
            panic!("expected OSC message after thaw");
        };
        assert_eq!(message.address, "/render/camera/fov");

        service.shutdown().await.unwrap();
        proxy.lock().await.shutdown().await;
    }

    #[tokio::test]
    async fn control_service_can_isolate_routes_and_report_unknown_routes() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let ingress_addr = reserved.local_addr().unwrap();
        drop(reserved);

        let proxy = Arc::new(Mutex::new(
            ManagedUdpProxy::start(
                proxy_config(
                    &ingress_addr.to_string(),
                    &destination_listener.local_addr().unwrap().to_string(),
                ),
                InMemoryTelemetry::default(),
                32,
                ProxyRuntimeSafetyPolicy::default(),
                ProxyLaunchProfileMode::Normal,
                ManagedProxyStartupOptions::default(),
            )
            .await
            .unwrap(),
        ));
        let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
        let mut service = ControlService::spawn("127.0.0.1:0", controller)
            .await
            .unwrap();

        let isolate_response = request(
            service.listen_addr(),
            "POST /routes/camera/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(isolate_response.contains("HTTP/1.1 200 OK"));
        assert!(isolate_response.contains("\"isolated_route_ids\":[\"camera\"]"));

        let status_response = request(
            service.listen_addr(),
            "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(status_response.contains("HTTP/1.1 200 OK"));
        assert!(status_response.contains("\"isolated_route_ids\":[\"camera\"]"));

        let missing_response = request(
            service.listen_addr(),
            "POST /routes/missing/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(missing_response.contains("HTTP/1.1 404 Not Found"));
        assert!(missing_response.contains("unknown route `missing`"));

        service.shutdown().await.unwrap();
        proxy.lock().await.shutdown().await;
    }
}
