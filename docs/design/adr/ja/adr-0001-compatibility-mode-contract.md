# ADR-0001: Compatibility Mode Contract

- Status: accepted
- Date: 2026-03-23

## Context

broker は既存 OSC 1.0 behavior を守りつつ、legacy tolerance と
1.1 指向の extension を明示的に扱う必要があります。

## Decision

- `osc1_0_strict`、`osc1_0_legacy_tolerant`、`osc1_1_extended` をサポートする
- 基準となる interoperability contract は `osc1_0_strict` とする
- missing type-tag の許容は explicit な legacy mode にだけ認める
- 1.1 指向の extension は additive とし、黙って mandatory にしない

## Consequences

- compatibility behavior が explicit で testable になる
- parser / encoder は必ず対象 mode を名指しする必要がある
- egress は unsafe な down-conversion を推測せず reject できる

## Rejected Alternatives

- 単一の permissive parser mode
- すべての peer に 1.1-style behavior を既定適用すること

## Affected Documents

- [Compatibility Matrix](../../ja/compatibility-matrix.md)
- [Architecture Principles](../../ja/architecture-principles.md)
- [Conformance Vector And Interoperability Suite Guide](../../ja/conformance-vector-and-interoperability-suite-guide.md)
