use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_config::BrokerConfig;
use rosc_runtime::{
    DestinationPolicy, DestinationRegistry, DestinationWorkerHandle, IngressQueue, QueuePolicy,
    Runtime, UdpEgressSink, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::{BrokerEvent, InMemoryTelemetry, TelemetrySink};

pub struct UdpProxyApp {
    runtime: Arc<Runtime<InMemoryTelemetry>>,
    destinations: Arc<DestinationRegistry>,
    ingresses: BTreeMap<String, UdpIngressBinding>,
}

impl UdpProxyApp {
    pub async fn from_config(config: &BrokerConfig, telemetry: InMemoryTelemetry) -> Result<Self> {
        config.validate_runtime_references()?;

        let routing = rosc_route::RoutingEngine::new(config.routes.clone())?;
        let runtime = Arc::new(Runtime {
            routing,
            telemetry: telemetry.clone(),
        });

        let mut destinations = DestinationRegistry::default();
        for destination in &config.udp_destinations {
            let target: SocketAddr = destination
                .target
                .parse()
                .with_context(|| format!("invalid udp target address {}", destination.target))?;
            let sink = Arc::new(UdpEgressSink::bind(&destination.bind, target).await?);
            destinations.register(DestinationWorkerHandle::spawn(
                destination.id.clone(),
                DestinationPolicy::default(),
                sink,
                Arc::new(telemetry.clone()),
            ));
        }

        let mut ingresses = BTreeMap::new();
        for ingress in &config.udp_ingresses {
            let binding = UdpIngressBinding::bind(
                &ingress.bind,
                UdpIngressConfig {
                    ingress_id: ingress.id.clone(),
                    compatibility_mode: ingress.mode,
                    max_packet_size: ingress.max_packet_size,
                },
            )
            .await?;
            ingresses.insert(ingress.id.clone(), binding);
        }

        Ok(Self {
            runtime,
            destinations: Arc::new(destinations),
            ingresses,
        })
    }

    pub fn ingress_local_addr(&self, ingress_id: &str) -> Option<SocketAddr> {
        self.ingresses
            .get(ingress_id)
            .and_then(|binding| binding.local_addr().ok())
    }

    pub async fn relay_once(&self, ingress_id: &str) -> Result<usize> {
        let binding = self
            .ingresses
            .get(ingress_id)
            .with_context(|| format!("unknown ingress id {ingress_id}"))?;
        let packet = binding.recv_next().await?;
        self.runtime.telemetry.emit(BrokerEvent::PacketAccepted {
            ingress_id: packet.metadata.ingress_id.clone(),
        });
        let outcome = self
            .runtime
            .dispatch_packet(&packet, &self.destinations)
            .await;
        for failure in &outcome.failures {
            self.runtime.telemetry.emit(BrokerEvent::PacketDropped {
                ingress_id: failure.destination_id.clone(),
                reason: failure.reason.clone(),
            });
        }
        Ok(outcome.dispatched)
    }

    pub async fn spawn_ingress_tasks(&mut self, ingress_queue_depth: usize) {
        let (queue, mut rx) = IngressQueue::new(QueuePolicy {
            max_depth: ingress_queue_depth,
        });

        let runtime = Arc::clone(&self.runtime);
        let destinations = Arc::clone(&self.destinations);
        tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                let outcome = runtime.dispatch_packet(&packet, &destinations).await;
                for failure in outcome.failures {
                    runtime.telemetry.emit(BrokerEvent::PacketDropped {
                        ingress_id: failure.destination_id,
                        reason: failure.reason,
                    });
                }
            }
        });

        for (ingress_id, binding) in std::mem::take(&mut self.ingresses) {
            let queue = queue.clone();
            let telemetry = self.runtime.telemetry.clone();
            tokio::spawn(async move {
                loop {
                    match binding.recv_next().await {
                        Ok(packet) => {
                            telemetry.emit(BrokerEvent::PacketAccepted {
                                ingress_id: packet.metadata.ingress_id.clone(),
                            });
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
            });
        }
    }
}
