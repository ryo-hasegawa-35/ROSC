# Design Reading Order

## Purpose

This document explains how to read the design set efficiently.

The design folder is no longer a small collection of notes. It is becoming a
system architecture reference. Without a reading order, future implementation
work will slow down because people will repeatedly rediscover the same context.

## Recommended Reading Paths

## Path A: Fast Architectural Orientation

Read these first if you want the shortest path to understanding the system:

1. [Architecture Principles](./architecture-principles.md)
2. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
3. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
4. [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
5. [Compatibility Matrix](./compatibility-matrix.md)

This path answers:

- what the system is trying to preserve
- how packets are represented
- how failure is contained
- how state is restored
- how compatibility is enforced

## Path B: Routing And Runtime Behavior

Read these if you are about to work on the broker core:

1. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
2. [Compatibility Matrix](./compatibility-matrix.md)
3. [Route Configuration Grammar](./route-configuration-grammar.md)
4. [Route Rule Cookbook And Worked Examples](./route-rule-cookbook-and-worked-examples.md)
5. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
6. [Benchmark Workload Definition](./benchmark-workload-definition.md)

This path answers:

- what the hot path sees
- what route configuration must express
- how route authoring should look in practice
- how overload must behave
- how performance should be measured

## Path C: Operations And Trust

Read these if you are working on observability, recovery, or operator UX:

1. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
2. [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
3. [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
4. [Dashboard Information Architecture](./dashboard-information-architecture.md)
5. [Operator Workflow And Recovery Playbook](./operator-workflow-and-recovery-playbook.md)
6. [Profile-Specific Operator Guides](./profile-specific-operator-guides.md)
7. [Config Validation And Migration Note](./config-validation-and-migration-note.md)

This path answers:

- what unhealthy states exist
- what operators are allowed to see and alert on
- how operators recover
- how state is exposed safely
- how configuration changes stay trustworthy

## Path D: Extensibility And Integration

Read these if you are working on plugins, adapters, or host integration:

1. [Plugin Boundary Note](./plugin-boundary-note.md)
2. [Transport And Adapter Contract](./transport-and-adapter-contract.md)
3. [Security Overlay Model](./security-overlay-model.md)
4. [Native IPC ABI Note](./native-ipc-abi-note.md)
5. [Federation And High-Availability Model](./federation-and-high-availability-model.md)

This path answers:

- what may extend the system
- what must remain stable
- where security and identity are enforced
- how local native acceleration fits the architecture
- how multi-broker systems should behave

## Path E: Discovery, Packaging, And Verification

Read these if you are shaping release engineering, schema tooling, or delivery
quality:

1. [Discovery And Service Metadata Model](./discovery-and-service-metadata-model.md)
2. [Schema Definition Format](./schema-definition-format.md)
3. [Deployment Topology And Release Profile Guide](./deployment-topology-and-release-profile-guide.md)
4. [Testing Strategy And Fuzz Corpus Plan](./testing-strategy-and-fuzz-corpus-plan.md)
5. [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)
6. [Config Validation And Migration Note](./config-validation-and-migration-note.md)

This path answers:

- how services become visible
- how typed tooling should evolve
- how product shapes map to deployments
- how trust in implementation quality should be built

## Path F: Delivery Discipline And External Integration

Read these if you are preparing SDKs, release process, or interoperability
evidence:

1. [Adapter SDK API Reference](./adapter-sdk-api-reference.md)
2. [Conformance Vector And Interoperability Suite Guide](./conformance-vector-and-interoperability-suite-guide.md)
3. [Schema Evolution And Deprecation Policy](./schema-evolution-and-deprecation-policy.md)
4. [Dashboard Interaction Spec And Screen Inventory](./dashboard-interaction-spec-and-screen-inventory.md)
5. [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

This path answers:

- how external adapter authors should integrate
- how compatibility claims are evidenced
- how schema change is kept safe over time
- how dashboard structure becomes operator interaction
- how releases stay operable, not just buildable

## Path G: Evidence, Telemetry, And Decision Discipline

Read these if you are about to make broad cross-cutting changes or claim
production readiness:

1. [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
2. [Benchmark Workload Definition](./benchmark-workload-definition.md)
3. [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)
4. [Architecture Decision Record Index](./architecture-decision-record-index.md)
5. [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

This path answers:

- what the system must emit as evidence
- how performance claims should be interpreted
- how major decisions should remain traceable
- how release confidence is justified

## If You Only Read Three Documents

Read:

1. [Architecture Principles](./architecture-principles.md)
2. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
3. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)

These three documents hold the strongest architectural constraints.

## Implementation Sequence By Topic

If implementation begins later, the safest sequence is:

1. packet model
2. compatibility contract
3. route grammar
4. fault behavior
5. recovery behavior
6. benchmark harness expectations
7. adapter contract
8. operator-facing systems
9. native and distributed extensions

## Documents That Should Be Treated As Normative

These should be treated as architecture-level constraints, not casual notes:

- architecture principles
- internal packet and metadata model
- compatibility matrix
- fault model and overload behavior
- recovery model and cache semantics
- route configuration grammar

## Documents That Are More Operational And Evolvable

These should remain aligned with the normative documents, but may evolve more
rapidly as product shape becomes clearer:

- dashboard information architecture
- operator workflow and recovery playbook
- profile-specific operator guides
- benchmark workload definition
- benchmark result interpretation guide
- metrics and telemetry schema
- config validation and migration note
- discovery and service metadata model
- schema definition format
- deployment topology and release profile guide
- testing strategy and fuzz corpus plan
- adapter SDK API reference
- conformance vector and interoperability suite guide
- schema evolution and deprecation policy
- dashboard interaction spec and screen inventory
- release checklist and operational runbook
- architecture decision record index

## Cross-Folder Planning Companion

Before setting up repository rules or starting large implementation work, also
read:

- [GitHub Foundation And Collaboration Plan](../../concepts/en/github-foundation-and-collaboration-plan.md)

## Reading Rule For Future Contributors

Before changing a core behavior, contributors should confirm whether the change
touches:

- compatibility
- packet semantics
- fault containment
- recovery semantics
- operator trust

If yes, they should reread the relevant normative documents before coding.
