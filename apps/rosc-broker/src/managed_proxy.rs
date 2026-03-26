use anyhow::{Context, Result};
use rosc_config::BrokerConfig;
use rosc_telemetry::InMemoryTelemetry;

use crate::{
    ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy, UdpProxyApp, apply_launch_profile,
    emit_config_transition,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FrozenStartupBehavior {
    #[default]
    Normal,
    OperatorRequested,
    Restored,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ManagedProxyStartupOptions {
    pub frozen_behavior: FrozenStartupBehavior,
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
        app.set_launch_profile(prepared.status);
        let blockers = safety_policy.blockers(&app.status_snapshot());
        if !blockers.is_empty() {
            anyhow::bail!("udp proxy startup blocked:\n{}", format_blockers(blockers));
        }
        start_ingress_tasks(&mut app, ingress_queue_depth, startup_options).await?;
        emit_config_transition(&telemetry, 1, None, &config);

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

    pub fn freeze_traffic(&self) {
        self.app.freeze_traffic();
    }

    pub fn thaw_traffic(&self) {
        self.app.thaw_traffic();
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
            }
        } else {
            ManagedProxyStartupOptions::default()
        };
        let previous_config = self.config.clone();
        self.app.shutdown().await;

        match start_app(
            &next_prepared.config,
            self.telemetry.clone(),
            self.ingress_queue_depth,
            next_prepared.status,
            startup_options,
        )
        .await
        {
            Ok(app) => {
                let next_revision = self.config_revision + 1;
                emit_config_transition(
                    &self.telemetry,
                    next_revision,
                    Some(&self.config),
                    &next_config,
                );
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
    app.set_launch_profile(launch_profile);
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
        FrozenStartupBehavior::OperatorRequested => app.freeze_traffic(),
        FrozenStartupBehavior::Restored => app.restore_frozen_traffic(),
    }
    app.spawn_ingress_tasks(ingress_queue_depth).await
}
