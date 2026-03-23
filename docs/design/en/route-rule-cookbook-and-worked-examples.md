# Route Rule Cookbook And Worked Examples

## Purpose

This document turns the route grammar into concrete operator-facing examples.

The grammar document defines what configuration must be able to express. This
document shows how to combine those fields for real deployment patterns without
accidentally weakening compatibility, isolation, or recovery safety.

## Reading Note

- This document is illustrative, not a replacement for the normative grammar.
- If an example conflicts with the route grammar, the grammar wins.
- Examples prefer clarity over maximum compactness.

Related documents:

- [Route Configuration Grammar](./route-configuration-grammar.md)
- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
- [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)

## Example 1: Transparent Localhost Proxy

Use when:

- a creative tool already emits valid OSC 1.0
- the safest first adoption path is localhost insertion
- no schema or transform logic is needed yet

Why this shape is good:

- strict compatibility mode stays visible
- queue depth stays intentionally small
- the route remains easy to reason about during first deployment

```toml
[[routes]]
id = "ue5_local_proxy"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_ue5_local"]
address_patterns = ["/ue5/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 16
coalesce = "none"
priority = 100

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 8
timeout_ms = 5

[routes.cache]
policy = "none"
persist = "ephemeral"

[routes.recovery]
late_joiner = "none"
rehydrate_on_connect = false
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_show_router"
transport = "osc_udp"
```

Operator notes:

- For first rollout, prefer no transform and no cache.
- If the route later gains transforms, consider splitting it so the transparent
  proxy path remains independently observable.

## Example 2: Fan-Out With Independent Failure Domains

Use when:

- one ingress namespace feeds both a renderer and an operator-facing tap
- the destinations have different latency tolerance
- observability should never slow the primary renderer path

Design rule:

- if destinations need different queue, timeout, or breaker policy, split them
  into separate routes rather than one route with many destinations

```toml
[[routes]]
id = "camera_fov_to_renderer"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_local_proxy"]
address_patterns = ["/render/camera/fov"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 8
coalesce = "by_address"
priority = 90

[routes.fault]
drop_policy = "drop_oldest"
breaker_threshold = 10
timeout_ms = 5

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 15000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_renderer_a"
transport = "osc_udp"

[[routes]]
id = "camera_fov_to_dashboard"
enabled = true
mode = "osc1_0_strict"
class = "Telemetry"

[routes.match]
ingress_ids = ["udp_local_proxy"]
address_patterns = ["/render/camera/fov"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 128
coalesce = "by_address"
priority = 20

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 120
burst_limit = 240
breaker_threshold = 40
timeout_ms = 25

[routes.observability]
metrics_level = "minimal"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "dashboard_tap"
transport = "internal"
```

Operator notes:

- The renderer path keeps tight latency and low depth.
- The dashboard path is explicitly disposable under pressure.

## Example 3: Legacy Tolerant Ingress Bridge

Use when:

- an older sender omits the OSC type tag string
- the bridge should accept legacy input without making tolerant behavior global
- downstream systems should still see normalized routing metadata

```toml
[[routes]]
id = "legacy_tracking_bridge"
enabled = true
mode = "osc1_0_legacy_tolerant"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_legacy_tracker"]
address_patterns = ["/tracking/*"]
protocols = ["osc_udp"]

[routes.transform]
rename_address = "/sensors/tracking"

[routes.queue]
max_depth = 256
coalesce = "by_key"
coalesce_key = "source_entity_id"
priority = 60

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 5000
burst_limit = 8000
quarantine_on_malformed = true
timeout_ms = 10

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_td_tracker_bus"
transport = "osc_udp"
```

Operator notes:

- Keep tolerant behavior close to the edge.
- Do not let tolerant ingress silently redefine strict internal compatibility
  claims.

## Example 4: Late Joiner State Cache

Use when:

- a restarted consumer must quickly recover scene state
- the namespace represents state, not transient triggers
- last value behavior is sufficient

```toml
[[routes]]
id = "scene_state_cache"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_scene_control"]
address_patterns = ["/scene/state/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 32
coalesce = "by_address"
priority = 85

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 600000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
rehydrate_on_restart = true
manual_only = false
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_projection_nodes"
transport = "osc_udp"
```

Operator notes:

- Trigger namespaces should usually be routed separately with `manual_only = true`
  or no automatic rehydrate at all.
- Keep cache TTL tied to operational truth, not convenience alone.

## Example 5: High-Rate Sensor Stream With Bounded Loss

Use when:

- one namespace carries high-volume positional or depth-like data
- bounded loss is acceptable
- critical control traffic must remain protected elsewhere

