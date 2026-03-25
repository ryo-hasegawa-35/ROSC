use rosc_config::BrokerConfig;
use rosc_route::CapturePolicy;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyLaunchProfileMode {
    #[default]
    Normal,
    SafeMode,
}

impl ProxyLaunchProfileMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::SafeMode => "safe_mode",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProxyLaunchProfileStatus {
    pub mode: ProxyLaunchProfileMode,
    pub disabled_capture_routes: Vec<String>,
    pub disabled_replay_routes: Vec<String>,
    pub disabled_restart_rehydrate_routes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedLaunchConfig {
    pub config: BrokerConfig,
    pub status: ProxyLaunchProfileStatus,
}

pub fn apply_launch_profile(
    config: &BrokerConfig,
    mode: ProxyLaunchProfileMode,
) -> PreparedLaunchConfig {
    match mode {
        ProxyLaunchProfileMode::Normal => PreparedLaunchConfig {
            config: config.clone(),
            status: ProxyLaunchProfileStatus {
                mode,
                ..ProxyLaunchProfileStatus::default()
            },
        },
        ProxyLaunchProfileMode::SafeMode => {
            let mut config = config.clone();
            let mut status = ProxyLaunchProfileStatus {
                mode,
                ..ProxyLaunchProfileStatus::default()
            };

            for route in &mut config.routes {
                if route.observability.capture != CapturePolicy::Off {
                    status.disabled_capture_routes.push(route.id.clone());
                    route.observability.capture = CapturePolicy::Off;
                }
                if route.recovery.replay_allowed {
                    status.disabled_replay_routes.push(route.id.clone());
                    route.recovery.replay_allowed = false;
                }
                if route.recovery.rehydrate_on_restart {
                    status
                        .disabled_restart_rehydrate_routes
                        .push(route.id.clone());
                    route.recovery.rehydrate_on_restart = false;
                }
            }

            PreparedLaunchConfig { config, status }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProxyLaunchProfileMode, apply_launch_profile};

    #[test]
    fn safe_mode_disables_optional_recovery_and_capture_surfaces() {
        let config = rosc_config::BrokerConfig::from_toml_str(
            r#"
            [[udp_ingresses]]
            id = "udp_localhost_in"
            bind = "127.0.0.1:9000"
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

            [routes.recovery]
            replay_allowed = true
            rehydrate_on_restart = true

            [routes.cache]
            policy = "last_value_per_address"

            [routes.observability]
            capture = "always_bounded"

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#,
        )
        .unwrap();

        let prepared = apply_launch_profile(&config, ProxyLaunchProfileMode::SafeMode);

        assert_eq!(prepared.status.mode, ProxyLaunchProfileMode::SafeMode);
        assert_eq!(prepared.status.disabled_capture_routes, vec!["camera"]);
        assert_eq!(prepared.status.disabled_replay_routes, vec!["camera"]);
        assert_eq!(
            prepared.status.disabled_restart_rehydrate_routes,
            vec!["camera"]
        );
        assert_eq!(
            prepared.config.routes[0].observability.capture,
            rosc_route::CapturePolicy::Off
        );
        assert!(!prepared.config.routes[0].recovery.replay_allowed);
        assert!(!prepared.config.routes[0].recovery.rehydrate_on_restart);
        assert!(
            prepared.config.routes[0].recovery.rehydrate_on_connect
                == config.routes[0].recovery.rehydrate_on_connect
        );
    }
}
