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
    dispatch_count: Option<usize>,
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
        _ => route_nested_request(&request, control).await,
    }
}

async fn route_nested_request(
    request: &HttpRequest,
    control: Arc<dyn ProxyControlPlane>,
) -> HttpResponse {
    if let Some(destination_id) = request
        .path
        .strip_prefix("/destinations/")
        .and_then(|path| path.strip_suffix("/rehydrate"))
    {
        if request.method != "POST" || destination_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(destination_id) = decode_uri_component(destination_id) else {
            return invalid_component_error("destination id");
        };
        return map_route_result(
            "rehydrate_destination",
            control.rehydrate_destination(&destination_id).await,
        );
    }

    let Some(route_path) = request.path.strip_prefix("/routes/") else {
        return HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unsupported control route {}", request.path),
            }),
        };
    };
    let (route_path, query) = split_query(route_path);

    if let Some(route_id) = route_path.strip_suffix("/isolate") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_route_result("isolate_route", control.isolate_route(&route_id).await);
    }

    if let Some(route_id) = route_path.strip_suffix("/restore") {
        if request.method != "POST" || route_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        return map_route_result("restore_route", control.restore_route(&route_id).await);
    }

    if let Some((route_id, sandbox_destination_id)) = route_path.split_once("/replay/") {
        if request.method != "POST" || route_id.is_empty() || sandbox_destination_id.is_empty() {
            return method_or_path_error(request);
        }
        let Ok(route_id) = decode_uri_component(route_id) else {
            return invalid_component_error("route id");
        };
        let Ok(sandbox_destination_id) = decode_uri_component(sandbox_destination_id) else {
            return invalid_component_error("sandbox destination id");
        };
        let limit = replay_limit(query);
        return map_route_result(
            "sandbox_replay",
            control
                .replay_route_to_sandbox(&route_id, &sandbox_destination_id, limit)
                .await,
        );
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
        Err(ControlPlaneError::UnknownDestination(destination_id)) => HttpResponse {
            status: "404 Not Found",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: format!("unknown destination `{destination_id}`"),
            }),
        },
        Err(ControlPlaneError::ActionFailed(message)) => HttpResponse {
            status: "422 Unprocessable Entity",
            body: ResponseBody::Error(ErrorResponse {
                ok: false,
                error: message,
            }),
        },
    }
}

fn action_response(action: &'static str, result: ControlPlaneActionResult) -> ActionResponse {
    ActionResponse {
        ok: true,
        action,
        applied: result.applied,
        dispatch_count: result.dispatch_count,
        status: result.status,
    }
}

fn split_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (path, None),
    }
}

fn replay_limit(query: Option<&str>) -> usize {
    query
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == "limit").then_some(value)
            })
        })
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(100)
}

