use std::sync::Arc;

use rosc_broker::HealthService;
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn request(listener_addr: std::net::SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(listener_addr).await.unwrap();
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        listener_addr
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8(response).unwrap()
}

#[tokio::test]
async fn health_service_serves_requests_and_releases_port_on_shutdown() {
    let telemetry = InMemoryTelemetry::default();
    telemetry.emit(BrokerEvent::PacketAccepted {
        ingress_id: "udp_localhost_in".to_owned(),
    });

    let mut service = HealthService::spawn("127.0.0.1:0", Arc::new(telemetry.clone()))
        .await
        .unwrap();
    let listen_addr = service.listen_addr();

    let healthz = request(listen_addr, "/healthz").await;
    assert!(healthz.contains("200 OK"));
    assert!(healthz.ends_with("ok\n"));

    let metrics = request(listen_addr, "/metrics").await;
    assert!(metrics.contains("rosc_ingress_packets_total"));

    service.shutdown().await.unwrap();

    let rebound = TcpListener::bind(listen_addr).await.unwrap();
    assert_eq!(rebound.local_addr().unwrap(), listen_addr);
}
