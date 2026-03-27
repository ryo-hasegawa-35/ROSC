use std::sync::Arc;

use anyhow::Result;
use rosc_runtime::{IngressQueue, IngressReceiver, QueuePolicy, Runtime};
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};
use tokio::sync::watch;

use super::UdpProxyApp;
use crate::route_control::RouteControlState;
use crate::traffic_control::TrafficControlState;

pub(super) async fn spawn_ingress_tasks(
    app: &mut UdpProxyApp,
    ingress_queue_depth: usize,
) -> Result<()> {
    if app.tasks.is_running() {
        anyhow::bail!("udp proxy ingress tasks are already running");
    }
    app.ensure_ingresses_bound().await?;

    let (queue, rx) = IngressQueue::new(QueuePolicy {
        max_depth: ingress_queue_depth,
    });
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let runtime = Arc::clone(&app.runtime);
    let recovery = Arc::clone(&app.recovery);
    let destinations = Arc::clone(&app.destinations);
    let traffic_control = app.traffic_control.clone();
    let route_control = app.route_control.clone();
    let dispatcher_traffic_control = traffic_control.clone();
    let dispatcher_shutdown = shutdown_rx.clone();
    app.tasks.dispatcher = Some(tokio::spawn(async move {
        run_dispatcher_loop(
            runtime,
            recovery,
            destinations,
            dispatcher_traffic_control,
            route_control,
            rx,
            dispatcher_shutdown,
        )
        .await;
    }));

    for (ingress_id, binding) in std::mem::take(&mut app.ingresses) {
        let queue = queue.clone();
        let telemetry = app.runtime.telemetry.clone();
        let traffic_control = traffic_control.clone();
        let mut ingress_shutdown = shutdown_rx.clone();
        app.tasks.ingresses.push(tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = ingress_shutdown.changed() => {
                        break;
                    }
                    packet = binding.recv_next() => {
                        match packet {
                            Ok(packet) => {
                                telemetry.emit(BrokerEvent::PacketAccepted {
                                    ingress_id: packet.metadata.ingress_id.clone(),
                                });
                                if traffic_control.is_frozen() {
                                    telemetry.emit(BrokerEvent::PacketDropped {
                                        ingress_id: ingress_id.clone(),
                                        reason: "traffic_frozen".to_owned(),
                                    });
                                    continue;
                                }
                                match queue.try_send(packet) {
                                    Ok(()) => {}
                                    Err(error) => telemetry.emit(BrokerEvent::PacketDropped {
                                        ingress_id: ingress_id.clone(),
                                        reason: error.to_string(),
                                    }),
                                }
                            }
                            Err(error) => {
                                telemetry.emit(BrokerEvent::PacketDropped {
                                    ingress_id: ingress_id.clone(),
                                    reason: error.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }));
    }

    app.tasks.shutdown = Some(shutdown_tx);
    Ok(())
}

pub(super) async fn ensure_ingresses_bound(app: &mut UdpProxyApp) -> Result<()> {
    if !app.ingresses.is_empty() {
        return Ok(());
    }

    app.ingresses = super::build::bind_ingresses_from_specs(&app.ingress_specs).await?;
    app.ingress_addrs = super::build::ingress_addresses(&app.ingresses)?;
    super::build::refresh_ingress_status(&mut app.status, &app.ingress_addrs);
    Ok(())
}

pub(super) async fn run_dispatcher_loop(
    runtime: Arc<Runtime<InMemoryTelemetry>>,
    recovery: Arc<rosc_recovery::RecoveryEngine<InMemoryTelemetry>>,
    destinations: Arc<rosc_runtime::DestinationRegistry>,
    traffic_control: TrafficControlState,
    route_control: RouteControlState,
    mut rx: IngressReceiver,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                break;
            }
            packet = rx.recv() => {
                let Some(packet) = packet else {
                    break;
                };

                if !wait_until_dispatch_allowed(&traffic_control, &mut shutdown).await {
                    break;
                }

                let outcome = runtime
                    .dispatch_routing_outcome(
                        filter_isolated_routes(runtime.route_outcome(&packet), &route_control),
                        &destinations,
                    )
                    .await;
                recovery.observe_dispatches(&outcome.successful_dispatches);
                for failure in outcome.failures {
                    runtime.telemetry.emit(BrokerEvent::DispatchFailed {
                        route_id: failure.route_id,
                        destination_id: failure.destination_id,
                        reason: failure.reason,
                    });
                }
            }
        }
    }
}

