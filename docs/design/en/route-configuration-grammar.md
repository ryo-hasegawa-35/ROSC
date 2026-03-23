# Route Configuration Grammar

## Purpose

This document defines the semantic grammar for route configuration.

The goal is not to lock one final file format forever. The goal is to define
the fields, phases, and behaviors that every configuration format must express.

## Design Goals

- human-readable
- hot-reloadable
- validation-friendly
- explicit about failure behavior
- explicit about cache and recovery behavior
- explicit about security scope

## Recommended First External Format

TOML is a good initial choice because it maps cleanly to Rust structures and is
readable for operators. YAML could be supported later, but the semantic model
should not depend on YAML-specific behavior.

## Route Lifecycle

Each route should define:

1. what traffic it matches
2. how that traffic is classified
3. what transforms are applied
4. where it is sent
5. how it behaves under pressure
6. what state is cached or recovered
7. what security and observability rules apply

## Required Route Fields

- `id`
- `enabled`
- `mode`
- `match`
- `class`
- `destinations`

### `id`

- stable identifier
- unique within configuration
- used in logs, metrics, replay, and operator actions

### `enabled`

- boolean
- disabled routes remain defined but do not process live traffic

### `mode`

Allowed values:

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`

### `match`

Defines the ingress selection criteria.

Possible subfields:

- `ingress_ids`
- `source_endpoints`
- `address_patterns`
- `security_scopes`
- `protocols`

### `class`

Traffic class:

- `CriticalControl`
- `StatefulControl`
- `SensorStream`
- `Telemetry`
- `ForensicCapture`

### `destinations`

One or more output definitions.

Each destination should include:

- `target`
- `transport`
- optional `encoding`
- optional `enabled`

## Optional Route Sections

- `transform`
- `queue`
- `fault`
- `cache`
- `recovery`
- `security`
- `observability`

## Match Grammar

### Address Matching

`address_patterns` should allow one or more patterns.

Examples:

- `"/ue5/camera/*"`
- `"/td/tracking/*"`
- `"//spherical"` only when route mode allows extended semantics

### Source Matching

Possible selectors:

- exact endpoint
- ingress binding ID
- authenticated source identity
- project scope

### Protocol Matching

Possible selectors:

- `osc_udp`
- `osc_tcp`
- `osc_slip`
- `ws_json`
- `mqtt`
- `ipc`

## Transform Section

Purpose:

- rename address
- static map
- scale values
- call Wasm transform
- invoke external adapter pipeline

Possible fields:

- `rename_address`
- `map_arguments`
- `drop_arguments`
- `prepend_arguments`
- `wasm_filter`
- `external_plugin`

Rules:

- transforms only run on safely transformable packets
- failed transforms follow the route fault policy
- transforms produce derived packets, never mutate the original packet in place

## Queue Section

Possible fields:

- `max_depth`
- `coalesce`
- `coalesce_key`
- `priority`
- `deadline_ms`
- `batching`

`coalesce` values may include:

- `none`
- `by_address`
- `by_key`

Rules:

- if `coalesce = "by_key"`, `coalesce_key` must also be defined
- `coalesce_key` should reference a stable normalized field or adapter-provided
  key, not an arbitrary raw payload fragment

## Fault Section

Possible fields:

- `drop_policy`
- `rate_limit_per_sec`
- `burst_limit`
- `breaker_threshold`
- `quarantine_on_malformed`
- `timeout_ms`

`drop_policy` values may include:

- `drop_newest`
- `drop_oldest`
- `sample`
- `disable_temporarily`

## Cache Section

Possible fields:

- `policy`
- `key`
- `ttl_ms`
- `persist`

`policy` values:

- `none`
- `last_value_per_address`
- `last_value_per_key`
- `snapshot_set`
- `journal_window`
- `durable_journal`

`persist` values:

- `ephemeral`
- `warm`
- `durable`

## Recovery Section

Possible fields:

- `late_joiner`
- `rehydrate_on_connect`
- `rehydrate_on_restart`
- `manual_only`
- `replay_allowed`

Example values:

- `late_joiner = "latest"`
- `rehydrate_on_connect = true`
- `replay_allowed = false`

## Security Section

Possible fields:

- `scope`
- `require_verified_source`
- `allowed_identities`
- `allow_legacy_bridge`

Rules:

- security config should never require changing raw OSC payloads for legacy
  downstream peers
- route authorization happens before transform and egress

## Observability Section

Possible fields:

- `metrics_level`
- `capture`
- `capture_trigger`
- `log_payload`
- `emit_correlation_id`

`metrics_level` values may include:

- `off`
- `minimal`
- `standard`
- `detailed`
- `forensic`

`capture` values may include:

- `off`
- `on_error`
- `on_breaker`
- `always_bounded`

Rules:

- forensic capture must remain bounded
- diagnostics settings must not remove queue isolation

## Example TOML Route

```toml
[[routes]]
id = "ue5_camera_fov"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_localhost_in"]
address_patterns = ["/ue5/camera/fov"]
protocols = ["osc_udp"]

[routes.transform]
rename_address = "/render/camera/fov"

[routes.queue]
max_depth = 8
coalesce = "by_address"
priority = 80

[routes.fault]
drop_policy = "drop_oldest"
breaker_threshold = 20
timeout_ms = 10

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 10000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
replay_allowed = false

[routes.security]
scope = "project_a"
require_verified_source = false
allow_legacy_bridge = true

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_renderer"
transport = "osc_udp"

[[routes.destinations]]
target = "dashboard_tap"
transport = "internal"
```

## Validation Rules

- route IDs must be unique
- unsupported mode and pattern combinations must fail validation
- cache and recovery policy combinations must be validated together
- dangerous defaults must fail closed where appropriate
- destination references must resolve

## Hot Reload Rules

- config update should be validated before apply
- failed apply should not destroy the last good config
- route changes should appear as diffable operator events
- route disable should be safer than delete for emergency response

## Non-Negotiable Invariants

- configuration must express compatibility mode explicitly
- fault policy must be explicit for live routes
- cache and recovery policy must never be inferred from traffic accidentally
- security scope must be attachable without changing raw OSC payload semantics
- config must be machine-validated before apply
