# ROSC Concepts And Planning

This folder captures the project vision, phased roadmap, planning notes,
delivery priorities, and collaboration setup for a next-generation OSC routing
bus built in Rust.

## Source Baseline

This roadmap is grounded in the following primary sources:

- `../../references/osc-1.0-specification.pdf`
- `../../references/osc-1.1-nime-2009.pdf`
- https://opensoundcontrol.stanford.edu/spec-1_0.html
- https://opensoundcontrol.stanford.edu/spec-1_1.html
- https://opensoundcontrol.stanford.edu/files/2009-NIME-OSC-1.1.pdf
- https://opensoundcontrol.stanford.edu/

Important interpretation notes:

- OSC 1.0 is the only fully published specification in the classic format.
- The OpenSoundControl site states that OSC 1.1 does not have a separately
  published formal specification page in the style of 1.0; the 2009 NIME paper
  is the authoritative reflection of the 1.1 vision.
- Well-formed OSC 1.0 messages should continue to work in 1.1-oriented systems.
- OSC is best treated as an encoding/content format, not a complete
  inter-application protocol. Discovery, security, schema, and service behavior
  should be layered on top.

## Project Principles

- Preserve strict backward compatibility with existing OSC 1.0 traffic.
- Treat optional higher-level behavior as additive, never mandatory for raw OSC.
- Keep the routing core small, deterministic, and independently testable.
- Build the system as a modular platform so features can be shipped as packages
  or plugins rather than one monolith.
- Support Windows, macOS, and Linux from the beginning of the build and test
  pipeline.

## Phase Map

- [Phase 00](./phase-00-foundation.md): Specification baseline, benchmarks, test
  harness, and repository foundation.
- [Phase 01](./phase-01-core-proxy.md): Local OSC proxy, routing engine, and
  transport core.
- [Phase 02](./phase-02-observability-recovery.md): Dashboard, cache, capture,
  replay, and operational tooling.
- [Phase 03](./phase-03-adapters-discovery.md): WebSocket, MQTT, discovery, and
  protocol metadata.
- [Phase 04](./phase-04-extensibility-schema.md): Plugin model, Wasm filters,
  schema, and code generation.
- [Phase 05](./phase-05-native-integration.md): Shared memory IPC and native
  UE5 / TouchDesigner integration.
- [Phase 06](./phase-06-security-sync-release.md): Zero-trust overlays, sync,
  hardening, and cross-platform release packaging.

## Supporting Planning Documents

- [Advantage Requirements](./advantage-requirements.md)
- [Effort And Risks](./effort-and-risks.md)
- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [License And Contributor Policy](./license-and-contributor-policy.md)
- [CI Expansion And Required-Check Roadmap](./ci-expansion-and-required-check-roadmap.md)
- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [GitHub Backlog Map](./github-backlog-map.md)
- [AI Collaboration And Agent Interop Plan](./ai-collaboration-and-agent-interop-plan.md)
- [Gemini PR Review Assistant](./gemini-pr-review-assistant.md)

## Related Design Specs

- [Design Reading Order](../../design/en/reading-order.md)
- [Glossary](../../design/en/glossary.md)
- [Implementation Readiness Checklist](../../design/en/implementation-readiness-checklist.md)
- [Architecture Principles](../../design/en/architecture-principles.md)
- [Internal Packet And Metadata Model](../../design/en/internal-packet-and-metadata-model.md)
- [Fault Model And Overload Behavior](../../design/en/fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](../../design/en/recovery-model-and-cache-semantics.md)
- [Compatibility Matrix](../../design/en/compatibility-matrix.md)
- [Route Configuration Grammar](../../design/en/route-configuration-grammar.md)
- [Benchmark Workload Definition](../../design/en/benchmark-workload-definition.md)
- [Route Rule Cookbook And Worked Examples](../../design/en/route-rule-cookbook-and-worked-examples.md)
- [Metrics And Telemetry Schema](../../design/en/metrics-and-telemetry-schema.md)
- [Benchmark Result Interpretation Guide](../../design/en/benchmark-result-interpretation-guide.md)
- [Plugin Boundary Note](../../design/en/plugin-boundary-note.md)
- [Security Overlay Model](../../design/en/security-overlay-model.md)
- [Operator Workflow And Recovery Playbook](../../design/en/operator-workflow-and-recovery-playbook.md)
- [Profile-Specific Operator Guides](../../design/en/profile-specific-operator-guides.md)
- [Transport And Adapter Contract](../../design/en/transport-and-adapter-contract.md)
- [Dashboard Information Architecture](../../design/en/dashboard-information-architecture.md)
- [Native IPC ABI Note](../../design/en/native-ipc-abi-note.md)
- [Federation And High-Availability Model](../../design/en/federation-and-high-availability-model.md)
- [Config Validation And Migration Note](../../design/en/config-validation-and-migration-note.md)
- [Discovery And Service Metadata Model](../../design/en/discovery-and-service-metadata-model.md)
- [Schema Definition Format](../../design/en/schema-definition-format.md)
- [C ABI Reference Header And Error-Code Catalog](../../design/en/c-abi-reference-header-and-error-code-catalog.md)
- [Deployment Topology And Release Profile Guide](../../design/en/deployment-topology-and-release-profile-guide.md)
- [Testing Strategy And Fuzz Corpus Plan](../../design/en/testing-strategy-and-fuzz-corpus-plan.md)
- [Rust Workspace And Crate Boundaries](../../design/en/rust-workspace-and-crate-boundaries.md)
- [OSC Conformance Corpus Plan](../../design/en/osc-conformance-corpus-plan.md)
- [Benchmark Fixture Inventory And Reproducibility Plan](../../design/en/benchmark-fixture-inventory-and-reproducibility-plan.md)
- [Adapter SDK API Reference](../../design/en/adapter-sdk-api-reference.md)
- [Conformance Vector And Interoperability Suite Guide](../../design/en/conformance-vector-and-interoperability-suite-guide.md)
- [Schema Evolution And Deprecation Policy](../../design/en/schema-evolution-and-deprecation-policy.md)
- [Dashboard Interaction Spec And Screen Inventory](../../design/en/dashboard-interaction-spec-and-screen-inventory.md)
- [Release Checklist And Operational Runbook](../../design/en/release-checklist-and-operational-runbook.md)
- [Architecture Decision Record Index](../../design/en/architecture-decision-record-index.md)
- [ADR Folder](../../design/adr/en/README.md)

Japanese versions:

- [Japanese Index](../ja/README.md)

## Suggested Delivery Strategy

The project should be developed in this order:

1. Make raw OSC routing correct and fast.
2. Make the system observable and recoverable under stress.
3. Expand protocol reach without breaking existing OSC users.
4. Add extensibility so advanced features do not bloat the core.
5. Add native integrations only after the broker core is operationally stable.
6. Add security and advanced synchronization as optional overlays.

## Compatibility Baseline To Protect

The implementation should always preserve these behaviors:

- OSC packets remain 4-byte aligned.
- All numeric fields remain big-endian as defined by OSC.
- UDP datagram mode remains a first-class transport.
- Existing 1.0 address pattern behavior remains available.
- Receivers remain robust to older messages that omit the type tag string.
- Unknown or unsupported extension types are handled defensively and never
  silently corrupt the stream.
