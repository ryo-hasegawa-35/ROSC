use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use anyhow::{Context, Result};
use rosc_config::{BrokerConfig, DropPolicyConfig};
use rosc_runtime::{
    BreakerPolicy, DestinationPolicy, DestinationRegistry, DestinationWorkerHandle, DropPolicy,
    UdpEgressSink, UdpIngressBinding, UdpIngressConfig,
};
use rosc_telemetry::InMemoryTelemetry;

use super::{IngressBindingSpec, ProxyRuntimeTasks, UdpProxyApp};
use crate::proxy_status::{UdpProxyStatusSnapshot, proxy_status_from_config};
use crate::route_control::RouteControlState;
use crate::traffic_control::TrafficControlState;

pub(super) async fn build_proxy_app(
    config: &BrokerConfig,
    telemetry: InMemoryTelemetry,
) -> Result<UdpProxyApp> {
    config.validate_runtime_references()?;

    let routing = rosc_route::RoutingEngine::new(config.routes.clone())?;
    let runtime = Arc::new(rosc_runtime::Runtime {
        routing,
        telemetry: telemetry.clone(),
    });
    let recovery = Arc::new(rosc_recovery::RecoveryEngine::new(telemetry.clone()));

    let ingresses = bind_ingresses(config).await?;
    let ingress_addrs = ingress_addresses(&ingresses)?;
    let ingress_specs = ingress_specs(&config.udp_ingresses, &ingress_addrs);
    let mut status = proxy_status_from_config(config)?;
    refresh_ingress_status(&mut status, &ingress_addrs);

    let destinations = build_destinations(config, &ingress_addrs, telemetry.clone()).await?;

    Ok(UdpProxyApp {
        runtime,
        recovery,
        destinations: Arc::new(destinations),
        traffic_control: TrafficControlState::default(),
        route_control: RouteControlState::default(),
        ingress_specs,
        ingress_addrs,
        ingresses,
        status,
        tasks: ProxyRuntimeTasks::default(),
    })
}

pub(super) async fn bind_ingresses(
    config: &BrokerConfig,
) -> Result<BTreeMap<String, UdpIngressBinding>> {
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
    Ok(ingresses)
}

pub(super) async fn bind_ingresses_from_specs(
    specs: &BTreeMap<String, IngressBindingSpec>,
) -> Result<BTreeMap<String, UdpIngressBinding>> {
    let mut ingresses = BTreeMap::new();
    for (ingress_id, spec) in specs {
        let binding = UdpIngressBinding::bind(&spec.bind_address, spec.config.clone()).await?;
        ingresses.insert(ingress_id.clone(), binding);
    }
    Ok(ingresses)
}

pub(super) fn ingress_specs(
    ingresses: &[rosc_config::UdpIngressConfig],
    ingress_addrs: &BTreeMap<String, SocketAddr>,
) -> BTreeMap<String, IngressBindingSpec> {
    ingresses
        .iter()
        .map(|ingress| {
            let bind_address = ingress_addrs
                .get(&ingress.id)
                .map(ToString::to_string)
                .unwrap_or_else(|| ingress.bind.clone());
            (
                ingress.id.clone(),
                IngressBindingSpec {
                    bind_address,
                    config: UdpIngressConfig {
                        ingress_id: ingress.id.clone(),
                        compatibility_mode: ingress.mode,
                        max_packet_size: ingress.max_packet_size,
                    },
                },
            )
        })
        .collect()
}

pub(super) fn ingress_addresses(
    ingresses: &BTreeMap<String, UdpIngressBinding>,
) -> Result<BTreeMap<String, SocketAddr>> {
    ingresses
        .iter()
        .map(|(ingress_id, binding)| {
            binding
                .local_addr()
                .map(|addr| (ingress_id.clone(), addr))
                .with_context(|| {
                    format!("failed to inspect local address for ingress `{ingress_id}`")
                })
        })
        .collect()
}

pub(super) fn refresh_ingress_status(
    status: &mut UdpProxyStatusSnapshot,
    ingress_addrs: &BTreeMap<String, SocketAddr>,
) {
    for ingress in &mut status.ingresses {
        ingress.bound_local_addr = ingress_addrs.get(&ingress.id).map(ToString::to_string);
    }
}

pub(super) async fn build_destinations(
    config: &BrokerConfig,
    ingress_addrs: &BTreeMap<String, SocketAddr>,
    telemetry: InMemoryTelemetry,
) -> Result<DestinationRegistry> {
    let mut destinations = DestinationRegistry::default();

    for destination in &config.udp_destinations {
        let target: SocketAddr = destination
            .target
            .parse()
            .with_context(|| format!("invalid udp target address {}", destination.target))?;
        if let Some((ingress_id, ingress_addr)) = ingress_addrs
            .iter()
            .find(|(_, ingress_addr)| ingress_receives_target(**ingress_addr, target))
        {
            anyhow::bail!(
                "udp destination `{}` targets ingress `{}` at {}; refusing proxy self-loop",
                destination.id,
                ingress_id,
                ingress_addr
            );
        }

        let sink = Arc::new(UdpEgressSink::bind(&destination.bind, target).await?);
        destinations.register(DestinationWorkerHandle::spawn(
            destination.id.clone(),
            destination_policy(destination),
            sink,
            Arc::new(telemetry.clone()),
        ));
    }

    Ok(destinations)
}

fn destination_policy(destination: &rosc_config::UdpDestinationConfig) -> DestinationPolicy {
    DestinationPolicy {
        queue_depth: destination.policy.queue_depth,
        drop_policy: match destination.policy.drop_policy {
            DropPolicyConfig::DropNewest => DropPolicy::DropNewest,
            DropPolicyConfig::DropOldest => DropPolicy::DropOldest,
        },
        breaker: BreakerPolicy {
            open_after_consecutive_failures: destination
                .policy
                .breaker
                .open_after_consecutive_failures,
            open_after_consecutive_queue_overflows: destination
                .policy
                .breaker
                .open_after_consecutive_queue_overflows,
            cooldown: std::time::Duration::from_millis(destination.policy.breaker.cooldown_ms),
        },
    }
}

fn ingress_receives_target(ingress_addr: SocketAddr, target: SocketAddr) -> bool {
    if ingress_addr.port() != target.port() {
        return false;
    }

    if ingress_addr.ip() == target.ip() {
        return true;
    }

    match (ingress_addr.ip(), target.ip()) {
        (IpAddr::V4(ingress_ip), IpAddr::V4(target_ip)) => {
            ingress_ip.is_unspecified() && (target_ip.is_loopback() || target_ip.is_unspecified())
        }
        (IpAddr::V6(ingress_ip), IpAddr::V6(target_ip)) => {
            ingress_ip.is_unspecified() && (target_ip.is_loopback() || target_ip.is_unspecified())
        }
        _ => false,
    }
}
