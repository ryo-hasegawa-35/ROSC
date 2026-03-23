# Phase 00: Foundation

## Goal

Create the specification baseline, repository structure, tests, benchmark
harness, and compatibility rules that every later phase depends on.

## Deliverables

- Workspace layout for core, adapters, dashboard, plugins, and integration SDKs
- Parser and encoder test vectors based on OSC 1.0 examples
- Bundle, timetag, and address-pattern conformance tests
- Compatibility matrix for:
  - OSC 1.0 strict mode
  - legacy missing-type-tag tolerance
  - OSC 1.1-inspired optional behavior
- Baseline benchmark harness for:
  - packet parse throughput
  - routing latency
  - egress fan-out
  - burst traffic behavior
- Fuzzing targets for malformed packets, nested bundles, and invalid type tags
- Cross-platform CI skeleton for Windows, macOS, and Linux

## Must Decide In This Phase

- Internal event representation
- Route configuration format
- Error handling policy
- Memory ownership model
- How strict vs tolerant parsing modes are selected
- Whether the dashboard is embedded or separate during development

## Non-Goals

- No production dashboard yet
- No schema system yet
- No shared memory IPC yet
- No zero-trust security yet

## Exit Criteria

- We can parse and re-emit canonical OSC 1.0 packets without changing bytes
  except where normalization is explicitly intended.
- We have tests covering message, bundle, timetag, and pattern matching basics.
- We have a repeatable benchmark script and baseline numbers.
- CI proves the crate layout builds on all three desktop platforms.

## Rough Effort

40-80 hours

## Why This Phase Matters

If this phase is weak, later features will stack on top of vague assumptions and
backward compatibility will drift. This phase is where "correctness" becomes a
project asset instead of a hope.
