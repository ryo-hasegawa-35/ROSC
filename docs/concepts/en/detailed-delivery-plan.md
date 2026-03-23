# Detailed Delivery Plan

## Purpose

This document expands the phase roadmap into a more operational delivery plan.
It is intended to answer:

- what we should decide before writing significant code
- what order we should build things in
- what evidence we need before moving to the next phase
- where the highest uncertainty and cost actually sit

## Delivery Tracks

The project should be managed as six parallel design tracks, even if one person
implements most of it.

### Track A: Specification And Compatibility

Scope:

- OSC 1.0 parser and encoder correctness
- legacy tolerance policy
- OSC 1.1-inspired optional extensions
- address pattern mode selection
- framing behavior for UDP, TCP, and SLIP

Outputs:

- compatibility matrix
- canonical test vectors
- conformance suite
- documented "strict / tolerant / extended" modes

### Track B: Core Data Plane

Scope:

- ingress
- queueing
- routing
- fan-out
- egress isolation
- memory ownership and allocation policy

Outputs:

- packet flow design
- backpressure policy
- performance benchmarks
- overload behavior policy

### Track C: Control Plane And Operations

Scope:

- config model
- dashboard
- metrics
- logs
- capture / replay
- late joiner cache

Outputs:

- operator workflows
- observability requirements
- failure recovery behavior

### Track D: Adapter Ecosystem

Scope:

- WebSocket
- JSON
- MQTT
- mDNS / DNS-SD
- adapter SDK
- transport metadata

Outputs:

- adapter contract
- metadata model
- discovery UX

### Track E: Extensibility And Integration

Scope:

- Wasm filters
- schema
- code generation
- external plugin protocol
- UE5 / TouchDesigner integration
- shared memory IPC

Outputs:

- plugin model
- schema lifecycle
- IPC ABI
- native integration boundaries

### Track F: Security, Sync, And Release

Scope:

- zero-trust overlay
- project scoping
- access control
- Ableton Link
- installers
- packaging
- cross-platform service behavior

Outputs:

- secure deployment profiles
- sync quality model
- release packaging matrix

## Design Gates Before Major Implementation

The following decisions should be explicitly written down before deep
implementation starts.

### Gate 1: Compatibility Contract

We should lock:

- supported OSC 1.0 baseline behaviors
- tolerance behavior for missing type tags
- policy for unknown type tags
- whether `//` is globally disabled by default
- transport framing support matrix

### Gate 2: Internal Event Model

We should lock:

- internal packet representation
- normalized value representation
- metadata attached to ingress packets
- ownership and borrowing strategy
- timestamp representation inside the broker

### Gate 3: Route Model

We should lock:

- route rule grammar
- static transform capabilities for v1
- per-route cache policy
- drop / retry / isolate semantics
- how route configs are validated

### Gate 4: Operational Model

We should lock:

- what metrics are mandatory
- what logs are structured
- whether replay is dry-run by default
- what is safe to expose in the dashboard
- how config reload and rollback work

### Gate 5: Extension Boundary

We should lock:

- compile-time feature list
- Wasm plugin API shape
- external plugin IPC contract
- schema scope for v1
- native integration ownership boundaries

## Detailed Milestones

## Milestone 00A: Specification Freeze

Deliver:

- compatibility matrix
- packet examples and edge cases
- strict / tolerant / extended mode definition
- preliminary benchmark workload definitions

Questions to resolve:

- Do we pass through unsupported but parseable extension tags or drop them?
- Do we preserve original packet bytes for replay and diagnostics?
- How should timetag semantics be represented internally given the 1.1 caution?

## Milestone 00B: Repository And Test Harness

Deliver:

- Rust workspace layout
- test fixture folder
- fuzz targets
- benchmark harness
- CI matrix for Windows, macOS, Linux

Questions to resolve:

- single binary vs multiple crates
- embedded frontend asset strategy
- baseline dependencies allowed in the core crate

## Milestone 01A: Fast Path Prototype

Deliver:

- UDP ingress
- parser
- route match engine
- UDP egress
- metrics endpoint

Success condition:

- localhost proxy scenario works end to end
- packet forwarding behavior is measurable

## Milestone 01B: Overload And Isolation

Deliver:

- egress isolation per destination
- queue pressure accounting
- route drop policy
- burst traffic testing

Success condition:

- one bad consumer cannot stall unrelated consumers

## Milestone 02A: Operational Visibility

Deliver:

- basic dashboard
- route graph
- throughput and drop views
- structured logs

Success condition:

- operators can diagnose routing issues without Wireshark

