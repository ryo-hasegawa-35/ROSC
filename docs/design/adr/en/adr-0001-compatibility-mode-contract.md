# ADR-0001: Compatibility Mode Contract

- Status: accepted
- Date: 2026-03-23

## Context

The broker must preserve existing OSC 1.0 behavior while still making room for
legacy tolerance and 1.1-oriented extensions.

## Decision

- support `osc1_0_strict`, `osc1_0_legacy_tolerant`, and `osc1_1_extended`
- keep `osc1_0_strict` as the baseline interoperability contract
- allow missing type-tag tolerance only in the explicit legacy mode
- treat 1.1-oriented extensions as additive, never silently mandatory

## Consequences

- compatibility behavior becomes testable and explicit
- parser and encoder work must always name the mode they serve
- egress behavior can reject unsafe down-conversion rather than guess

## Rejected Alternatives

- a single permissive parser mode
- treating 1.1-style behavior as the default for all peers

## Affected Documents

- [Compatibility Matrix](../../en/compatibility-matrix.md)
- [Architecture Principles](../../en/architecture-principles.md)
- [Conformance Vector And Interoperability Suite Guide](../../en/conformance-vector-and-interoperability-suite-guide.md)
