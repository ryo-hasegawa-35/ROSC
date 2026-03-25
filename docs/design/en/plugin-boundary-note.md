# Plugin Boundary Note

## Purpose

This document defines how extensibility should work without destabilizing the
broker core.

The guiding rule is:

- put flexibility at the edge
- keep the data plane small and predictable

## Extension Layers

The project should support four extension layers with different trust and
capability levels.

### Layer 1: Compile-Time Features

Examples:

- dashboard
- discovery
- mqtt adapter
- websocket adapter
- ue5 integration

Use when:

- the capability is product-level
- the capability is trusted
- static packaging is acceptable

### Layer 2: Wasm Transforms

Examples:

- smoothing
- scaling
- remapping
- custom filtering

Use when:

- user logic needs hot reload
- deterministic sandboxing matters
- packet-level transforms are sufficient

### Layer 3: External Process Plugins

Examples:

- proprietary hardware bridges
- cloud connectors
- heavyweight analysis modules

Use when:

- the feature is large
- failure containment matters
- stable IPC contract is preferable to in-process ABI

### Layer 4: Native Host Integrations

Examples:

- UE5 plugin
- TouchDesigner bridge

Use when:

- local performance or UX requires deeper integration
- the feature is intentionally closer to the host application

## What Should Never Be The Main Plugin Model

Avoid relying on in-process native Rust dynamic library loading as the primary
extension model. ABI stability and fault containment are both weak there.

## Plugin Trust Tiers

### Trusted

- bundled with the product
- can be enabled in production profiles

### Approved

- installed by operator policy
- monitored and bounded

### Experimental

- disabled by default
- safe mode may turn these off automatically

## Wasm Plugin Contract

Inputs should include:

- normalized packet view when safely available
- limited metadata
- route context

Outputs may include:

- pass through unchanged
- emit derived packet
- drop packet with reason

Constraints:

- no direct network I/O
- bounded memory
- bounded execution time
- no direct mutation of broker state

## Wasm Hot-Path Guardrails

Wasm is valuable for portability and containment, but it must not quietly
become the default mechanism for the most latency-sensitive control path.

Rules:

- critical low-jitter routes should default to native core transforms or
  compile-time features
- Wasm transforms must be opt-in per route, never implicit for all traffic
- the host/Wasm boundary must minimize copying and prefer borrowed or shared
  packet views where safely possible
- every Wasm-capable route class should have benchmark evidence for added
  latency and jitter, not just throughput
- if deterministic performance cannot be demonstrated, the feature belongs off
  the hot path and behind an operator warning

## External Plugin Contract

Recommended properties:

- explicit protocol version
- capability advertisement
- bounded request / response sizes
- timeout policy
- disconnect handling
- health telemetry

External plugins should communicate with the broker over a stable contract, not
through arbitrary shared memory or undocumented host calls.

## Host Integration Boundary

Native host integrations should not be allowed to redefine broker semantics.

They may:

- accelerate transport
- improve embedding
- provide host-specific discovery and tooling

They should not:

- redefine route semantics
- bypass broker authorization
- break compatibility fallback

## Failure Policy

- plugin failure is local
- repeated plugin failure opens a breaker
- safe mode can disable plugins and still boot the broker core
- plugin telemetry is visible in operator views

## Data Access Policy

Plugins should only receive what they need.

Possible access tiers:

- address only
- normalized packet
- normalized packet plus limited metadata
- route-local cache access

Direct unrestricted access to global broker internals should be avoided.

## Versioning Policy

Every plugin contract should include:

- contract version
- minimum broker version
- declared capabilities
- declared resource limits

## Observability Requirements

The broker should expose:

- plugin load status
- plugin version
- plugin latency
- plugin timeout count
- plugin error count
- plugin disable events

## Non-Negotiable Invariants

- core routing must remain useful with all optional plugins disabled
- plugin failure must not corrupt packet lineage
- plugins must not erase raw-packet replay capability
- in-process extensibility must not outrank safety and containment
