# ADR-0011: Benchmark Gate And Release Evidence

- Status: accepted
- Date: 2026-03-23

## Context

The project makes strong claims about speed, jitter, stability, and recovery.
Those claims must be tied to reproducible evidence.

## Decision

- require named workloads and reproducible fixture inputs for benchmark claims
- record benchmark context explicitly
- treat benchmark interpretation as part of release evidence
- forbid headline performance claims without stated workload class

## Consequences

- release notes can link to evidence instead of anecdotes
- benchmark automation needs stable fixture inventories
- regressions become easier to detect and explain

## Rejected Alternatives

- ad hoc local benchmark screenshots
- release claims without workload context

## Affected Documents

- [Benchmark Workload Definition](../../en/benchmark-workload-definition.md)
- [Benchmark Result Interpretation Guide](../../en/benchmark-result-interpretation-guide.md)
- [Testing Strategy And Fuzz Corpus Plan](../../en/testing-strategy-and-fuzz-corpus-plan.md)
