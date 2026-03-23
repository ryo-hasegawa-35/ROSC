# Adapter SDK API Reference

## Purpose

This document defines the intended API surface for the adapter SDK.

It is not an implementation package reference yet. It is the architectural
reference that future SDK implementations should follow so that adapters behave
predictably and safely across transports and deployment profiles.

## Design Goals

- keep the adapter SDK smaller than the broker internals
- make capabilities explicit
- preserve transport neutrality at the broker core
- isolate adapter failure from routing correctness
- make SDK usage auditable and observable

## SDK Scope

The adapter SDK should help adapter authors do the following:

- register adapter identity and capabilities
- publish ingress packets into the broker
- consume egress work from the broker
- report health, state, and errors
- expose discovery results and metadata

The SDK should not encourage adapters to:

- mutate route semantics
- bypass broker security policy
- hold undocumented global broker state
- depend on internal Rust crate layout

## Conceptual API Surface

The SDK should expose conceptual modules for:

- adapter registration
- ingress publishing
- egress subscription
- health reporting
- discovery reporting
- diagnostics reporting
- lifecycle control

## Registration API

An adapter should register:

- `adapter_id`
- `adapter_kind`
- `protocol_family`
- `sdk_contract_version`
- `capabilities`
- `supported_profiles`

Registration should fail clearly if:

- capability declarations are invalid
- contract versions are incompatible
- required fields are missing

## Ingress Publishing API

The ingress-facing API should let an adapter submit:

- immutable raw payload or canonical message payload
- ingress metadata
- source endpoint reference
- receive timestamp
- security or discovery metadata if available

Rules:

- ingress payload is append-only at submission time
- adapters may annotate, but not rewrite route meaning
- ingress submission must return explicit backpressure or failure signals

## Egress Consumption API

The egress-facing API should let an adapter receive:

- packet reference or payload
- destination reference
- send policy hint
- delivery expectations
- correlation metadata where allowed

The adapter should return:

- success
- retryable failure
- terminal failure
- timeout
- backpressure condition

## Health Reporting API

An adapter should report at least:

- adapter state
- adapter health class
- session count where relevant
- disconnect count
- reconnect count
- local pressure signals

## Discovery Reporting API

Discovery-capable adapters should be able to submit:

- discovered service record
- freshness or TTL
- trust classification if known
- raw observation source

## Diagnostics API

The SDK should support structured reporting for:

- adapter errors
- warnings
- transport anomalies
- dropped or rejected traffic counts
- degraded state transitions

## Capability Declaration

Capabilities should be declared as stable names or flags, not ad hoc strings
buried in adapter logic.

Examples:

- message-oriented
- stream-oriented
- secure-identity
- discovery-capable
- ordered-delivery
- best-effort-delivery
- native-binary-payload

## State Model

The SDK should standardize lifecycle states:

- `Init`
- `Registering`
- `Ready`
- `Degraded`
- `Disconnected`
- `Recovering`
- `Stopped`

## Error Model

The SDK should prefer:

- stable error categories
- machine-readable error codes
- optional structured detail

The SDK should avoid:

- panic as public contract behavior
- undocumented side effects
- implicit retries with invisible semantics

## Threading Expectations

The SDK should document:

- what is safe to call concurrently
- what requires exclusive access
- whether callbacks are synchronous, asynchronous, or event-driven

## Resource Ownership

The SDK should make clear:

- who owns packet buffers
- who owns session handles
- when metadata snapshots are immutable
- how cleanup occurs on failure

## Versioning Policy

The adapter SDK should define:

- SDK contract version
- feature negotiation mechanism
- deprecation window policy

## Reference Adapter Categories

The SDK should be able to support at least:

- UDP OSC adapter
- TCP / SLIP OSC adapter
- WebSocket / JSON adapter
- MQTT adapter
- shared-memory IPC adapter
- discovery-only adapter

## Observability Requirements

Every SDK-based adapter should be able to surface:

- version
- state
- health
- throughput
- failure counts
- reconnect behavior
- capability declaration

## Non-Negotiable Invariants

- the SDK must not expose more power than the architecture intends
- adapter authors must not need access to private broker internals
- adapter behavior must remain inspectable and diagnosable
- capability and health signals must be explicit, not inferred from silence
