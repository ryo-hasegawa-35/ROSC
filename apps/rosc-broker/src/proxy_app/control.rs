use anyhow::{Context, Result};
use rosc_packet::PacketEnvelope;
use rosc_recovery::{RehydrateRequest, SandboxReplayRequest};
use rosc_telemetry::{BrokerEvent, TelemetrySink};

use super::UdpProxyApp;

pub(super) fn emit_launch_profile_event(app: &UdpProxyApp, revision: u64) {
    app.runtime
        .telemetry
        .emit(BrokerEvent::LaunchProfileChanged {
            revision,
            mode: app.status.launch_profile.mode.as_str().to_owned(),
            disabled_capture_routes: app.status.launch_profile.disabled_capture_routes.len(),
            disabled_replay_routes: app.status.launch_profile.disabled_replay_routes.len(),
            disabled_restart_rehydrate_routes: app
                .status
                .launch_profile
                .disabled_restart_rehydrate_routes
                .len(),
        });
}

pub(super) fn freeze_traffic(app: &UdpProxyApp) -> bool {
    let changed = !app.traffic_control.is_frozen();
    app.traffic_control.freeze();
    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "freeze_traffic".to_owned(),
        details: vec![format!("applied={changed}")],
    });
    if changed {
        app.runtime
            .telemetry
            .emit(BrokerEvent::TrafficFreezeChanged { frozen: true });
    }
    changed
}

pub(super) fn thaw_traffic(app: &UdpProxyApp) -> bool {
    let changed = app.traffic_control.is_frozen();
    app.traffic_control.thaw();
    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "thaw_traffic".to_owned(),
        details: vec![format!("applied={changed}")],
    });
    if changed {
        app.runtime
            .telemetry
            .emit(BrokerEvent::TrafficFreezeChanged { frozen: false });
    }
    changed
}

pub(super) fn restore_frozen_traffic(app: &UdpProxyApp) -> bool {
    if app.traffic_control.is_frozen() {
        return false;
    }
    app.traffic_control.freeze();
    app.runtime
        .telemetry
        .emit(BrokerEvent::TrafficFreezeChanged { frozen: true });
    true
}

pub(super) fn isolate_route(app: &UdpProxyApp, route_id: &str) -> bool {
    if !app.status.routes.iter().any(|route| route.id == route_id) {
        return false;
    }
    let changed = app.route_control.isolate(route_id.to_owned());
    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "isolate_route".to_owned(),
        details: vec![format!("route_id={route_id}"), format!("applied={changed}")],
    });
    if changed {
        app.runtime
            .telemetry
            .emit(BrokerEvent::RouteIsolationChanged {
                route_id: route_id.to_owned(),
                isolated: true,
            });
    }
    changed
}

pub(super) fn restore_route(app: &UdpProxyApp, route_id: &str) -> bool {
    if !app.status.routes.iter().any(|route| route.id == route_id) {
        return false;
    }
    let changed = app.route_control.restore(route_id);
    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "restore_route".to_owned(),
        details: vec![format!("route_id={route_id}"), format!("applied={changed}")],
    });
    if changed {
        app.runtime
            .telemetry
            .emit(BrokerEvent::RouteIsolationChanged {
                route_id: route_id.to_owned(),
                isolated: false,
            });
    }
    changed
}

pub(super) fn restore_all_routes(app: &UdpProxyApp) -> usize {
    let isolated_route_ids = app.route_control.snapshot();
    let mut restored_route_ids = Vec::new();
    for route_id in isolated_route_ids {
        if app.route_control.restore(&route_id) {
            app.runtime
                .telemetry
                .emit(BrokerEvent::RouteIsolationChanged {
                    route_id: route_id.clone(),
                    isolated: false,
                });
            restored_route_ids.push(route_id);
        }
    }

    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "restore_all_routes".to_owned(),
        details: vec![
            format!("restored_count={}", restored_route_ids.len()),
            format!("route_ids={}", restored_route_ids.join(",")),
            format!("applied={}", !restored_route_ids.is_empty()),
        ],
    });

    restored_route_ids.len()
}

