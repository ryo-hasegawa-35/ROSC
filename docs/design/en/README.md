# ROSC Design Specs

This folder contains the technical specification set for the broker core,
configuration semantics, observability model, recovery model, extensibility
boundaries, delivery discipline, and long-term design governance.

Concept, roadmap, and GitHub planning material lives in
[Concepts / Planning](../../concepts/en/README.md).

## Start Here

- [Design Reading Order](./reading-order.md)
- [Glossary](./glossary.md)
- [Implementation Readiness Checklist](./implementation-readiness-checklist.md)

## Normative Core

- [Architecture Principles](./architecture-principles.md)
- [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
- [Compatibility Matrix](./compatibility-matrix.md)
- [Route Configuration Grammar](./route-configuration-grammar.md)

## Runtime Behavior And Operations

- [Route Rule Cookbook And Worked Examples](./route-rule-cookbook-and-worked-examples.md)
- [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
- [Benchmark Workload Definition](./benchmark-workload-definition.md)
- [Benchmark Fixture Inventory And Reproducibility Plan](./benchmark-fixture-inventory-and-reproducibility-plan.md)
- [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)
- [Dashboard Information Architecture](./dashboard-information-architecture.md)
- [Dashboard Interaction Spec And Screen Inventory](./dashboard-interaction-spec-and-screen-inventory.md)
- [Operator Workflow And Recovery Playbook](./operator-workflow-and-recovery-playbook.md)
- [Profile-Specific Operator Guides](./profile-specific-operator-guides.md)
- [Config Validation And Migration Note](./config-validation-and-migration-note.md)
- [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

## Integration And Extensibility

- [Transport And Adapter Contract](./transport-and-adapter-contract.md)
- [Plugin Boundary Note](./plugin-boundary-note.md)
- [Adapter SDK API Reference](./adapter-sdk-api-reference.md)
- [Discovery And Service Metadata Model](./discovery-and-service-metadata-model.md)
- [Schema Definition Format](./schema-definition-format.md)
- [Schema Evolution And Deprecation Policy](./schema-evolution-and-deprecation-policy.md)
- [Security Overlay Model](./security-overlay-model.md)
- [Native IPC ABI Note](./native-ipc-abi-note.md)
- [C ABI Reference Header And Error-Code Catalog](./c-abi-reference-header-and-error-code-catalog.md)

## Distributed Operation, Delivery, And Evidence

- [Federation And High-Availability Model](./federation-and-high-availability-model.md)
- [Deployment Topology And Release Profile Guide](./deployment-topology-and-release-profile-guide.md)
- [Testing Strategy And Fuzz Corpus Plan](./testing-strategy-and-fuzz-corpus-plan.md)
- [Rust Workspace And Crate Boundaries](./rust-workspace-and-crate-boundaries.md)
- [OSC Conformance Corpus Plan](./osc-conformance-corpus-plan.md)
- [Conformance Vector And Interoperability Suite Guide](./conformance-vector-and-interoperability-suite-guide.md)
- [Architecture Decision Record Index](./architecture-decision-record-index.md)
- [ADR Folder](../adr/en/README.md)

## Related Planning Documents

- [Detailed Delivery Plan](../../concepts/en/detailed-delivery-plan.md)
- [GitHub Foundation And Collaboration Plan](../../concepts/en/github-foundation-and-collaboration-plan.md)
- [CI Expansion And Required-Check Roadmap](../../concepts/en/ci-expansion-and-required-check-roadmap.md)

## Language And References

- [Design Specs (Japanese)](../ja/README.md)
- [OSC 1.0 Specification PDF](../../references/osc-1.0-specification.pdf)
- [OSC 1.1 NIME 2009 PDF](../../references/osc-1.1-nime-2009.pdf)
