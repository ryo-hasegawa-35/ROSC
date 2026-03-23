# Benchmark Result Interpretation Guide

## Purpose

This document explains how to read benchmark outcomes responsibly.

The workload definition says what to run. This document says how to decide
whether the result is actually good, risky, misleading, or release-blocking.

Related documents:

- [Benchmark Workload Definition](./benchmark-workload-definition.md)
- [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
- [Compatibility Matrix](./compatibility-matrix.md)
- [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

## Benchmarking Is About Trust, Not Only Speed

A faster result is not automatically a better result.

Interpretation should always ask:

- did compatibility remain correct
- did route isolation remain intact
- did overload behavior stay explicit
- did observability overhead remain acceptable
- did recovery behavior remain correct

## Required Context For Any Result

Every benchmark report should state:

- workload definition version
- software revision or document revision
- operating system
- CPU class
- memory class
- active feature toggles
- release profile
- route count and destination count
- whether the run was warm or cold

If this context is missing, the result is incomplete.

## Primary Decision Questions

### Question 1: Did The System Stay Correct?

Reject or investigate immediately if:

- packets are lost outside declared drop policy
- compatibility mode behavior changed unexpectedly
- malformed traffic causes crash, stall, or hidden corruption

### Question 2: Did The System Stay Predictable?

Watch:

- p95 and p99 latency growth
- jitter spread
- queue depth growth pattern
- breaker behavior under fault

A predictable degraded system is better than a fast chaotic one.

### Question 3: Did Isolation Hold?

The most important routing question is usually:

- did the unhealthy or slow path stay local

If a slow destination harms healthy critical paths, treat that as a major
regression even if headline throughput improved.

### Question 4: Did Recovery Stay Useful?

Check:

- rehydrate latency
- cache correctness
- restart recovery time
- replay safety boundaries

Fast but incorrect recovery is a failed result.

## How To Read Key Signals

### Throughput

Use throughput only as a secondary signal.

Higher packets per second is useful only if:

- compatibility stays correct
- tail latency stays acceptable
- drop reasons stay inside policy

### Tail Latency

p95 and p99 matter more than median for live show behavior.

Interpretation rule:

- median tells you what is common
- p95 and p99 tell you whether the pipe stays trustworthy

### Jitter

Jitter matters when timing consistency is more important than raw speed.

Treat widening jitter under mixed load as a warning even if average latency
looks fine.

### Queue Depth

Queue depth reveals future trouble before drops happen.

Interpretation rule:

- a stable shallow queue suggests headroom
- a steadily climbing queue suggests eventual instability

### Drop Count By Reason

Not all drops mean the same thing.

Differentiate:

- intentional sampling on disposable streams
- overload drops on critical routes
- malformed packet rejection
- security rejection

Critical-route drops should be treated as near-release-blocking unless
explicitly allowed by profile policy.

### Breaker Events

Breaker openings are not automatically failures.

Interpretation rule:

- a breaker opening on an optional analytics path may prove isolation works
- a breaker opening on a primary control path is a serious incident indicator

### CPU And Memory

High resource use is not automatically disqualifying, but it matters when:

- tail latency worsens
- diagnostics become part of the bottleneck
- profile targets include smaller machines

## How To Compare Runs Safely

Only compare runs when all of these are substantially aligned:

- workload version
- platform class
- feature toggles
- profile
- route count and destination count
- warm-up behavior

Recommended comparison method:

1. run the same workload multiple times
2. compare median and tail behavior across runs
3. investigate variance before claiming improvement

## Result Classes

### Pass With Headroom

Use when:

- correctness holds
- tail latency remains bounded
- isolation holds
- resource growth is acceptable

### Pass With Caution

Use when:

- correctness holds
- one or more resource or telemetry costs increased
- degradation remains explicit and bounded

### Investigate Before Claiming Improvement

Use when:

- headline throughput improved
- but tail latency, jitter, or queue growth worsened
- or variance between runs is high

### Fail

Use when:

- compatibility broke
- isolation broke
- crash or hidden corruption occurred
- recovery became incorrect

## Common Interpretation Mistakes

- celebrating throughput gains while ignoring tail latency
- comparing different workload versions as if they were equal
- forgetting that enabling metrics or capture changes the measurement target
- assuming sampled drops and critical drops have the same meaning
- hiding unstable variance behind one unusually good run

## Release Evidence Rule

A release claim should be backed by:

- the benchmark workload used
- the profile and toggles used
- the interpretation class
- known caveats

Benchmarks should support honest release notes, not marketing-only numbers.

## Non-Negotiable Rules

- No benchmark result may be presented without enough context to reproduce its
  meaning.
- Compatibility regressions outrank speed improvements.
- Isolation regressions outrank headline throughput gains.
- Tail behavior and recovery behavior are first-class release criteria.
