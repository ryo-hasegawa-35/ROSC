# ADR-0006: Telemetry Levels And Cardinality Budget

- Status: accepted
- Date: 2026-03-23

## Context

operator visibility は必要ですが、unbounded telemetry はそれ自体が
stability problem になりえます。

## Decision

- `metrics_level` を explicit に定義する
- canonical metric name を stable に保つ
- label cardinality を bounded に制御する
- 高詳細 diagnostics は baseline health metric と分けて扱う

## Consequences

- dashboard と alert が stable な metric identity に依存できる
- detailed capture は維持しつつ time-series cost を爆発させにくい
- high-cardinality proposal は個別 justification が必要になる

## Rejected Alternatives

- per-message metrics label
- stable metric name を持たない logs-only observability

## Affected Documents

- [Metrics And Telemetry Schema](../../ja/metrics-and-telemetry-schema.md)
- [Dashboard Information Architecture](../../ja/dashboard-information-architecture.md)
- [Benchmark Result Interpretation Guide](../../ja/benchmark-result-interpretation-guide.md)
