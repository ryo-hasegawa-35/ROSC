use rosc_config::{BrokerConfig, ConfigApplyResult};
use rosc_telemetry::{BrokerEvent, TelemetrySink};
use std::collections::BTreeMap;

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
    emit_config_transition(telemetry, 1, None, config);
}

pub fn emit_config_transition<TTelemetry>(
    telemetry: &TTelemetry,
    revision: u64,
    previous: Option<&BrokerConfig>,
    current: &BrokerConfig,
) where
    TTelemetry: TelemetrySink,
{
    let (ingress_counts, destination_counts, route_counts) = match previous {
        Some(previous) => (
            diff_named_counts(
                previous
                    .udp_ingresses
                    .iter()
                    .map(|ingress| (ingress.id.as_str(), ingress)),
                current
                    .udp_ingresses
                    .iter()
                    .map(|ingress| (ingress.id.as_str(), ingress)),
            ),
            diff_named_counts(
                previous
                    .udp_destinations
                    .iter()
                    .map(|destination| (destination.id.as_str(), destination)),
                current
                    .udp_destinations
                    .iter()
                    .map(|destination| (destination.id.as_str(), destination)),
            ),
            diff_named_counts(
                previous
                    .routes
                    .iter()
                    .map(|route| (route.id.as_str(), route)),
                current
                    .routes
                    .iter()
                    .map(|route| (route.id.as_str(), route)),
            ),
        ),
        None => (
            (current.udp_ingresses.len(), 0, 0),
            (current.udp_destinations.len(), 0, 0),
            (current.routes.len(), 0, 0),
        ),
    };

    telemetry.emit(BrokerEvent::ConfigApplied {
        revision,
        added_ingresses: ingress_counts.0,
        removed_ingresses: ingress_counts.1,
        changed_ingresses: ingress_counts.2,
        added_destinations: destination_counts.0,
        removed_destinations: destination_counts.1,
        changed_destinations: destination_counts.2,
        added_routes: route_counts.0,
        removed_routes: route_counts.1,
        changed_routes: route_counts.2,
    });
}

fn diff_named_counts<'a, T: Eq + 'a>(
    previous: impl IntoIterator<Item = (&'a str, &'a T)>,
    current: impl IntoIterator<Item = (&'a str, &'a T)>,
) -> (usize, usize, usize) {
    let previous: BTreeMap<&str, &T> = previous.into_iter().collect();
    let current: BTreeMap<&str, &T> = current.into_iter().collect();

    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;

    for id in current.keys() {
        match previous.get(id) {
            None => added += 1,
            Some(previous_value) if *previous_value != current[id] => changed += 1,
            Some(_) => {}
        }
    }

    for id in previous.keys() {
        if !current.contains_key(id) {
            removed += 1;
        }
    }

    (added, removed, changed)
}

#[cfg(test)]
mod tests {
    use super::{emit_applied_config, emit_config_transition, emit_initial_config_applied};
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

    #[test]
    fn config_transition_uses_revision_and_change_counts() {
        let before = rosc_config::BrokerConfig::from_toml_str(
            r#"
            [[udp_ingresses]]
            id = "in_a"
            bind = "127.0.0.1:9000"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "out_a"
            bind = "127.0.0.1:0"
            target = "127.0.0.1:9001"

            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["in_a"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [[routes.destinations]]
            target = "out_a"
            transport = "osc_udp"
            "#,
        )
        .unwrap();
        let after = rosc_config::BrokerConfig::from_toml_str(
            r#"
            [[udp_ingresses]]
            id = "in_b"
            bind = "127.0.0.1:9002"
            mode = "osc1_0_strict"

            [[udp_destinations]]
            id = "out_a"
            bind = "127.0.0.1:0"
            target = "127.0.0.1:9002"

            [[routes]]
            id = "camera"
            enabled = true
            mode = "osc1_0_strict"
            class = "StatefulControl"

            [routes.match]
            ingress_ids = ["in_b"]
            address_patterns = ["/ue5/camera/fov"]
            protocols = ["osc_udp"]

            [[routes.destinations]]
            target = "out_a"
            transport = "osc_udp"
            "#,
        )
        .unwrap();

        let telemetry = InMemoryTelemetry::default();
        emit_config_transition(&telemetry, 2, Some(&before), &after);
        let snapshot = telemetry.snapshot();

        assert_eq!(snapshot.config_revision, 2);
        assert_eq!(snapshot.config_added_ingresses_total, 1);
        assert_eq!(snapshot.config_removed_ingresses_total, 1);
        assert_eq!(snapshot.config_changed_destinations_total, 1);
        assert_eq!(snapshot.config_changed_routes_total, 1);
    }
}
