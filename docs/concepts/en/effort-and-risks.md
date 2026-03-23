# Effort And Risks

## Assumptions

The rough estimates below assume:

- 1 senior Rust engineer driving the core full-time
- occasional frontend help for the dashboard
- occasional C++ help for UE5 integration
- proper CI, tests, and benchmarking are included
- the goal is production-grade reliability, not just a prototype

## Rough Engineering Cost

These are implementation-hour ranges, not calendar guarantees.

| Phase | Focus | Estimated Hours |
| --- | --- | ---: |
| 00 | Spec baseline, tests, benches, repo setup | 40-80 |
| 01 | OSC core proxy and routing engine | 120-200 |
| 02 | Dashboard, cache, replay, observability | 120-180 |
| 03 | Adapters, discovery, metadata | 160-260 |
| 04 | Plugins, Wasm, schema, codegen | 160-300 |
| 05 | Shared memory IPC, UE5 / TD native integration | 220-400 |
| 06 | Security overlays, sync, release hardening | 180-320 |
| Total | Full roadmap | 1000-1740 |

In practice this is roughly:

- 6-10 person-months for a focused technical build
- 9-15 months if done part-time while learning requirements from real shows

## Highest-Risk Areas

### 1. Shared memory native integration

Risk:

- hardest part to stabilize
- platform-specific
- debugging complexity rises sharply
- TouchDesigner and UE5 plugin maintenance cost continues after first release

Mitigation:

- keep IPC binary protocol minimal
- ship the broker first
- treat native plugins as a later acceleration path

### 2. Security overlays

Risk:

- can damage plug-and-play adoption
- can create lockouts in live environments
- requires careful UX, fallback behavior, and recovery tooling

Mitigation:

- optional modes
- clear insecure / secure profiles
- "broker terminates security, downstream stays compatible" architecture

### 3. Schema and code generation

Risk:

- easy to overdesign
- can ossify workflows too early
- needs clear value for UE5 / TD users to justify the complexity

Mitigation:

- start with schema as optional linting and documentation
- generate bindings for a small set of real use cases first

### 4. Ableton Link and timing guarantees

Risk:

- users will expect near-perfect sync once this feature exists
- network, OS scheduling, and app behavior still impose hard limits

Mitigation:

- expose measured timing quality
- separate "timestamp propagation" from "guaranteed sync"

## Recommended Resource Plan

- One owner for broker core, parser, routing, tests, and performance.
- One part-time owner for dashboard, frontend, and UX.
- One part-time owner for native integration once Phase 05 starts.
- Real user testing with at least 2 show-style workloads before security and
  native integration are declared stable.

## Additional Features Worth Considering

- Route-level priority and backpressure policies
- Config hot reload with validation and rollback
- OpenTelemetry or Prometheus metrics export
- Health checks and heartbeats
- Traffic recording export format for offline analysis
- Simulation mode with synthetic traffic generators
- Preset profiles for UE5, TouchDesigner, lighting, and sensors
- Crash-safe config snapshots and automatic recovery
- Rule debugger that shows why a packet matched or was dropped

## Suggested Milestone Definition

The project should only move to the next phase when the previous one has:

- reproducible benchmarks
- fuzz coverage where applicable
- cross-platform CI passing
- at least one real-world scenario validated
