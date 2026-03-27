mod common;

use std::sync::Arc;
use std::time::Duration;

use common::control_service::{
    open_partial_request, proxy_config, request, start_proxy, start_service,
};
use rosc_broker::{ControlService, ManagedUdpProxyController};
use tokio::io::AsyncReadExt;
use tokio::net::UdpSocket;

#[tokio::test]
async fn control_service_rejects_non_loopback_listener() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let controller = Arc::new(ManagedUdpProxyController::new(Arc::clone(&proxy)));
    let error = match ControlService::spawn("0.0.0.0:0", controller).await {
        Ok(_) => panic!("non-loopback control listener should be rejected"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("control listener must bind to a loopback address")
    );

    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_accepts_localhost_listener() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "localhost:0").await;
    assert!(service.listen_addr().ip().is_loopback());

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_slow_client_does_not_block_other_requests() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let slow_stream = open_partial_request(service.listen_addr()).await;
    let fast_response = tokio::time::timeout(
        Duration::from_millis(500),
        request(
            service.listen_addr(),
            "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
        ),
    )
    .await
    .expect("a slow client should not block later requests");
    assert!(fast_response.contains("HTTP/1.1 200 OK"));

    drop(slow_stream);
    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_shutdown_is_not_blocked_by_partial_request() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let _slow_stream = open_partial_request(service.listen_addr()).await;
    tokio::time::timeout(Duration::from_millis(500), service.shutdown())
        .await
        .expect("shutdown should not wait on a partial request")
        .unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_times_out_partial_request_and_recovers() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let mut slow_stream = open_partial_request(service.listen_addr()).await;
    let mut timeout_response = String::new();
    tokio::time::timeout(
        Duration::from_secs(3),
        slow_stream.read_to_string(&mut timeout_response),
    )
    .await
    .expect("partial request should receive a timeout response")
    .unwrap();
    assert!(timeout_response.contains("HTTP/1.1 408 Request Timeout"));
    assert!(timeout_response.contains("request headers not received within"));

    let fast_response = request(
        service.listen_addr(),
        "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(fast_response.contains("HTTP/1.1 200 OK"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}
