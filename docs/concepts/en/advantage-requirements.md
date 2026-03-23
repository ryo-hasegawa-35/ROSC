# Advantage Requirements

## Objective

Before narrowing the implementation plan, we should define what would make this
system overwhelmingly better than ordinary OSC usage and ordinary OSC routers.

The goal is not to replace raw OSC with an incompatible protocol. The goal is
to preserve raw OSC compatibility while building a much stronger runtime,
operations layer, and safety model around it.

## Core Hypothesis

Plain OSC is powerful because it is simple, flexible, and widely supported.
Plain OSC is weak because it leaves too much undefined:

- transport behavior under stress
- congestion and backpressure policy
- fault isolation
- recovery after restart
- observability
- security
- schema and validation
- interoperability metadata

An overwhelming successor system should keep the flexibility of OSC while
solving these operational gaps.

## What "Overwhelmingly Better" Means

The system should dominate ordinary OSC setups across these axes:

1. Much lower added jitter under load
2. Much better fault containment when one consumer is slow or broken
3. Much safer operation on messy shared networks
4. Much faster recovery after restart or partial failure
5. Much stronger visibility into what happened and why
6. Much easier migration from existing UE5 / TouchDesigner / tool workflows
7. Much richer extensibility without destabilizing the core

## Pillar 1: Deterministic Performance

The system should not merely be fast in throughput benchmarks. It should remain
predictable under ugly real-world traffic.

Desired capabilities:

- preallocated buffers and bounded memory growth
- route-level and destination-level queue isolation
- lock-minimized hot path
- zero-copy or near-zero-copy packet handling where practical
- bounded-latency serialization and dispatch
- explicit overload modes instead of accidental collapse
- QoS classes for routes:
  - low-latency control
  - bursty sensor data
  - best-effort telemetry
- per-route priority and deadline hints
- adaptive batching where batching reduces overhead without harming control data
- optional timestamp propagation and jitter measurement
- benchmark modes that reflect actual show traffic, not synthetic happy paths

What makes this superior:

- one bad sink cannot poison the whole bus
- a high-rate sensor stream cannot starve critical control traffic
- operators can choose graceful degradation instead of random failure

## Pillar 2: Fault Containment

Ordinary OSC networks often fail noisily and globally. This system should fail
locally and predictably.

Desired capabilities:

- per-destination circuit breakers
- quarantine mode for noisy or malformed senders
- bounded queues with explicit drop policy
- route-level rate limiting and burst caps
- configurable shed-load mode under overload
- validation layer that can reject malformed traffic before it contaminates the
  wider graph
- crash-resistant broker boundaries around optional plugins and adapters
- safe mode boot profile that starts the broker with risky extensions disabled

What makes this superior:

- problems become isolated incidents, not network-wide meltdowns
- debugging starts from known failure boundaries

## Pillar 3: Safety And Security

Raw OSC compatibility must remain intact, but shared networks and permanent
installations need stronger guarantees.

Desired capabilities:

- project-scoped namespaces
- broker-enforced ACLs
- optional signed or tokenized envelopes for secure ingress
- safe compatibility bridge:
  - secure traffic terminates at the broker
  - downstream legacy peers still receive plain compatible OSC
- sender identity and provenance tracking
- anti-spoofing and replay protection for secure modes
- route permissions:
  - read
  - write
  - transform
  - observe
- schema-aware validation for critical namespaces
- operator approval mode for dangerous config changes

What makes this superior:

- the system can live on hostile or chaotic networks
- security does not break legacy tools because it is additive

## Pillar 4: Fast Recovery And Continuity

This is one of the biggest opportunities to surpass ordinary OSC workflows.
Most OSC systems are fragile after restart.

Desired capabilities:

- stateful last-value cache
- namespace-level recovery policy
- warm restart that restores route config and selected runtime state
- crash-safe config snapshots
- durable event journal for selected streams
- late-joiner catch-up
- one-click rehydrate for restarted nodes
- rolling restart support for the broker
- optional active / standby broker pair
- health monitoring and automatic failover hooks

What makes this superior:

- a restarted app can return to a correct state quickly
- operators do not need to manually resend initialization sequences

## Pillar 5: Observability And Forensics

The system should make invisible network behavior visible.

Desired capabilities:

- real-time traffic topology view
- per-route throughput, latency, and drop metrics
- correlation IDs and provenance metadata
- time-travel packet buffer
- trigger-based capture:
  - on error
  - on latency spike
  - on drop threshold
- safe replay in sandbox mode
- rule debugger showing why a packet matched, transformed, or was dropped
- diff view for config changes and their traffic impact
- audit trail for operator actions

What makes this superior:

- issues can be diagnosed from inside the product instead of by external packet
  sniffing alone
- hard bugs become reproducible

## Pillar 6: Compatibility And Migration

Superiority only matters if adoption is easy.

Desired capabilities:

