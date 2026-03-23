# Implementation Readiness Checklist

## Purpose

This checklist helps future implementation work start from stable design
assumptions rather than from improvisation.

It is intentionally written before coding begins.

## Before Any Core Coding

Confirm that these documents have been read and accepted as current:

- [GitHub Foundation And Collaboration Plan](../../concepts/en/github-foundation-and-collaboration-plan.md)
- [Architecture Principles](./architecture-principles.md)
- [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
- [Compatibility Matrix](./compatibility-matrix.md)
- [Route Configuration Grammar](./route-configuration-grammar.md)

## Before Repository / GitHub Setup

Confirm:

- protected branch expectations are documented
- review ownership boundaries are clear enough for `CODEOWNERS`
- issue and label taxonomy match the architecture areas
- documentation quality checks exist or are planned first

## Before Parser / Encoder Work

Confirm:

- compatibility modes are understood
- legacy missing-type-tag policy is accepted
- unknown tagged value behavior is accepted
- raw packet retention policy is clear

## Before Route Authoring Or Example Publication

Confirm:

- route grammar and cookbook examples still agree
- examples do not accidentally imply unsupported fields as normative
- route IDs and destination IDs follow the intended naming discipline

## Before Routing Core Work

Confirm:

- normalized packet model is stable enough
- route grammar is stable enough
- cookbook examples reflect intended hot-path use cases
- traffic classes are agreed
- queue boundaries are explicit
- overload actions are explicit

## Before Observability Work

Confirm:

- operator questions are known
- health states are consistent with fault model
- telemetry levels and canonical metric names are agreed
- cardinality limits are explicit
- replay and rehydrate remain distinct
- diagnostics budget is bounded

## Before Benchmarking Or Performance Claims

Confirm:

- workload definition and interpretation guide are both current
- benchmark context fields are defined
- comparison rules are understood
- release claims will distinguish speed from trustworthiness

## Before Recovery Work

Confirm:

- cache classes are defined
- dangerous trigger traffic is marked non-automatic
- warm vs durable persistence is understood
- route-level recovery policy exists

## Before Plugin / Adapter Work

Confirm:

- plugin trust tiers are defined
- adapter capability contract is accepted
- security boundary remains broker-owned
- plugin failure containment is explicit

## Before Discovery / Metadata Work

Confirm:

- discovery does not bypass manual configuration
- trust levels are explicit
- service metadata shape is accepted
- stale discovery handling is defined

## Before Schema / Codegen Work

Confirm:

- schema remains optional
- schema type system aligns with packet model
- code generation targets are scoped
- schema validation levels are understood

## Before SDK / External Adapter Work

Confirm:

- adapter SDK contract exists
- capability declaration vocabulary is stable enough
- ownership rules are explicit
- interoperability evidence expectations are known

## Before Native IPC Work

Confirm:

- fallback path exists
- host-facing ABI stability matters more than convenience
- shared memory ownership rules are accepted
- observability requirements for IPC exist

## Before Distributed / HA Work

Confirm:

- broker identity model exists
- replication scope is defined
- split-brain prevention model exists
- failover trigger policy exists

## Before Config Hot Reload Work

Confirm:

- validation stages are defined
- last-known-good policy exists
- migration visibility is preserved
- risky applies have review or confirmation rules

## Before Packaging / Release Work

Confirm:

- deployment topologies are documented
- release profile contents are documented
- fallback story exists per profile
- testing expectations are written for the profile

## Before Declaring Compatibility Claims

Confirm:

- conformance vectors exist
- interoperability scenarios exist
- regression policy is defined
- release notes map to actual evidence

## Before Architecture-Changing Work

Confirm:

- the ADR index reflects the decision area
- a proposed or accepted ADR exists for non-trivial semantic shifts
- affected design documents are listed explicitly

## Ready-To-Code Gate

Coding should begin only when:

- the affected design docs exist
- terms are consistent
- failure behavior is explicit
- fallback behavior is explicit
- non-negotiable invariants are clear
- repository review discipline will not undermine the design intent

## Red Flags

Pause implementation if any of these are true:

- route semantics depend on undocumented adapter behavior
- recovery behavior is assumed rather than written
- a performance optimization requires weakening compatibility without review
- a plugin needs unrestricted broker internals
- a local acceleration path would become mandatory
- repository process would allow high-risk merges without architecture review