pub(super) fn restore_route_isolation(app: &UdpProxyApp, route_id: &str) -> bool {
    if !app.status.routes.iter().any(|route| route.id == route_id) {
        return false;
    }
    let changed = app.route_control.isolate(route_id.to_owned());
    if changed {
        app.runtime
            .telemetry
            .emit(BrokerEvent::RouteIsolationChanged {
                route_id: route_id.to_owned(),
                isolated: true,
            });
    }
    changed
}

pub(super) async fn relay_once(app: &UdpProxyApp, ingress_id: &str) -> Result<usize> {
    let binding = app
        .ingresses
        .get(ingress_id)
        .with_context(|| format!("unknown ingress id {ingress_id}"))?;
    let packet = binding.recv_next().await?;
    app.runtime.telemetry.emit(BrokerEvent::PacketAccepted {
        ingress_id: packet.metadata.ingress_id.clone(),
    });
    if app.traffic_control.is_frozen() {
        app.runtime.telemetry.emit(BrokerEvent::PacketDropped {
            ingress_id: packet.metadata.ingress_id.clone(),
            reason: "traffic_frozen".to_owned(),
        });
        return Ok(0);
    }
    let outcome = app.dispatch_packet(&packet).await;
    app.recovery
        .observe_dispatches(&outcome.successful_dispatches);
    for failure in &outcome.failures {
        app.runtime.telemetry.emit(BrokerEvent::DispatchFailed {
            route_id: failure.route_id.clone(),
            destination_id: failure.destination_id.clone(),
            reason: failure.reason.clone(),
        });
    }
    Ok(outcome.dispatched)
}

pub(super) async fn rehydrate_destination(
    app: &UdpProxyApp,
    destination_id: &str,
) -> Result<usize> {
    let outcome = app.recovery.rehydrate(RehydrateRequest {
        route_id: None,
        destination_id: Some(destination_id.to_owned()),
    })?;

    let mut dispatched = 0usize;
    for dispatch in outcome.dispatches {
        if app.destinations.dispatch(dispatch).await.is_ok() {
            dispatched += 1;
        }
    }

    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "rehydrate_destination".to_owned(),
        details: vec![
            format!("destination_id={destination_id}"),
            format!("dispatch_count={dispatched}"),
            format!("applied={}", dispatched > 0),
        ],
    });

    Ok(dispatched)
}

pub(super) async fn replay_route_to_sandbox(
    app: &UdpProxyApp,
    route_id: &str,
    sandbox_destination_id: &str,
    limit: usize,
) -> Result<usize> {
    let outcome = app.recovery.sandbox_replay(SandboxReplayRequest {
        route_id: route_id.to_owned(),
        source_destination_id: None,
        sandbox_destination_id: sandbox_destination_id.to_owned(),
        limit,
    })?;

    let mut dispatched = 0usize;
    for dispatch in outcome.dispatches {
        if app.destinations.dispatch(dispatch).await.is_ok() {
            dispatched += 1;
        }
    }

    app.runtime.telemetry.emit(BrokerEvent::OperatorAction {
        action: "sandbox_replay".to_owned(),
        details: vec![
            format!("route_id={route_id}"),
            format!("sandbox_destination_id={sandbox_destination_id}"),
            format!("limit={limit}"),
            format!("dispatch_count={dispatched}"),
            format!("applied={}", dispatched > 0),
        ],
    });

    Ok(dispatched)
}

pub(super) async fn dispatch_packet(
    app: &UdpProxyApp,
    packet: &PacketEnvelope,
) -> rosc_runtime::DispatchOutcome {
    let outcome = app.runtime.route_outcome(packet);
    let filtered = super::dispatch::filter_isolated_routes(outcome, &app.route_control);
    app.runtime
        .dispatch_routing_outcome(filtered, &app.destinations)
        .await
}
