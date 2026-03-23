# Architecture Decision Record Index

## 目的

この文書は、architecture decision をどう記録し追跡するかを定義します。

すでに強い design note はありますが、将来の code change、repository setup、
operational tradeoff が、それらを静かに読み替えてしまわないように ADR の
decision trail を残します。

関連文書:

- [Architecture Principles](./architecture-principles.md)
- [Implementation Readiness Checklist](./implementation-readiness-checklist.md)
- [GitHub Foundation And Collaboration Plan](../../concepts/ja/github-foundation-and-collaboration-plan.md)

## ADR Policy

次のような decision には ADR があるべきです。

- 長期的な compatibility posture を変える
- packet や routing semantics を変える
- fault containment や recovery guarantee を変える
- telemetry meaning や evidence standard を変える
- plugin、adapter、IPC、distributed trust boundary を変える
- repository-wide な development discipline を長期にわたって変える

## 想定する ADR Status 値

- `proposed`
- `accepted`
- `superseded`
- `rejected`
- `withdrawn`

## 最低限の ADR Field

すべての ADR は次を含むべきです。

- ADR ID
- title
- status
- date
- context
- decision
- consequences
- rejected alternatives
- affected documents

## Storage Convention

ADR file は、言語ごとの mirrored tree に保存します。

- `docs/design/adr/en/`
- `docs/design/adr/ja/`

## 現在の Accepted ADR Set

- [ADR-0001 Compatibility Mode Contract](../adr/ja/adr-0001-compatibility-mode-contract.md)
- [ADR-0002 Dual Packet Representation](../adr/ja/adr-0002-dual-packet-representation.md)
- [ADR-0003 Route Semantic Model Before File Format](../adr/ja/adr-0003-route-semantic-model-before-file-format.md)
- [ADR-0004 Traffic Classes And Isolation Rules](../adr/ja/adr-0004-traffic-classes-and-isolation-rules.md)
- [ADR-0005 Recovery Contract](../adr/ja/adr-0005-recovery-contract.md)
- [ADR-0006 Telemetry Levels And Cardinality Budget](../adr/ja/adr-0006-telemetry-levels-and-cardinality-budget.md)
- [ADR-0007 Plugin Boundary And Trust Tiers](../adr/ja/adr-0007-plugin-boundary-and-trust-tiers.md)
- [ADR-0008 Security Overlay Is Additive](../adr/ja/adr-0008-security-overlay-is-additive.md)
- [ADR-0009 Native IPC ABI Stability And Fallback](../adr/ja/adr-0009-native-ipc-abi-stability-and-fallback.md)
- [ADR-0010 Broker Identity, Federation, And Failover](../adr/ja/adr-0010-broker-identity-federation-and-failover.md)
- [ADR-0011 Benchmark Gate And Release Evidence](../adr/ja/adr-0011-benchmark-gate-and-release-evidence.md)
- [ADR-0012 GitHub Protection And Docs-First Collaboration](../adr/ja/adr-0012-github-protection-and-docs-first-collaboration.md)
- [ADR-0013 Phase 00 Foundation Completion Gate](../adr/ja/adr-0013-phase-00-foundation-completion.md)

## 新しい ADR を作るタイミング

あとから contributor が古い commit、chat log、benchmark sheet を掘らないと
意図を復元できないような change は、ADR 化すべきです。

## Review Rule

normative な design document に触れ、その意味を大きく変える変更は、次の
どちらかであるべきです。

- accepted ADR を参照している
- 同じ planning window で proposed ADR を追加している

## Index Maintenance Rule

次のタイミングで index を更新します。

- 新しい ADR ID を予約した
- ADR の status が変わった
- 後続 ADR に supersede された

## Non-Negotiable Invariant

- ADR は design document を置き換えるものではなく、意図を明確にするもの
- Accepted ADR は docs tree から追える状態であるべき
- 実装品質を左右する repository process の decision も packet-format decision と同じ traceability を持つべき