fn decode_uri_component(component: &str) -> Result<String, ()> {
    let bytes = component.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(());
                }
                let high = decode_hex_nibble(bytes[index + 1]).ok_or(())?;
                let low = decode_hex_nibble(bytes[index + 2]).ok_or(())?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).map_err(|_| ())
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn invalid_component_error(label: &str) -> HttpResponse {
    HttpResponse {
        status: "400 Bad Request",
        body: ResponseBody::Error(ErrorResponse {
            ok: false,
            error: format!("invalid percent-encoding in {label}"),
        }),
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

    fn replayable_proxy_config(
        ingress_bind: &str,
        destination_addr: &str,
        sandbox_addr: &str,
    ) -> BrokerConfig {
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

            [[udp_destinations]]
            id = "sandbox_tap"
            bind = "127.0.0.1:0"
            target = "{sandbox_addr}"

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

            [routes.cache]
            policy = "last_value_per_address"
            ttl_ms = 10000
            persist = "warm"

            [routes.recovery]
            late_joiner = "latest"
            rehydrate_on_connect = true
            replay_allowed = true

            [routes.observability]
            capture = "always_bounded"

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#
        ))
        .unwrap()
    }

    fn custom_id_proxy_config(
        ingress_bind: &str,
        destination_addr: &str,
        destination_id: &str,
        route_id: &str,
    ) -> BrokerConfig {
        BrokerConfig::from_toml_str(&format!(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "{ingress_bind}"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "{destination_id}"
            bind = "127.0.0.1:0"
            target = "{destination_addr}"

            [[routes]]
            id = "{route_id}"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["udp_localhost_in"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [routes.transform]
            rename_address = "/render/camera/fov"

            [routes.cache]
            policy = "last_value_per_address"
            ttl_ms = 10000
            persist = "warm"

            [routes.recovery]
            late_joiner = "latest"
            rehydrate_on_connect = true

            [[routes.destinations]]
            target = "{destination_id}"
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

    #[tokio::test]
    async fn control_service_can_rehydrate_and_replay_to_sandbox() {
        let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let ingress_addr = reserved.local_addr().unwrap();
        drop(reserved);

        let proxy = Arc::new(Mutex::new(
            ManagedUdpProxy::start(
                replayable_proxy_config(
                    &ingress_addr.to_string(),
                    &live_listener.local_addr().unwrap().to_string(),
                    &sandbox_listener.local_addr().unwrap().to_string(),
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

        send_packet(ingress_addr).await;
        let mut buffer = [0u8; 2048];
        let _ = tokio::time::timeout(Duration::from_secs(1), live_listener.recv_from(&mut buffer))
            .await
            .unwrap()
            .unwrap();

        let rehydrate_response = request(
            service.listen_addr(),
            "POST /destinations/udp_renderer/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(rehydrate_response.contains("HTTP/1.1 200 OK"));
        assert!(rehydrate_response.contains("\"action\":\"rehydrate_destination\""));
        assert!(rehydrate_response.contains("\"dispatch_count\":1"));

        let (size, _) =
            tokio::time::timeout(Duration::from_secs(1), live_listener.recv_from(&mut buffer))
                .await
                .unwrap()
                .unwrap();
        let parsed =
            parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
        let ParsedOscPacket::Message(message) = parsed else {
            panic!("expected rehydrated OSC message");
        };
        assert_eq!(message.address, "/render/camera/fov");

        let replay_response = request(
            service.listen_addr(),
            "POST /routes/camera/replay/sandbox_tap?limit=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(replay_response.contains("HTTP/1.1 200 OK"));
        assert!(replay_response.contains("\"action\":\"sandbox_replay\""));
        assert!(replay_response.contains("\"dispatch_count\":1"));

        let (size, _) = tokio::time::timeout(
            Duration::from_secs(1),
            sandbox_listener.recv_from(&mut buffer),
        )
        .await
        .unwrap()
        .unwrap();
        let parsed =
            parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
        let ParsedOscPacket::Message(message) = parsed else {
            panic!("expected sandbox replay OSC message");
        };
        assert_eq!(message.address, "/render/camera/fov");

        let unknown_destination_response = request(
            service.listen_addr(),
            "POST /destinations/missing/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(unknown_destination_response.contains("HTTP/1.1 404 Not Found"));
        assert!(unknown_destination_response.contains("unknown destination `missing`"));

        service.shutdown().await.unwrap();
        proxy.lock().await.shutdown().await;
    }

    #[tokio::test]
    async fn control_service_decodes_percent_encoded_route_and_destination_ids() {
        let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let reserved = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let ingress_addr = reserved.local_addr().unwrap();
        drop(reserved);

        let route_id = "camera/main?1";
        let destination_id = "udp/renderer?1";
        let proxy = Arc::new(Mutex::new(
            ManagedUdpProxy::start(
                custom_id_proxy_config(
                    &ingress_addr.to_string(),
                    &destination_listener.local_addr().unwrap().to_string(),
                    destination_id,
                    route_id,
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
            "POST /routes/camera%2Fmain%3F1/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(isolate_response.contains("HTTP/1.1 200 OK"));
        assert!(isolate_response.contains("\"isolated_route_ids\":[\"camera/main?1\"]"));

        send_packet(ingress_addr).await;
        let mut buffer = [0u8; 2048];
        let blocked = tokio::time::timeout(
            Duration::from_millis(200),
            destination_listener.recv_from(&mut buffer),
        )
        .await;
        assert!(
            blocked.is_err(),
            "encoded route isolation should block dispatch"
        );

        let restore_response = request(
            service.listen_addr(),
            "POST /routes/camera%2Fmain%3F1/restore HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(restore_response.contains("HTTP/1.1 200 OK"));

        send_packet(ingress_addr).await;
        let _ = tokio::time::timeout(
            Duration::from_secs(1),
            destination_listener.recv_from(&mut buffer),
        )
        .await
        .unwrap()
        .unwrap();

        let rehydrate_response = request(
            service.listen_addr(),
            "POST /destinations/udp%2Frenderer%3F1/rehydrate HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(rehydrate_response.contains("HTTP/1.1 200 OK"));
        assert!(rehydrate_response.contains("\"dispatch_count\":1"));

        service.shutdown().await.unwrap();
        proxy.lock().await.shutdown().await;
    }
}