- raw OSC 1.0 over UDP remains first-class
- tolerant parsing for older senders that omit type tags
- strict, tolerant, and extended compatibility modes
- transparent localhost proxy mode
- easy fallback from enhanced features to plain OSC
- profile presets for:
  - UE5
  - TouchDesigner
  - browser control
  - sensors
  - lighting
- config import helpers for common existing setups
- sidecar deployment mode before deeper integration

What makes this superior:

- users can adopt it without rewriting their whole system
- the broker earns trust before asking for deeper commitment

## Pillar 7: Extensibility Without Core Fragility

Ordinary routers become messy when every special case is built into the core.

Desired capabilities:

- Cargo feature presets for lean builds
- Wasm transform plugins for safe user logic
- external process plugins for heavyweight integrations
- stable adapter SDK
- schema and code generation for typed workflows
- hot reload for selected transform modules
- simulation mode for testing plugin behavior

What makes this superior:

- custom behavior scales without turning the broker into an unstable monolith

## Pillar 8: Time And Sync Quality

Timing quality is one of the strongest reasons to build a new runtime around
OSC.

Desired capabilities:

- careful handling of OSC timetags without assuming unrealistic guarantees
- timestamp provenance:
  - created at source
  - observed at ingress
  - dispatched at egress
- jitter measurement per route
- clock quality reporting
- optional PTP / NTP / Ableton Link awareness
- deadline-aware scheduling for local dispatch
- diagnostics that distinguish transport delay from application delay

What makes this superior:

- users gain a measured timing model instead of vague hope

## Pillar 9: High Availability And Distributed Operation

If the product becomes infrastructure, it should not be tied to one process on
one machine forever.

Desired capabilities:

- broker-to-broker federation
- route replication across nodes
- active / standby failover
- edge brokers for local segmentation
- secure tunnel mode for remote links
- per-site policy packs

What makes this superior:

- the system can scale from one workstation to an installation network

## Pillar 10: Operator Experience

The best system is the one operators can trust under pressure.

Desired capabilities:

- dashboard that answers "what is happening right now?"
- safe mode startup
- traffic freeze / thaw for controlled recovery
- one-click isolate route
- one-click resend cached state
- clear warning levels:
  - notice
  - degraded
  - danger
- config validation before apply
- rollback after bad deploy
- guided recovery playbooks embedded in the UI

What makes this superior:

- it reduces panic during live incidents
- it turns advanced behavior into something humans can actually operate

## Non-Negotiable Principles

No matter how ambitious the feature set becomes, these rules should remain:

- raw OSC compatibility must never be casually broken
- enhanced behavior must be additive and negotiable
- the core data plane must stay simpler than the ecosystem around it
- diagnostics must not silently sabotage the hot path
- the system should prefer explicit degradation over hidden collapse

## Feature Universe Worth Exploring

Ignoring cost for now, these are especially attractive differentiators:

- route QoS classes and deadline-aware routing
- full time-travel capture and replay
- late-joiner rehydration plus warm restart
- broker-enforced namespace security
- active / standby broker continuity
- trigger-based incident capture
- schema-driven validation and code generation
- Wasm transform runtime
- per-route circuit breaker and quarantine model
- safe live config reload with rollback
- measured timing diagnostics with clock-quality reporting
- browser-native operations console

## Current Planning Stack

The following design notes now exist and should be kept consistent:

- internal packet and metadata model
- fault model and overload behavior
- recovery model and cache semantics
- compatibility matrix
- route configuration grammar
- benchmark workload definition
- plugin boundary note
- security overlay model
- operator workflow and recovery playbook
- transport and adapter contract
- dashboard information architecture
- native IPC ABI note
- federation and high-availability model
- config validation and migration note
- discovery and service-metadata model
- schema definition format
- C ABI reference header and error-code catalog
- deployment topology and release profile guide
- testing strategy and fuzz corpus plan
- adapter SDK API reference
- conformance vector and interoperability suite guide
- schema evolution and deprecation policy
- dashboard interaction spec and screen inventory
- release checklist and operational runbook

The following support documents now exist and should stay aligned with this
vision:

1. [Route Rule Cookbook And Worked Examples](../../design/en/route-rule-cookbook-and-worked-examples.md)
2. [Profile-Specific Operator Guides](../../design/en/profile-specific-operator-guides.md)
3. [Metrics And Telemetry Schema](../../design/en/metrics-and-telemetry-schema.md)
4. [Benchmark Result Interpretation Guide](../../design/en/benchmark-result-interpretation-guide.md)
5. [Architecture Decision Record Index](../../design/en/architecture-decision-record-index.md)
6. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)

## Summary

To become overwhelmingly better than ordinary OSC practice, the project needs
to win on more than speed. The real opportunity is to combine:

- deterministic performance
- containment of failure
- operational visibility
- security as an additive layer
- fast recovery
- compatibility-first adoption

That combination is what can turn a flexible message format into dependable
infrastructure.
