# ADR-0007: Plugin Boundary And Trust Tiers

- Status: accepted
- Date: 2026-03-23

## Context

The platform needs extensibility, but unrestricted plugin access would weaken
the broker’s safety and determinism.

## Decision

- define trust tiers for built-in, Wasm, and external-process extensions
- keep broker-owned safety semantics outside plugin control
- expose stable plugin and adapter boundaries rather than unrestricted internals
- prefer containment over convenience

## Consequences

- extensibility remains possible without turning the broker into a shared heap
- plugin failure can be isolated more cleanly
- some integrations will need explicit SDK support instead of internal access

## Rejected Alternatives

- unrestricted in-process plugin access
- allowing plugins to own security or routing invariants

## Affected Documents

- [Plugin Boundary Note](../../en/plugin-boundary-note.md)
- [Adapter SDK API Reference](../../en/adapter-sdk-api-reference.md)
- [Security Overlay Model](../../en/security-overlay-model.md)
