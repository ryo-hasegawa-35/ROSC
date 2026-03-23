# Fault Model And Overload Behavior

## Purpose

This document defines how the broker should behave when the environment becomes
unhealthy: malformed traffic, traffic floods, slow consumers, broken adapters,
plugin crashes, memory pressure, and operator mistakes.

The key principle is simple:

- fail locally
- degrade explicitly
- preserve critical control paths first

## Failure Domains

Failures should be contained inside the smallest practical boundary.

### Sender Domain

Examples:

- packet flood
- malformed payloads
- spoofed traffic
- invalid namespace usage

Expected containment:

- sender-level quarantine
- sender-level rate limiting
- sender-level drop counters

### Route Domain

Examples:

- expensive transform
- bad route rule
- pathological fan-out
- route-specific flood

Expected containment:

- route-level queue budget
- route-level circuit breaker
- route-level disable or isolate action

### Destination Domain

Examples:

- slow UDP sink
- disconnected TCP client
- blocked WebSocket peer
- failing MQTT broker

Expected containment:

- per-destination egress queue
- per-destination circuit breaker
- destination-specific shed behavior

### Adapter / Plugin Domain

Examples:

- Wasm timeout
- adapter panic
- IPC plugin disconnect
- untrusted transform consuming too much CPU

Expected containment:

- process or runtime boundary
- timeout and kill policy
- adapter isolation from broker core

### Broker Domain

Examples:

- memory pressure
- allocator churn
- excessive diagnostics overhead
- configuration thrash

Expected containment:

- global degraded mode
- capture reduction
- nonessential feature shedding
- safe mode restart path

## Fault Taxonomy

The broker should classify faults in terms that operators can understand.

Suggested categories:

- `MalformedInput`
- `LegacyOpaqueInput`
- `RateExceeded`
- `QueueOverflow`
- `DestinationStalled`
- `TransformFailed`
- `TransformTimedOut`
- `AdapterUnavailable`
- `SecurityDenied`
- `ConfigInvalid`
- `ClockUncertain`
- `ResourcePressure`

## Overload States

The broker should expose explicit operating states instead of silently
deteriorating.

### `Healthy`

- all queues within target bounds
- no active shedding
- no circuit breaker open

### `Pressured`

- some queues growing
- temporary drops may begin on best-effort routes
- operator warning should be visible

### `Degraded`

- noncritical routes are shedding load
- one or more destinations or transforms are isolated
- core control paths are still protected

### `Emergency`

- broker is protecting itself from collapse
- diagnostics, capture, or optional transforms may be reduced
- only highest-priority classes receive strongest protection

### `SafeMode`

- broker boots or reboots with risky extensions disabled
- focus is on preserving a minimal compatible routing core

## Queueing Principles

- Every queue must be bounded.
- Queue ownership must be explicit.
- One destination must not block unrelated destinations.
- Control traffic and telemetry traffic should not share the same failure budget.

Recommended queue ownership:

- ingress queue per socket or ingress worker
- route queue per route or route group
- egress queue per destination
- plugin inbox per plugin boundary

## Traffic Classes

Not all packets should be treated equally under overload.

Suggested traffic classes:

- `CriticalControl`
- `StatefulControl`
- `SensorStream`
- `Telemetry`
- `ForensicCapture`

Suggested intent:

- `CriticalControl`
  - protect latency
  - prefer freshest packet
  - minimal queue depth
- `StatefulControl`
  - preserve latest correct state
  - allow cache-aware coalescing
- `SensorStream`
  - tolerate sampling or drop-old behavior
  - never starve control routes
- `Telemetry`
  - best effort
  - first to shed under pressure
- `ForensicCapture`
  - must never block primary routing
  - bounded side path only

## Drop And Shed Policies

Drop behavior must be intentional and visible.

Supported policies should include:

- `DropNewest`
- `DropOldest`
- `Sample`
- `CoalesceByAddress`
- `CoalesceByKey`
- `DisableRouteTemporarily`

Recommended defaults:

- `CriticalControl`
  - tiny queue
  - `DropOldest`
- `StatefulControl`
  - coalesce to latest state when safe
- `SensorStream`
  - sample or `DropOldest`
- `Telemetry`
  - `DropNewest` or route disable under sustained overload

## Circuit Breakers

Circuit breakers should exist at least at route and destination boundaries.

Suggested states:

- `Closed`
- `Open`
- `HalfOpen`

Open conditions may include:

- repeated send failures
- repeated transform failures
- sustained timeout
- queue overflow beyond threshold

Half-open recovery:

- probe with limited traffic
- close again only after success window

## Quarantine

Quarantine is stronger than ordinary shedding.

Suggested quarantine triggers:

- malformed packet storm
- repeated security violations
- repeated namespace violations
- sender flood above hard cap

Quarantine actions:

- drop all traffic from source for a cooling-off interval
- flag source in diagnostics
- require operator acknowledgement for repeated offenders if configured

## Transform And Plugin Failure Policy

Transforms and plugins must never be allowed to destabilize the data plane.

Rules:

- every transform has CPU and wall-time budgets
- transform failure is local to that route or plugin
- repeated failures trip a breaker
- optional transform paths may be bypassed under degraded mode
- broker core continues routing unaffected traffic

## Diagnostics Under Pressure

Observability is important, but it must not destroy the hot path.

Policy:

- metrics remain on longest
- capture is reduced before routing is sacrificed
- replay is never allowed to compete with critical live traffic
- dashboard refresh can be sampled or degraded

## Security Under Pressure

Security routes and compatibility routes should fail differently.

Recommended policy:

- secure ingress with failed verification should fail closed
- legacy compatible raw OSC routes should fail according to route drop policy
- security checks must not block unrelated nonsecure routes

## Recovery Interaction

Fault handling and recovery must fit together.

Examples:

- circuit breaker open should not erase cached good state
- replay traffic must be marked so it does not retrigger quarantine wrongly
- warm restart should restore breaker state only if explicitly configured

## Operator Controls

The operator should be able to intervene clearly.

Useful controls:

- isolate route
- drain route
- freeze replay
- disable plugin
- resend cached state
- enter safe mode
- acknowledge recurring fault

## Metrics And Alerts

The broker should surface at least:

- queue depth by domain
- drops by reason
- quarantine events
- breaker open count
- transform timeout count
- adapter disconnect count
- degraded mode transitions
- emergency mode transitions

## Non-Negotiable Invariants

- one slow consumer must not stall unrelated consumers
- malformed or hostile traffic must not crash the broker
- optional features must shed before the core data plane collapses
- every overload action must be measurable
- hidden queue growth is unacceptable

## Follow-On Documents

This model should align directly with:

- internal packet and metadata model
- recovery model and cache semantics
- route configuration grammar
- security overlay model
