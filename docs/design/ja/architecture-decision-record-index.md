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

ADR file を追加する場合は、言語ごとに mirrored tree へ置きます。

- `docs/design/adr/en/`
- `docs/design/adr/ja/`

index は individual ADR file より先に存在していて構いません。

## 初期 ADR Backlog

初期実装の前後で、まず formal ADR にすべき decision は次です。

### ADR-0001: Compatibility Mode Contract

Scope:

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`
- legacy missing-type-tag policy

### ADR-0002: Dual Packet Representation

Scope:

- raw byte retention
- normalized internal view
- parse failure と unknown-tag behavior

### ADR-0003: Route Semantic Model Before File Format

Scope:

- semantic route field
- first external format としての TOML
- apply 前 validation

### ADR-0004: Traffic Classes And Isolation Rules

Scope:

- traffic class vocabulary
- per-destination isolation
- breaker と quarantine expectation

### ADR-0005: Recovery Contract

Scope:

- rehydrate と replay の分離
- cache class
- automatic / manual recovery boundary

### ADR-0006: Telemetry Levels And Cardinality Budget

Scope:

- `metrics_level` semantics
- canonical metric name
- bounded-label policy

### ADR-0007: Plugin Boundary And Trust Tiers

Scope:

- plugin trust tier
- Wasm と external process の boundary
- broker-owned safety semantics

### ADR-0008: Security Overlay Is Additive

Scope:

- broker edge での source verification
- legacy bridge の扱い
- raw OSC backward compatibility

### ADR-0009: Native IPC ABI Stability And Fallback

Scope:

- IPC acceleration は optional のまま
- UDP fallback を first-class に保つ
- ABI versioning expectation

### ADR-0010: Broker Identity, Federation, And Failover

Scope:

- broker identity
- replication scope
- split-brain prevention
- failover authority

### ADR-0011: Benchmark Gate And Release Evidence

Scope:

- mandatory benchmark context
- interpretation class
- release evidence requirement

### ADR-0012: GitHub Protection And Docs-First Collaboration

Scope:

- protected branch baseline
- review expectation
- risky code 前の documentation gate

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