pub(super) fn filter_isolated_routes(
    mut outcome: rosc_route::RoutingOutcome,
    route_control: &RouteControlState,
) -> rosc_route::RoutingOutcome {
    outcome
        .dispatches
        .retain(|dispatch| !route_control.is_isolated(&dispatch.route_id));
    outcome
        .failures
        .retain(|failure| !route_control.is_isolated(&failure.route_id));
    outcome
}

async fn wait_until_dispatch_allowed(
    traffic_control: &TrafficControlState,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    let mut freeze_rx = traffic_control.subscribe();
    while *freeze_rx.borrow_and_update() {
        tokio::select! {
            biased;
            _ = shutdown.changed() => return false,
            changed = freeze_rx.changed() => {
                if changed.is_err() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{RouteControlState, TrafficControlState, run_dispatcher_loop};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use rosc_osc::{
        CompatibilityMode, OscArgument, OscMessage, ParsedOscPacket, TypeTagSource, encode_packet,
    };
    use rosc_packet::{IngressMetadata, PacketEnvelope, TransportKind};
    use rosc_recovery::RecoveryEngine;
    use rosc_route::RoutingEngine;
    use rosc_runtime::{
        BreakerPolicy, DestinationPolicy, DestinationRegistry, DestinationWorkerHandle, DropPolicy,
        IngressQueue, QueuePolicy, Runtime, UdpEgressSink,
    };
    use rosc_telemetry::InMemoryTelemetry;
    use tokio::net::UdpSocket;
    use tokio::sync::watch;

    #[tokio::test]
    async fn dispatcher_holds_queued_packet_while_traffic_is_frozen() {
        let config = rosc_config::BrokerConfig::from_toml_str(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "127.0.0.1:0"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "udp_renderer"
            bind = "127.0.0.1:0"
            target = "127.0.0.1:9001"

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
            "#,
        )
        .unwrap();

        let telemetry = InMemoryTelemetry::default();
        let runtime = Arc::new(Runtime {
            routing: RoutingEngine::new(config.routes).unwrap(),
            telemetry: telemetry.clone(),
        });
        let recovery = Arc::new(RecoveryEngine::new(telemetry));
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sink = Arc::new(
            UdpEgressSink::bind("127.0.0.1:0", listener.local_addr().unwrap())
                .await
                .unwrap(),
        );

        let mut destinations = DestinationRegistry::default();
        destinations.register(DestinationWorkerHandle::spawn(
            "udp_renderer",
            DestinationPolicy {
                queue_depth: 8,
                drop_policy: DropPolicy::DropOldest,
                breaker: BreakerPolicy::default(),
            },
            sink,
            Arc::new(InMemoryTelemetry::default()),
        ));
        let destinations = Arc::new(destinations);

        let traffic_control = TrafficControlState::default();
        traffic_control.freeze();

        let (queue, rx) = IngressQueue::new(QueuePolicy { max_depth: 8 });
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let dispatcher = tokio::spawn(run_dispatcher_loop(
            runtime,
            recovery,
            destinations,
            traffic_control.clone(),
            RouteControlState::default(),
            rx,
            shutdown_rx,
        ));

        let raw = encode_packet(&ParsedOscPacket::Message(OscMessage {
            address: "/ue5/camera/fov".to_owned(),
            type_tag_source: TypeTagSource::Explicit,
            arguments: vec![OscArgument::Float32(75.0)],
        }))
        .unwrap();
        let packet = PacketEnvelope::parse_osc(
            raw,
            IngressMetadata {
                ingress_id: "udp_localhost_in".to_owned(),
                transport: TransportKind::OscUdp,
                source_endpoint: None,
                received_at: SystemTime::UNIX_EPOCH,
                compatibility_mode: CompatibilityMode::Osc1_0Strict,
            },
        )
        .unwrap();
        queue.try_send(packet).unwrap();

        let mut buffer = [0u8; 2048];
        let blocked =
            tokio::time::timeout(Duration::from_millis(200), listener.recv_from(&mut buffer)).await;
        assert!(
            blocked.is_err(),
            "queued packet should not dispatch while frozen"
        );

        traffic_control.thaw();
        let (size, _) =
            tokio::time::timeout(Duration::from_secs(1), listener.recv_from(&mut buffer))
                .await
                .expect("queued packet should dispatch after thaw")
                .unwrap();
        assert!(size > 0);

        dispatcher.abort();
        let _ = dispatcher.await;
    }
}
