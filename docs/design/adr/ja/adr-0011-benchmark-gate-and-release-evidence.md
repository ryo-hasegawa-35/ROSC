# ADR-0011: Benchmark Gate And Release Evidence

- Status: accepted
- Date: 2026-03-23

## Context

project は speed、jitter、stability、recovery について強い主張を目指します。
その主張は reproducible な evidence と結び付いていなければいけません。

## Decision

- benchmark claim には named workload と reproducible fixture input を要求する
- benchmark context を explicit に記録する
- benchmark interpretation を release evidence の一部として扱う
- workload class を示さない headline performance claim を禁止する

## Consequences

- release note が anecdote ではなく evidence にリンクできる
- benchmark automation には stable な fixture inventory が必要になる
- regression を検出・説明しやすくなる

## Rejected Alternatives

- ad hoc local benchmark screenshot
- workload context のない release claim

## Affected Documents

- [Benchmark Workload Definition](../../ja/benchmark-workload-definition.md)
- [Benchmark Result Interpretation Guide](../../ja/benchmark-result-interpretation-guide.md)
- [Testing Strategy And Fuzz Corpus Plan](../../ja/testing-strategy-and-fuzz-corpus-plan.md)
