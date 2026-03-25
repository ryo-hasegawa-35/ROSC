use std::sync::Arc;

use rosc_runtime::serve_health_http_once;
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn health_endpoint_serves_healthz_and_metrics() {
    let telemetry = InMemoryTelemetry::default();
    telemetry.emit(BrokerEvent::RouteMatched {
        route_id: "camera_fov".to_owned(),
    });
    telemetry.emit(BrokerEvent::LaunchProfileChanged {
        mode: "safe_mode".to_owned(),
        disabled_capture_routes: 1,
        disabled_replay_routes: 0,
        disabled_restart_rehydrate_routes: 1,
    });
    let reporter = Arc::new(telemetry.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn({
        let reporter = reporter.clone();
        async move {
            serve_health_http_once(&listener, reporter).await.unwrap();
        }
    });

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("rosc_route_matches_total{route_id=\"camera_fov\"} 1"));
    assert!(response.contains("rosc_launch_profile_mode{mode=\"safe_mode\"} 1"));
    server.await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn({
        let reporter = reporter.clone();
        async move {
            serve_health_http_once(&listener, reporter).await.unwrap();
        }
    });

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.ends_with("ok\n"));
    server.await.unwrap();
}