## Milestone 02B: Recovery Tooling

Deliver:

- last-value cache
- late joiner sync
- capture and replay
- ring buffer inspection

Success condition:

- a failed node can be resynchronized
- a packet issue can be replayed safely

## Milestone 03A: Browser And Device Integration

Deliver:

- WebSocket adapter
- JSON message mapping
- adapter SDK draft

Success condition:

- browser-based monitoring and control work without impacting raw OSC flow

## Milestone 03B: Discovery And Stream Transports

Deliver:

- mDNS / DNS-SD support
- metadata publication
- TCP and SLIP framing modes

Success condition:

- discovery helps setup but manual fallback always remains possible

## Milestone 04A: Runtime Extensibility

Deliver:

- Wasm transform API
- plugin lifecycle
- hot reload

Success condition:

- a user-defined transform can be loaded without rebuilding the broker

## Milestone 04B: Schema And Codegen

Deliver:

- schema draft
- validator
- one codegen target for a real workflow

Success condition:

- schema catches integration mistakes before runtime for at least one real case

## Milestone 05A: IPC Core

Deliver:

- C ABI wrapper
- shared memory proof of concept
- local latency measurement tooling

Success condition:

- same logical route graph works over IPC as well as UDP

## Milestone 05B: Host Integration

Deliver:

- UE5 plugin
- TouchDesigner bridge plan or implementation
- fallback path to normal OSC

Success condition:

- native integration provides measurable benefit without becoming mandatory

## Milestone 06A: Security Overlay

Deliver:

- secure route profile
- scoped project IDs
- signed or tokenized access path
- abuse protection

Success condition:

- secure mode is available without breaking legacy operation

## Milestone 06B: Sync And Release Packaging

Deliver:

- Ableton Link integration
- sync diagnostics
- installers or packaged releases
- service-mode validation

Success condition:

- all three desktop operating systems can install and run supported builds

## Research And Validation Scenarios

The roadmap should be validated against at least these traffic profiles:

### Scenario 1: UE5 Localhost Proxy

- UE5 sends camera and gameplay control data to localhost
- broker forwards to original destination plus dashboard
- success metric: low added jitter and no stalls under burst load

### Scenario 2: TouchDesigner Sensor Storm

- high-frequency depth or tracking data
- multiple downstream consumers
- success metric: slow consumers do not collapse the whole bus

### Scenario 3: Exhibition Recovery

- one node restarts mid-show
- broker restores state via cache
- success metric: operator recovers without reinitializing the whole network

### Scenario 4: Browser Control Surface

- browser UI connected over WebSocket
- OSC nodes still receive plain compatible traffic
- success metric: web control does not interfere with base transport reliability

### Scenario 5: Shared Network Safety

- noisy shared network
- broker filters by project scope
- success metric: unrelated traffic is blocked without damaging legacy local use

## Benchmark Plan

Early benchmark numbers do not need to be marketing numbers; they need to be
decision-making numbers.

Measure at minimum:

- packets per second parse throughput
- end-to-end median routing latency
- p95 / p99 jitter
- queue growth under overload
- egress fairness across multiple destinations
- replay overhead when diagnostics are enabled

Benchmark modes:

- diagnostics off
- metrics only
- metrics plus capture
- cache enabled
- adapter fan-out enabled

## Current Planning Notes

These companion notes now exist and should remain aligned with this roadmap:

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

## Suggested Near-Term Work Order

If we continue planning before coding, these support documents should now be
treated as part of the active planning stack:

1. [Route Rule Cookbook And Worked Examples](../../design/en/route-rule-cookbook-and-worked-examples.md)
2. [Profile-Specific Operator Guides](../../design/en/profile-specific-operator-guides.md)
3. [Metrics And Telemetry Schema](../../design/en/metrics-and-telemetry-schema.md)
4. [Benchmark Result Interpretation Guide](../../design/en/benchmark-result-interpretation-guide.md)
5. [Architecture Decision Record Index](../../design/en/architecture-decision-record-index.md)
6. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)

## Recommended v0.1 Scope

v0.1 should include only:

- OSC 1.0 UDP core
- tolerant parser
- route rules
- destination isolation
- metrics
- minimal dashboard or metrics endpoint
- stress tests and fuzzing

v0.1 should exclude:

- MQTT
- full discovery automation
- Wasm runtime
- schema and codegen
- native UE5 / TouchDesigner plugins
- zero-trust
- Ableton Link

## Why This Order Is Good

It preserves the most important promise first:

- raw OSC compatibility
- stable routing under load
- observability under failure

Everything else becomes easier if the pipe is already trustworthy.
