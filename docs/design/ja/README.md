# ROSC 設計仕様

このフォルダには、broker core、configuration semantics、observability、
recovery、extensibility boundary、delivery discipline、そして将来の設計判断の
運用まで含めた技術仕様をまとめています。

構想、ロードマップ、GitHub 準備方針は
[Concepts / Planning](../../concepts/ja/README.md) にあります。

## 最初に見る入口

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
- [Conformance Vector And Interoperability Suite Guide](./conformance-vector-and-interoperability-suite-guide.md)
- [Architecture Decision Record Index](./architecture-decision-record-index.md)

## 関連する計画文書

- [Detailed Delivery Plan](../../concepts/ja/detailed-delivery-plan.md)
- [GitHub Foundation And Collaboration Plan](../../concepts/ja/github-foundation-and-collaboration-plan.md)

## 言語版と参考資料

- [Design Specs (English)](../en/README.md)
- [OSC 1.0 Specification PDF](../../references/osc-1.0-specification.pdf)
- [OSC 1.1 NIME 2009 PDF](../../references/osc-1.1-nime-2009.pdf)
