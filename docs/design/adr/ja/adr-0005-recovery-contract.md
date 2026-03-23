# ADR-0005: Recovery Contract

- Status: accepted
- Date: 2026-03-23

## Context

recovery はこの project の大きな差別化要素ですが、
unsafe な replay は packet loss より危険になりえます。

## Decision

- `rehydrate` と `replay` を分離する
- cache class を explicit に定義する
- automatic / manual recovery は route-level policy で決める
- trigger 的な traffic は default で non-automatic とする

## Consequences

- late joiner recovery を強くしつつ、すべての traffic を安全と偽らずに済む
- operator UI と log は recovery path を明確に区別する必要がある
- capture / replay tooling は route policy を尊重する必要がある

## Rejected Alternatives

- 直近 traffic の全面的な automatic replay
- address name だけで recoverability を推測すること

## Affected Documents

- [Recovery Model And Cache Semantics](../../ja/recovery-model-and-cache-semantics.md)
- [Operator Workflow And Recovery Playbook](../../ja/operator-workflow-and-recovery-playbook.md)
- [Route Rule Cookbook And Worked Examples](../../ja/route-rule-cookbook-and-worked-examples.md)
