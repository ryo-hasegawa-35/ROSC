# ADR-0006: Telemetry Levels And Cardinality Budget

- Status: accepted
- Date: 2026-03-23

## Context

Operator visibility is required, but unbounded telemetry can become its own
stability problem.

## Decision

- define `metrics_level` explicitly
- keep canonical metric names stable
- enforce bounded label cardinality
- keep high-detail diagnostics separate from baseline health metrics

## Consequences

- dashboards and alerts can depend on stable metric identity
- detailed capture remains possible without exploding time-series cost
- developers must justify any high-cardinality proposal

## Rejected Alternatives

- per-message metrics labels
- logs-only observability without stable metric names

## Affected Documents

- [Metrics And Telemetry Schema](../../en/metrics-and-telemetry-schema.md)
- [Dashboard Information Architecture](../../en/dashboard-information-architecture.md)
- [Benchmark Result Interpretation Guide](../../en/benchmark-result-interpretation-guide.md)
