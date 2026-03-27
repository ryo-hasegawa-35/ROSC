use anyhow::{Context, Result};
use rosc_config::BrokerConfig;
use rosc_telemetry::{InMemoryTelemetry, TelemetrySink};

use crate::UdpProxyStatusSnapshot;
use crate::{
    ProxyLaunchProfileMode, ProxyOperatorDiagnostics, ProxyOperatorIncidents,
    ProxyOperatorOverview, ProxyOperatorReport, ProxyOperatorSnapshot, ProxyRuntimeSafetyPolicy,
    UdpProxyApp, apply_launch_profile, emit_config_transition,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FrozenStartupBehavior {
    #[default]
    Normal,
    OperatorRequested,
    Restored,
}

#[derive(Clone, Debug, Default)]
pub struct ManagedProxyStartupOptions {
    pub frozen_behavior: FrozenStartupBehavior,
    pub isolated_route_ids: Vec<String>,
}

pub struct ManagedUdpProxy {
    app: UdpProxyApp,
    config: BrokerConfig,
    telemetry: InMemoryTelemetry,
    config_revision: u64,
    ingress_queue_depth: usize,
    safety_policy: ProxyRuntimeSafetyPolicy,
    launch_profile_mode: ProxyLaunchProfileMode,
}

impl ManagedUdpProxy {
    pub async fn start(
        config: BrokerConfig,
        telemetry: InMemoryTelemetry,
        ingress_queue_depth: usize,
        safety_policy: ProxyRuntimeSafetyPolicy,
        launch_profile_mode: ProxyLaunchProfileMode,
        startup_options: ManagedProxyStartupOptions,
    ) -> Result<Self> {
        let prepared = apply_launch_profile(&config, launch_profile_mode);
        let mut app = UdpProxyApp::from_config(&prepared.config, telemetry.clone()).await?;
        app.apply_launch_profile(prepared.status);
        let blockers = safety_policy.blockers(&app.status_snapshot());
        if !blockers.is_empty() {
            anyhow::bail!("udp proxy startup blocked:\n{}", format_blockers(blockers));
        }
        start_ingress_tasks(&mut app, ingress_queue_depth, startup_options).await?;
        emit_config_transition(&telemetry, 1, None, &config);
        app.emit_launch_profile_event(1);

        Ok(Self {
            app,
            config,
            telemetry,
            config_revision: 1,
            ingress_queue_depth,
            safety_policy,
            launch_profile_mode,
        })
    }

    pub fn app(&self) -> &UdpProxyApp {
        &self.app
    }

    pub fn config(&self) -> &BrokerConfig {
        &self.config
    }

    pub fn status_snapshot(&self) -> UdpProxyStatusSnapshot {
        self.app.status_snapshot()
    }

    pub fn operator_report(&self) -> ProxyOperatorReport {
        crate::proxy_operator_report(&self.status_snapshot(), self.safety_policy)
    }

    pub fn operator_overview(&self) -> ProxyOperatorOverview {
        crate::proxy_operator_overview(&self.status_snapshot(), self.safety_policy)
    }

    pub fn operator_diagnostics(&self, history_limit: Option<usize>) -> ProxyOperatorDiagnostics {
        crate::proxy_operator_diagnostics(
            &self.status_snapshot(),
            self.safety_policy,
            history_limit,
        )
    }

    pub fn operator_incidents(&self, history_limit: Option<usize>) -> ProxyOperatorIncidents {
        let status = self.status_snapshot();
        let report = crate::proxy_operator_report(&status, self.safety_policy);
        let (recent_operator_actions, recent_config_events) = status
            .runtime
            .as_ref()
            .map(|runtime| {
                (
                    runtime.recent_operator_actions.clone(),
                    runtime.recent_config_events.clone(),
                )
            })
            .unwrap_or_default();
        crate::proxy_operator_incidents_from_histories(
            &report,
            recent_operator_actions,
            recent_config_events,
            history_limit,
        )
    }

    pub fn operator_snapshot(&self, history_limit: Option<usize>) -> ProxyOperatorSnapshot {
        crate::proxy_operator_snapshot(&self.status_snapshot(), self.safety_policy, history_limit)
    }

    pub fn freeze_traffic(&self) -> bool {
        self.app.freeze_traffic()
    }

    pub fn thaw_traffic(&self) -> bool {
        self.app.thaw_traffic()
    }

    pub fn has_route(&self, route_id: &str) -> bool {
        self.app.has_route(route_id)
    }

    pub fn has_destination(&self, destination_id: &str) -> bool {
        self.app.has_destination(destination_id)
    }

    pub fn isolated_routes(&self) -> Vec<String> {
        self.app.isolated_routes()
    }

    pub fn isolate_route(&self, route_id: &str) -> bool {
        self.app.isolate_route(route_id)
    }

    pub fn restore_route(&self, route_id: &str) -> bool {
        self.app.restore_route(route_id)
    }

    pub fn restore_all_routes(&self) -> usize {
        self.app.restore_all_routes()
    }

    pub async fn rehydrate_destination(&self, destination_id: &str) -> Result<usize> {
        self.app.rehydrate_destination(destination_id).await
    }

    pub async fn replay_route_to_sandbox(
        &self,
        route_id: &str,
        sandbox_destination_id: &str,
        limit: usize,
    ) -> Result<usize> {
        self.app
            .replay_route_to_sandbox(route_id, sandbox_destination_id, limit)
            .await
    }

    pub async fn reload(&mut self, next_config: BrokerConfig) -> Result<()> {
        let next_prepared = apply_launch_profile(&next_config, self.launch_profile_mode);
        let next_status = {
            let mut status = crate::proxy_status_from_config(&next_prepared.config)?;
            status.launch_profile = next_prepared.status.clone();
            status
        };
        let blockers = self.safety_policy.blockers(&next_status);
        if !blockers.is_empty() {
            anyhow::bail!("udp proxy reload blocked:\n{}", format_blockers(blockers));
        }

        let startup_options = if self.app.is_traffic_frozen() {
            ManagedProxyStartupOptions {
                frozen_behavior: FrozenStartupBehavior::Restored,
                isolated_route_ids: self.app.isolated_routes(),
            }
        } else {
            ManagedProxyStartupOptions {
                frozen_behavior: FrozenStartupBehavior::Normal,
                isolated_route_ids: self.app.isolated_routes(),
            }
        };
        let previous_isolated_routes = self.app.isolated_routes();
        let previous_config = self.config.clone();
        self.app.shutdown().await;

        match start_app(
            &next_prepared.config,
            self.telemetry.clone(),
            self.ingress_queue_depth,
            next_prepared.status,
            startup_options.clone(),
        )
        .await
        {
            Ok(app) => {
                let next_revision = self.config_revision + 1;
                let next_isolated_routes = app.isolated_routes();
                for route_id in previous_isolated_routes
                    .iter()
                    .filter(|route_id| !next_isolated_routes.contains(*route_id))
                {
                    self.telemetry
                        .emit(rosc_telemetry::BrokerEvent::RouteIsolationChanged {
                            route_id: route_id.clone(),
                            isolated: false,
                        });
                }
                emit_config_transition(
                    &self.telemetry,
                    next_revision,
                    Some(&self.config),
                    &next_config,
                );
                app.emit_launch_profile_event(next_revision);
                self.app = app;
                self.config = next_config;
                self.config_revision = next_revision;
                Ok(())
            }
            Err(error) => {
                let rollback_prepared =
                    apply_launch_profile(&previous_config, self.launch_profile_mode);
                let rollback = start_app(
                    &rollback_prepared.config,
                    self.telemetry.clone(),
                    self.ingress_queue_depth,
                    rollback_prepared.status,
                    startup_options,
                )
                .await
                .with_context(
                    || "failed to restore the previous proxy configuration after reload failure",
                )?;
                self.app = rollback;
                self.config = previous_config;
                Err(error).context("failed to apply the new proxy configuration")
            }
        }
    }

    pub async fn shutdown(&mut self) {
        self.app.shutdown().await;
    }
}

fn format_blockers(blockers: Vec<String>) -> String {
    blockers
        .into_iter()
        .map(|blocker| format!("- {blocker}"))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn start_app(
    config: &BrokerConfig,
    telemetry: InMemoryTelemetry,
    ingress_queue_depth: usize,
    launch_profile: crate::ProxyLaunchProfileStatus,
    startup_options: ManagedProxyStartupOptions,
) -> Result<UdpProxyApp> {
    let mut app = UdpProxyApp::from_config(config, telemetry).await?;
    app.apply_launch_profile(launch_profile);
    start_ingress_tasks(&mut app, ingress_queue_depth, startup_options).await?;
    Ok(app)
}

async fn start_ingress_tasks(
    app: &mut UdpProxyApp,
    ingress_queue_depth: usize,
    startup_options: ManagedProxyStartupOptions,
) -> Result<()> {
    match startup_options.frozen_behavior {
        FrozenStartupBehavior::Normal => {}
        FrozenStartupBehavior::OperatorRequested => {
            let _ = app.freeze_traffic();
        }
        FrozenStartupBehavior::Restored => {
            let _ = app.restore_frozen_traffic();
        }
    }
    for route_id in &startup_options.isolated_route_ids {
        app.restore_route_isolation(route_id);
    }
    app.spawn_ingress_tasks(ingress_queue_depth).await
}
