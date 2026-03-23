# ADR-0013: Phase 00 Foundation Completion Gate

- Status: accepted
- Date: 2026-03-23

## Context

"implementation を始めてよい状態" を explicit にしておかないと、
最初の code が architecture base を追い越してしまいます。

## Decision

- licensing、ADR、workspace planning、conformance planning、
  benchmark planning、CI scaffolding がそろった時点で Phase 00 完了とする
- 最初の implementation work は、その artifact が存在してから始める
- その後の Phase 00 change は blocker ではなく incremental governance
  improvement として扱う

## Consequences

- implementation readiness の共有定義ができる
- 将来の contributor が chat history を掘らずに readiness を確認できる
- repository / design readiness を committed artifact として監査しやすい

## Rejected Alternatives

- implicit verbal agreement で implementation を始めること
- readiness criteria を issue text だけに分散させること

## Affected Documents

- [Implementation Readiness Checklist](../../ja/implementation-readiness-checklist.md)
- [Detailed Delivery Plan](../../../concepts/ja/detailed-delivery-plan.md)
- [GitHub Foundation And Collaboration Plan](../../../concepts/ja/github-foundation-and-collaboration-plan.md)
