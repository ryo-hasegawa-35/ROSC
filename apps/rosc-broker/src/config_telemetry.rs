use rosc_config::{BrokerConfig, ConfigApplyResult};
use rosc_telemetry::{BrokerEvent, TelemetrySink};

pub fn emit_applied_config<TTelemetry>(telemetry: &TTelemetry, applied: &ConfigApplyResult)
where
    TTelemetry: TelemetrySink,
{
    telemetry.emit(BrokerEvent::ConfigApplied {
        revision: applied.revision,
        added_ingresses: applied.diff.added_ingresses.len(),
        removed_ingresses: applied.diff.removed_ingresses.len(),
        changed_ingresses: applied.diff.changed_ingresses.len(),
        added_destinations: applied.diff.added_destinations.len(),
        removed_destinations: applied.diff.removed_destinations.len(),
        changed_destinations: applied.diff.changed_destinations.len(),
        added_routes: applied.diff.added_routes.len(),
        removed_routes: applied.diff.removed_routes.len(),
        changed_routes: applied.diff.changed_routes.len(),
    });
}

pub fn emit_initial_config_applied<TTelemetry>(telemetry: &TTelemetry, config: &BrokerConfig)
where
    TTelemetry: TelemetrySink,
{
    telemetry.emit(BrokerEvent::ConfigApplied {
        revision: 1,
        added_ingresses: config.udp_ingresses.len(),
        removed_ingresses: 0,
        changed_ingresses: 0,
        added_destinations: config.udp_destinations.len(),
        removed_destinations: 0,
        changed_destinations: 0,
        added_routes: config.routes.len(),
        removed_routes: 0,
        changed_routes: 0,
    });
}

#[cfg(test)]
mod tests {
    use super::{emit_applied_config, emit_initial_config_applied};
    use rosc_config::ConfigManager;
    use rosc_telemetry::InMemoryTelemetry;

    #[test]
    fn initial_config_apply_sets_revision_one_and_added_totals() {
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

            [[routes.destinations]]
            target = "udp_renderer"
            transport = "osc_udp"
            "#,
        )
        .unwrap();

        let telemetry = InMemoryTelemetry::default();
        emit_initial_config_applied(&telemetry, &config);
        let snapshot = telemetry.snapshot();

        assert_eq!(snapshot.config_revision, 1);
        assert_eq!(snapshot.config_added_ingresses_total, 1);
        assert_eq!(snapshot.config_added_destinations_total, 1);
        assert_eq!(snapshot.config_added_routes_total, 1);
        assert_eq!(snapshot.config_rejections_total, 0);
    }

    #[test]
    fn applied_config_uses_manager_revision_and_diff_counts() {
        let telemetry = InMemoryTelemetry::default();
        let mut manager = ConfigManager::default();
        let applied = manager
            .apply_toml_str(
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

                [[routes.destinations]]
                target = "udp_renderer"
                transport = "osc_udp"
                "#,
            )
            .unwrap();

        emit_applied_config(&telemetry, &applied);
        let snapshot = telemetry.snapshot();

        assert_eq!(snapshot.config_revision, applied.revision);
        assert_eq!(snapshot.config_added_ingresses_total, 1);
        assert_eq!(snapshot.config_added_destinations_total, 1);
        assert_eq!(snapshot.config_added_routes_total, 1);
    }
}
