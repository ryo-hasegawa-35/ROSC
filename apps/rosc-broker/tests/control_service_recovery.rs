mod common;

use std::time::Duration;

use common::control_service::{
    custom_id_proxy_config, json_body, proxy_config, replayable_proxy_config, request, send_packet,
    start_proxy, start_service,
};
use rosc_osc::{ParsedOscPacket, parse_packet};
use tokio::net::UdpSocket;

#[tokio::test]
async fn control_service_can_isolate_routes_and_report_unknown_routes() {
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
async fn control_service_can_restore_all_isolated_routes() {
    let destination_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let _ = request(
        service.listen_addr(),
        "POST /routes/camera/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;

    let restore_response = request(
        service.listen_addr(),
        "POST /routes/restore-all HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(restore_response.contains("HTTP/1.1 200 OK"));
    assert!(restore_response.contains("\"action\":\"restore_all_routes\""));
    assert!(restore_response.contains("\"dispatch_count\":1"));
    assert!(restore_response.contains("\"isolated_route_ids\":[]"));

    let operator_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/operator-actions HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let actions = operator_history["actions"].as_array().unwrap();
    assert!(actions.iter().any(|action| {
        action["action"] == "restore_all_routes"
            && action["details"]
                == serde_json::json!(["restored_count=1", "route_ids=camera", "applied=true"])
    }));

    send_packet(ingress_addr).await;
    let mut buffer = [0u8; 2048];
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        destination_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_can_rehydrate_and_replay_to_sandbox() {
    let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        replayable_proxy_config(
            "127.0.0.1:0",
            &live_listener.local_addr().unwrap().to_string(),
            &sandbox_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

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
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
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

    let operator_history = json_body(
        &request(
            service.listen_addr(),
            "GET /history/operator-actions HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await,
    );
    let actions = operator_history["actions"].as_array().unwrap();
    assert!(actions.iter().any(|action| {
        action["action"] == "rehydrate_destination"
            && action["details"]
                == serde_json::json!([
                    "destination_id=udp_renderer",
                    "dispatch_count=1",
                    "applied=true"
                ])
    }));
    assert!(actions.iter().any(|action| {
        action["action"] == "sandbox_replay"
            && action["details"]
                == serde_json::json!([
                    "route_id=camera",
                    "sandbox_destination_id=sandbox_tap",
                    "limit=1",
                    "dispatch_count=1",
                    "applied=true"
                ])
    }));

    let (size, _) = tokio::time::timeout(
        Duration::from_secs(1),
        sandbox_listener.recv_from(&mut buffer),
    )
    .await
    .unwrap()
    .unwrap();
    let parsed = parse_packet(&buffer[..size], rosc_osc::CompatibilityMode::Osc1_0Strict).unwrap();
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
    let route_id = "camera/main?1";
    let destination_id = "udp/renderer?1";
    let proxy = start_proxy(
        custom_id_proxy_config(
            "127.0.0.1:0",
            &destination_listener.local_addr().unwrap().to_string(),
            destination_id,
            route_id,
        ),
        32,
    )
    .await;
    let ingress_addr = proxy
        .lock()
        .await
        .app()
        .ingress_local_addr("udp_localhost_in")
        .unwrap();
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

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
    assert!(blocked.is_err());

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

#[tokio::test]
async fn control_service_rejects_invalid_percent_encoding() {
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

    let response = request(
        service.listen_addr(),
        "POST /routes/camera%2/isolate HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    assert!(response.contains("invalid percent-encoding in route id"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}

#[tokio::test]
async fn control_service_rejects_invalid_replay_limit() {
    let live_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sandbox_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let proxy = start_proxy(
        replayable_proxy_config(
            "127.0.0.1:0",
            &live_listener.local_addr().unwrap().to_string(),
            &sandbox_listener.local_addr().unwrap().to_string(),
        ),
        32,
    )
    .await;
    let mut service = start_service(&proxy, "127.0.0.1:0").await;

    let zero_response = request(
        service.listen_addr(),
        "POST /routes/camera/replay/sandbox_tap?limit=0 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(zero_response.contains("HTTP/1.1 400 Bad Request"));
    assert!(zero_response.contains("invalid query parameter `limit`"));

    let malformed_response = request(
        service.listen_addr(),
        "POST /routes/camera/replay/sandbox_tap?limit=1x HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .await;
    assert!(malformed_response.contains("HTTP/1.1 400 Bad Request"));
    assert!(malformed_response.contains("invalid query parameter `limit`"));

    service.shutdown().await.unwrap();
    proxy.lock().await.shutdown().await;
}
