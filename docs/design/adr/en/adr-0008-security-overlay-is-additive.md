# ADR-0008: Security Overlay Is Additive

- Status: accepted
- Date: 2026-03-23

## Context

The project needs stronger safety on hostile or noisy networks without breaking
raw OSC interoperability.

## Decision

- enforce source verification and project scope at the broker edge
- keep raw OSC payload semantics unchanged
- make legacy bridging explicit and observable
- treat security as an overlay, not a mutation of the base OSC format

## Consequences

- existing OSC peers can still interoperate through explicit broker policy
- security behavior becomes visible rather than hidden inside payload hacks
- operators must reason about verified versus bridged paths separately

## Rejected Alternatives

- embedding authentication inside raw OSC payloads
- requiring a security handshake for every legacy OSC peer

## Affected Documents

- [Security Overlay Model](../../en/security-overlay-model.md)
- [Phase 06](../../../concepts/en/phase-06-security-sync-release.md)
- [Transport And Adapter Contract](../../en/transport-and-adapter-contract.md)
