# Design Reading Order

## 目的

この文書は、設計書セットを効率よく読む順番を示します。

設計フォルダは、もはや小さなメモ集ではなく、system architecture の
参照体系になりつつあります。読む順番がないと、将来の実装時に同じ前提を
何度も掘り返すことになります。

## 推奨 Reading Path

## Path A: 最短の全体把握

最短で構造を理解したいなら、まず次を読んでください。

1. [Architecture Principles](./architecture-principles.md)
2. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
3. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
4. [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
5. [Compatibility Matrix](./compatibility-matrix.md)

この path で分かること:

- 何を守るシステムなのか
- packet をどう表現するのか
- failure をどう閉じ込めるのか
- state をどう戻すのか
- compatibility をどう守るのか

## Path B: Routing と Runtime Behavior

broker core に触る前なら、次の順が安全です。

1. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
2. [Compatibility Matrix](./compatibility-matrix.md)
3. [Route Configuration Grammar](./route-configuration-grammar.md)
4. [Route Rule Cookbook And Worked Examples](./route-rule-cookbook-and-worked-examples.md)
5. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
6. [Rust Workspace And Crate Boundaries](./rust-workspace-and-crate-boundaries.md)
7. [OSC Conformance Corpus Plan](./osc-conformance-corpus-plan.md)
8. [Benchmark Workload Definition](./benchmark-workload-definition.md)

この path で分かること:

- hot path が何を見るか
- route configuration が何を表現すべきか
- route authoring を実際にどう書くべきか
- overload でどう振る舞うべきか
- performance をどう測るべきか

## Path C: Operations と Trust

observability、recovery、operator UX に関わるなら次です。

1. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
2. [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
3. [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
4. [Dashboard Information Architecture](./dashboard-information-architecture.md)
5. [Operator Workflow And Recovery Playbook](./operator-workflow-and-recovery-playbook.md)
6. [Profile-Specific Operator Guides](./profile-specific-operator-guides.md)
7. [Config Validation And Migration Note](./config-validation-and-migration-note.md)

この path で分かること:

- どんな unhealthy state があるか
- operator に何を見せ、何を alert すべきか
- operator がどう復旧するか
- state をどう安全に見せるか
- config change をどう信用できるものにするか

## Path D: Extensibility と Integration

plugin、adapter、host integration に触るなら次です。

1. [Plugin Boundary Note](./plugin-boundary-note.md)
2. [Transport And Adapter Contract](./transport-and-adapter-contract.md)
3. [Security Overlay Model](./security-overlay-model.md)
4. [Native IPC ABI Note](./native-ipc-abi-note.md)
5. [Federation And High-Availability Model](./federation-and-high-availability-model.md)

この path で分かること:

- どこまで拡張してよいか
- 何を安定境界として守るか
- security と identity をどこで扱うか
- local native acceleration をどう収めるか
- multi-broker system をどう設計するか

## Path E: Discovery、Packaging、Verification

release engineering、schema tooling、delivery quality に触るなら次です。

1. [Discovery And Service Metadata Model](./discovery-and-service-metadata-model.md)
2. [Schema Definition Format](./schema-definition-format.md)
3. [Deployment Topology And Release Profile Guide](./deployment-topology-and-release-profile-guide.md)
4. [Testing Strategy And Fuzz Corpus Plan](./testing-strategy-and-fuzz-corpus-plan.md)
5. [OSC Conformance Corpus Plan](./osc-conformance-corpus-plan.md)
6. [Benchmark Fixture Inventory And Reproducibility Plan](./benchmark-fixture-inventory-and-reproducibility-plan.md)
7. [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)
8. [Config Validation And Migration Note](./config-validation-and-migration-note.md)

この path で分かること:

- service をどう見つけるか
- typed tooling をどう育てるか
- product shape を deployment にどう落とすか
- implementation quality への trust をどう作るか

## Path F: Delivery Discipline と External Integration

SDK、release process、interoperability evidence に触るなら次です。

1. [Adapter SDK API Reference](./adapter-sdk-api-reference.md)
2. [Conformance Vector And Interoperability Suite Guide](./conformance-vector-and-interoperability-suite-guide.md)
3. [Schema Evolution And Deprecation Policy](./schema-evolution-and-deprecation-policy.md)
4. [Dashboard Interaction Spec And Screen Inventory](./dashboard-interaction-spec-and-screen-inventory.md)
5. [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

この path で分かること:

- external adapter author がどう統合すべきか
- compatibility claim をどう証拠化するか
- schema change をどう安全に保つか
- dashboard 構造を operator interaction へどう落とすか
- release を buildable だけでなく operable にする方法

## Path G: Evidence、Telemetry、Decision Discipline

cross-cutting な変更や production readiness の判断に触るなら次です。

1. [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
2. [Benchmark Workload Definition](./benchmark-workload-definition.md)
3. [Benchmark Fixture Inventory And Reproducibility Plan](./benchmark-fixture-inventory-and-reproducibility-plan.md)
4. [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)
5. [Architecture Decision Record Index](./architecture-decision-record-index.md)
6. [ADR Folder](../adr/ja/README.md)
7. [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

この path で分かること:

- 何を evidence として出すべきか
- performance claim をどう読むべきか
- major decision をどう traceable に保つか
- release confidence をどう正当化するか

## 3 本だけ読むなら

まず次を読んでください。

1. [Architecture Principles](./architecture-principles.md)
2. [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
3. [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)

この 3 本が最も強い architectural constraint を持っています。

## Topic ごとの実装順

後で実装に入るなら、最も安全な順番は次です。

1. packet model
2. compatibility contract
3. route grammar
4. fault behavior
5. recovery behavior
6. benchmark harness expectation
7. adapter contract
8. operator-facing system
9. native / distributed extension

## Normative として扱うべき文書

次は casual note ではなく architecture-level constraint として扱うべきです。

- architecture principles
- internal packet and metadata model
- compatibility matrix
- fault model and overload behavior
- recovery model and cache semantics
- route configuration grammar

## より運用寄りで進化しやすい文書

次は normative 文書に従う必要がありますが、product shape に合わせて比較的
速く変わりえます。

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

## Cross-Folder の Planning Companion

repository rule の整備や大きな implementation を始める前には、次も読むべきです。

- [GitHub Foundation And Collaboration Plan](../../concepts/ja/github-foundation-and-collaboration-plan.md)

## 将来の Contributor 向け Reading Rule

core behavior を変える前に、その変更が次に触れていないか確認すべきです。

- compatibility
- packet semantics
- fault containment
- recovery semantics
- operator trust

もし触れているなら、coding 前に relevant な normative document を読み直す
べきです。
