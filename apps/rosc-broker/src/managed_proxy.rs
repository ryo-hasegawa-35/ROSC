use anyhow::{Context, Result};
use rosc_config::BrokerConfig;
use rosc_telemetry::InMemoryTelemetry;

use crate::{ProxyLaunchProfileMode, ProxyRuntimeSafetyPolicy, UdpProxyApp, apply_launch_profile};

pub struct ManagedUdpProxy {
    app: UdpProxyApp,
    config: BrokerConfig,
    telemetry: InMemoryTelemetry,
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
    ) -> Result<Self> {
        let prepared = apply_launch_profile(&config, launch_profile_mode);
        let mut app = UdpProxyApp::from_config(&prepared.config, telemetry.clone()).await?;
        app.set_launch_profile(prepared.status);
        let blockers = safety_policy.blockers(&app.status_snapshot());
        if !blockers.is_empty() {
            anyhow::bail!("udp proxy startup blocked:\n{}", format_blockers(blockers));
        }
        app.spawn_ingress_tasks(ingress_queue_depth).await?;

        Ok(Self {
            app,
            config,
            telemetry,
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

        let previous_config = self.config.clone();
        self.app.shutdown().await;

        match start_app(
            &next_prepared.config,
            self.telemetry.clone(),
            self.ingress_queue_depth,
            next_prepared.status,
        )
        .await
        {
            Ok(app) => {
                self.app = app;
                self.config = next_config;
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
) -> Result<UdpProxyApp> {
    let mut app = UdpProxyApp::from_config(config, telemetry).await?;
    app.set_launch_profile(launch_profile);
    app.spawn_ingress_tasks(ingress_queue_depth).await?;
    Ok(app)
}
