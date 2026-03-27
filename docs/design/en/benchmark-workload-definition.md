# Benchmark Workload Definition

## Purpose

This document defines the workload suite used to evaluate performance,
predictability, and fault behavior.

The goal is not vanity throughput numbers. The goal is to measure whether the
broker remains trustworthy under realistic pressure.

## Measurement Principles

- Measure latency and jitter, not throughput alone.
- Measure overload behavior, not just steady state.
- Measure with diagnostics on and off.
- Measure route isolation, not only aggregate performance.
- Measure recovery time after disruption.

## Core Metrics

- packets per second
- median routing latency
- p95 routing latency
- p99 routing latency
- jitter distribution
- queue depth growth
- drop count by reason
- breaker open events
- rehydrate latency
- restart recovery time

## Test Environments

### Local Workstation

Use for:

- localhost proxy
- shared memory comparison
- dashboard overhead

### Small Network

Use for:

- discovery
- shared network noise
- multiple destination behavior

### Degraded / Synthetic Fault Environment

Use for:

- malformed traffic
- stalled consumers
- transform timeout
- adapter disconnect

## Workload Suite

### Workload A: Localhost Control Path

Intent:

- baseline for low-latency control traffic

Traffic:

- moderate rate scalar control messages
- small number of critical destinations

Measure:

- added latency
- jitter
- cost of metrics and dashboard tap

### Workload B: Sensor Storm

Intent:

- test bursty high-rate streams

Traffic:

- large volume sensor-like packets
- fan-out to multiple destinations

Measure:

- control-path isolation
- queue growth
- drop policy behavior

### Workload C: Mixed Show Traffic

Intent:

- realistic mixed environment

Traffic mix:

- camera and control values
- tracking data
- telemetry
- dashboard subscriptions

Measure:

- fairness between traffic classes
- tail latency
- effect of capture and metrics

### Workload D: Slow Destination

Intent:

- validate per-destination isolation

Traffic:

- one intentionally stalled destination
- multiple healthy destinations

Measure:

- whether healthy destinations remain stable
- breaker behavior
- queue containment

### Workload E: Malformed Traffic Flood

Intent:

- validate parser hardening and quarantine

Traffic:

- invalid packets
- truncated bundles
- random type tags

Measure:

- crash resistance
- quarantine timing
- healthy traffic continuity

### Workload F: Recovery And Rehydrate

Intent:

- validate continuity after restart or reconnect

Traffic:

- stateful control namespaces
- selected late joiner behavior

Measure:

- restart recovery time
- rehydrate correctness
- stale cache handling

### Workload G: Wasm Transform Boundary

Intent:

- quantify host/Wasm overhead on packet transforms

Traffic:

- repeated scalar control packets
- mixed route classes with Wasm disabled vs enabled

Measure:

- added per-packet latency
- jitter increase on the hot path
- copy count or allocation evidence where available

### Workload H: Schema Validation Depth

Intent:

- compare validation depth against control-plane safety gains

Traffic:

- same namespace under `off`, `shape_only`, `typed`, and `strict`

Measure:

- throughput cost
- p95 and p99 latency delta
- impact on bursty sensor traffic

### Workload I: Security Overlay Jitter

Intent:

- validate whether secure ingress stays inside the route jitter budget

Traffic:

- synchronized control traffic in plain mode and secure mode

Measure:

- added verification cost
- jitter spread increase
- whether secure mode changes tail latency enough to threaten sync-sensitive
  workloads

## Feature Toggle Matrix

Each workload should run under at least these modes:

- core only
- metrics enabled
- metrics plus dashboard
- capture enabled
- cache enabled
- transform enabled
- security overlay enabled where relevant

## Benchmark Reporting Format

Every run should record:

- git revision or document revision
- operating system
- CPU class
- active feature toggles
- workload definition version
- route count
- destination count

## Success Interpretation

The system is improving when:

- p95 and p99 stay bounded under realistic pressure
- critical traffic remains stable during sensor floods
- degraded mode is explicit instead of chaotic
- recovery is fast and correct
- diagnostics cost is measurable and acceptable

## Non-Negotiable Rules

- benchmark traffic classes must reflect actual intended product use
- every benchmark suite must include a fault or overload case
- reported latency must distinguish ingress-to-egress from end-to-end external
  network timing where possible
- benchmark results must be reproducible enough to compare revisions
