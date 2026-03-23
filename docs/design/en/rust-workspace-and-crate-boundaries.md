# Rust Workspace And Crate Boundaries

## Purpose

This document freezes the intended Rust workspace shape before code begins.

## Boundary Goals

- keep the hot path small and dependency-light
- separate protocol semantics from runtime orchestration
- let adapters, recovery, and telemetry evolve without contaminating parser
  correctness
- keep optional features optional

## Proposed Top-Level Layout

```text
/Cargo.toml                     workspace only
/crates/rosc-osc               OSC parse and encode primitives
/crates/rosc-packet            internal packet and metadata model
/crates/rosc-route             route matching and routing decisions
/crates/rosc-runtime           ingress, scheduling, egress isolation
/crates/rosc-config            configuration loading and semantic validation
/crates/rosc-telemetry         metrics, events, health export
/crates/rosc-recovery          cache, rehydrate, replay policy
/crates/rosc-adapter-sdk       stable adapter-facing contract
/crates/rosc-plugin-sdk        stable plugin-facing contract
/crates/rosc-security          additive security overlay services
/apps/rosc-broker              executable broker process
/apps/rosc-dashboard-api       optional dashboard-facing backend surface
/fixtures/                     conformance and benchmark inputs
/docs/                         architecture and planning documents
```

## Phase 01 Minimal Bootstrap Set

Only these crates should exist in the first coding window:

- `rosc-osc`
- `rosc-packet`
- `rosc-route`
- `rosc-runtime`
- `rosc-config`
- `rosc-telemetry`
- `apps/rosc-broker`

Everything else should remain planned until the core proves stable.

## Responsibility Matrix

- `rosc-osc`
  - owns OSC byte-level parsing and encoding behavior
  - must not depend on runtime, telemetry, or recovery crates
- `rosc-packet`
  - owns normalized packet representation and retained raw metadata
  - may depend on `rosc-osc`, but not on runtime orchestration
- `rosc-route`
  - owns route evaluation, destination selection, and traffic-class decisions
  - must stay independent from transport-specific adapter code
- `rosc-runtime`
  - owns ingress queues, task orchestration, egress isolation, and breaker flow
  - may depend on route, packet, telemetry, and security boundaries
- `rosc-config`
  - owns external config parsing, semantic validation, and last-known-good logic
  - must translate config into route/runtime structures without hidden defaults
- `rosc-telemetry`
  - owns metrics names, event emission, and health-reporting surfaces
  - must remain bounded and should not own routing decisions

## Dependency Rules

- adapters depend inward on the core, never the reverse
- telemetry is consumed by runtime, not used as a decision engine
- recovery depends on packet, route, runtime, and config semantics, but parser
  correctness must not depend on recovery
- plugin and adapter SDK crates expose stable boundaries and must not expose
  unrestricted core internals

## Decisions Frozen Before Coding

- workspace remains library-first, with one broker app as the first executable
- UDP / OSC compatibility is implemented in core crates, not as an external
  plugin
- dashboard UI is not allowed to depend directly on runtime internals
- IPC, federation, and advanced adapters remain later crates, not Phase 01
  bootstrap requirements

## Non-Goals

This document does not create the workspace yet. It defines the crate
boundaries so the first implementation PRs do not improvise them.
