# Metrics And Telemetry Schema

## Purpose

This document defines the canonical telemetry vocabulary for the broker.

The goal is to make dashboards, logs, alerts, benchmarks, and future exporters
describe the same reality. Export format can change later. Semantics should not
drift casually.

Related documents:

- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Dashboard Information Architecture](./dashboard-information-architecture.md)
- [Benchmark Workload Definition](./benchmark-workload-definition.md)
- [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)

## Telemetry Design Principles

- Telemetry exists to support action, not vanity graphs.
- Critical control and optional telemetry do not share the same failure budget.
- Cardinality must remain bounded by design.
- Route, destination, and broker identity must be traceable without logging raw
  payloads by default.
- Export adapters must preserve canonical meaning even if external systems use
  different naming conventions.

## Canonical Telemetry Layers

- counters
- gauges
- histograms
- structured events
- structured logs
- optional traces for long-latency workflows

The broker should treat metrics and events as primary. Traces are additive.

## Telemetry Levels

`metrics_level` should support these semantic levels:

- `off`
  - only emergency process health
- `minimal`
  - route and destination health with tight cardinality
- `standard`
  - default operational view
- `detailed`
  - richer diagnosis for selected routes or incidents
- `forensic`
  - bounded, incident-oriented deep evidence

Rules:

- `standard` is the recommended default for most production routes.
- `detailed` and `forensic` should be opt-in and bounded.
- `forensic` should usually require profile support or explicit incident mode.

## Naming Convention

Metric names should follow this pattern:

- `rosc_<domain>_<subject>_<measure>`

Examples:

- `rosc_ingress_packets_total`
- `rosc_route_latency_seconds`
- `rosc_destination_queue_depth`
- `rosc_cache_entries`
- `rosc_security_rejections_total`

Rules:

- counters should end in `_total`
- gauges should read like current state or quantity
- latency and duration histograms should use `_seconds`
- bytes should use `_bytes`

## Required Dimensions

The canonical schema may attach these dimensions where meaningful:

- `broker_id`
- `profile`
- `compat_mode`
- `traffic_class`
- `transport`
- `ingress_id`
- `route_id`
- `destination_id`
- `adapter_id`
- `plugin_id`
- `reason`
- `scope`

## Cardinality Rules

- `route_id` and `destination_id` should be stable identifiers from
  configuration, not user-entered free text.
- Raw OSC addresses should not become metric labels by default.
- Source IP labels should be avoided in steady-state metrics unless intentionally
  bucketed or sampled.
- Correlation IDs belong in events and logs, not in metric labels.
- Per-packet labels are forbidden in canonical metrics.

## Core Metric Families

### Process And Runtime

- `rosc_process_cpu_usage_ratio`
- `rosc_process_memory_bytes`
- `rosc_runtime_task_count`
- `rosc_runtime_fd_count`
- `rosc_runtime_uptime_seconds`

### Ingress

- `rosc_ingress_packets_total`
- `rosc_ingress_bytes_total`
- `rosc_ingress_parse_failures_total`
- `rosc_ingress_rejected_total`
- `rosc_ingress_queue_depth`

### Route

- `rosc_route_matches_total`
- `rosc_route_transform_failures_total`
- `rosc_route_latency_seconds`
- `rosc_route_drops_total`
- `rosc_route_disabled_total`

### Destination / Egress

- `rosc_destination_send_total`
- `rosc_destination_send_failures_total`
- `rosc_destination_queue_depth`
- `rosc_destination_latency_seconds`
- `rosc_destination_breaker_open_total`
- `rosc_destination_quarantine_total`

### Cache And Recovery

- `rosc_cache_entries`
- `rosc_cache_writes_total`
- `rosc_cache_evictions_total`
- `rosc_recovery_rehydrate_total`
- `rosc_recovery_rehydrate_latency_seconds`
- `rosc_recovery_replay_total`

### Security And Identity

- `rosc_security_rejections_total`
- `rosc_security_verified_sources`
- `rosc_security_legacy_bridge_total`
- `rosc_security_scope_mismatch_total`

### Discovery

- `rosc_discovery_services_visible`
- `rosc_discovery_stale_services_total`
- `rosc_discovery_refresh_total`

### Plugin And Adapter

- `rosc_plugin_invocations_total`
- `rosc_plugin_failures_total`
- `rosc_plugin_timeout_total`
- `rosc_adapter_reconnect_total`
- `rosc_adapter_backpressure_total`

### Federation And HA

- `rosc_cluster_replication_lag_seconds`
- `rosc_cluster_failover_events_total`
- `rosc_cluster_peer_disconnect_total`

## Structured Event Schema

Every high-value event should be able to carry:

- `event_id`
- `timestamp`
- `severity`
- `component`
- `event_type`
- `broker_id`
- `compat_mode`
- `traffic_class`
- `ingress_id`
- `route_id`
- `destination_id`
- `reason_code`
- `correlation_id`
- `operator_action_required`

Optional fields:

- `plugin_id`
- `scope`
- `capture_window_id`
- `config_revision`
- `peer_broker_id`

## Log Schema Expectations

Logs should be structured and machine-parseable.

Required fields:

- `timestamp`
- `level`
- `component`
- `message`
- `broker_id`

Recommended fields when relevant:

- `route_id`
- `destination_id`
- `reason_code`
- `correlation_id`
- `config_revision`

Rules:

- raw payload logging should be off by default
- payload excerpts, if allowed, should be bounded and redaction-aware

## Correlation Identifier Rules

- Correlation IDs should be generated at ingress or preserved from a trusted
  upstream context.
- The same correlation ID may appear in metrics-adjacent events, logs, capture
  metadata, and replay records.
- Correlation IDs should not be required for raw OSC compatibility.

## Alerting Guidance

Useful alert families include:

- sustained parse failures above normal baseline
- destination breaker repeatedly opening
- critical route drop count above zero
- replication lag above declared tolerance
- cache rehydrate failure on a stateful route

Alert rules should be profile-aware and traffic-class-aware.

## Export Model

The internal telemetry schema is canonical. Exporters may map it to:

- Prometheus-style metrics
- OpenTelemetry metrics, logs, or traces
- local dashboard streams
- forensic capture metadata

Exporter choice must not redefine the canonical field meaning.

## Retention Guidance

- minimal operational metrics should survive ordinary restarts where practical
- detailed and forensic data should be bounded by explicit retention policy
- capture metadata should outlive the incident long enough for investigation

## Non-Negotiable Invariants

- Telemetry must help explain safety behavior, not compete with it.
- Cardinality explosions are design bugs.
- Critical route health must remain visible even when optional telemetry is
  degraded.
- Canonical names and meanings must stay stable enough for benchmarks and
  operators to compare revisions honestly.
