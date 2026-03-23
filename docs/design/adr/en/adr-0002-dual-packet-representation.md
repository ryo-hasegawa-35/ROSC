# ADR-0002: Dual Packet Representation

- Status: accepted
- Date: 2026-03-23

## Context

The broker needs byte-level fidelity for compatibility and forensics, while the
core routing engine needs a normalized internal view.

## Decision

- retain raw packet bytes and ingress metadata
- derive a normalized packet representation for routing and operator tooling
- keep parse failure information explicit instead of manufacturing partial
  normalized state
- treat unknown tags according to compatibility mode, never by silent mutation

## Consequences

- memory use is higher than a normalized-only design
- replay, audit, and conformance tooling become much stronger
- routing code can stay independent from byte-level parsing details

## Rejected Alternatives

- normalized-only storage
- raw-only opaque packet handling

## Affected Documents

- [Internal Packet And Metadata Model](../../en/internal-packet-and-metadata-model.md)
- [Compatibility Matrix](../../en/compatibility-matrix.md)
- [Recovery Model And Cache Semantics](../../en/recovery-model-and-cache-semantics.md)