```toml
[[routes]]
id = "depth_stream_fanout"
enabled = true
mode = "osc1_0_strict"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_depth_sensor"]
address_patterns = ["/sensor/depth/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 2048
coalesce = "by_key"
coalesce_key = "sensor_stream_id"
priority = 40
batching = 8

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 60000
burst_limit = 90000
breaker_threshold = 100
timeout_ms = 8

[routes.cache]
policy = "journal_window"
ttl_ms = 2500
persist = "ephemeral"

[routes.recovery]
late_joiner = "none"
rehydrate_on_connect = false
replay_allowed = true

[routes.observability]
metrics_level = "minimal"
capture = "off"
emit_correlation_id = false

[[routes.destinations]]
target = "udp_td_depth_consumer"
transport = "osc_udp"

[[routes.destinations]]
target = "ws_depth_monitor"
transport = "ws_json"
encoding = "json"
```

Operator notes:

- This route intentionally values continuity over perfect retention.
- Never mix trigger traffic into the same route just because the sender is the
  same process.

## Example 6: Slow Destination Quarantine

Use when:

- one archival or diagnostic consumer can become slow
- healthy real-time consumers must remain unaffected
- breaker and quarantine behavior must be operator-visible

```toml
[[routes]]
id = "critical_to_renderer"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_show_control"]
address_patterns = ["/show/critical/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 8
coalesce = "none"
priority = 100

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 6
timeout_ms = 3

[[routes.destinations]]
target = "udp_renderer_primary"
transport = "osc_udp"

[[routes]]
id = "critical_to_archive"
enabled = true
mode = "osc1_0_strict"
class = "ForensicCapture"

[routes.match]
ingress_ids = ["udp_show_control"]
address_patterns = ["/show/critical/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 512
coalesce = "none"
priority = 10

[routes.fault]
drop_policy = "disable_temporarily"
breaker_threshold = 20
timeout_ms = 50

[routes.observability]
metrics_level = "detailed"
capture = "on_breaker"
capture_trigger = "breaker_open"
emit_correlation_id = true

[[routes.destinations]]
target = "file_capture_sink"
transport = "internal"
```

Operator notes:

- Quarantine is easier to trust when the archival path is already isolated.
- Breaker events on this route should never imply renderer instability.

## Example 7: Error-Triggered Forensic Capture

Use when:

- continuous full capture would be too expensive
- incidents are rare but require deep evidence
- operators need bounded capture windows on failure

```toml
[[routes]]
id = "tracking_forensic_window"
enabled = true
mode = "osc1_0_strict"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_tracking_bus"]
address_patterns = ["/tracking/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 512
coalesce = "by_key"
coalesce_key = "tracking_subject_id"
priority = 50

[routes.fault]
drop_policy = "sample"
breaker_threshold = 50
quarantine_on_malformed = true
timeout_ms = 12

[routes.cache]
policy = "journal_window"
ttl_ms = 10000
persist = "warm"

[routes.recovery]
manual_only = true
replay_allowed = true

[routes.observability]
metrics_level = "detailed"
capture = "on_error"
capture_trigger = "malformed_or_breaker"
log_payload = false
emit_correlation_id = true

[[routes.destinations]]
target = "udp_td_tracking"
transport = "osc_udp"
```

Operator notes:

- Prefer bounded capture plus strong metadata over always-on payload logging.
- Replay remains manual because sensor traffic can be unsafe to re-inject
  automatically into a live show.

## Example 8: Optional Secure Overlay Boundary

Use when:

- a shared network requires source verification at the broker edge
- downstream OSC peers must remain unchanged
- authenticated and legacy traffic need different treatment

```toml
[[routes]]
id = "verified_control_ingress"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_shared_network_in"]
address_patterns = ["/project_a/control/*"]
security_scopes = ["project_a"]
protocols = ["osc_udp"]

[routes.security]
scope = "project_a"
require_verified_source = true
allowed_identities = ["ops_tablet_a", "show_controller_a"]
allow_legacy_bridge = false

[routes.queue]
max_depth = 16
coalesce = "none"
priority = 95

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 8
timeout_ms = 5

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_internal_show_control"
transport = "osc_udp"
```

Operator notes:

- Verification happens at the broker boundary, not by mutating raw OSC payloads.
- Legacy routes may still exist, but they should be explicit and separately
  monitored.

## Common Mistakes To Avoid

- putting unrelated traffic classes into one route because they share an ingress
- sharing one route across destinations with different fault budgets
- enabling detailed or forensic metrics on every hot path by default
- treating state cache and replay as the same safety mechanism
- making secure overlay policy mandatory for all raw OSC peers

## Design Rules That Recur Across Examples

- Split routes when latency, durability, or trust posture diverge.
- Keep tolerant compatibility modes near the edge.
- Make disposable observability paths explicitly lower priority.
- Reserve automatic recovery for state, not triggers.
- Keep every risky behavior visible in metrics and operator tooling.
