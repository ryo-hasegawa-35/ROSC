# ADR-0009: Native IPC ABI Stability And Fallback

- Status: accepted
- Date: 2026-03-23

## Context

Native IPC can reduce local latency, but it must never trap the project in a
host-specific or unstable integration story.

## Decision

- keep IPC acceleration optional
- maintain UDP fallback as a first-class path
- define a stable versioned C ABI before host-specific integrations
- make shared-memory ownership and lifecycle explicit

## Consequences

- UE5 and TouchDesigner integrations can evolve without redefining the core
- local acceleration remains available without becoming mandatory
- ABI changes require deliberate versioning discipline

## Rejected Alternatives

- mandatory shared-memory transport
- engine-specific native interfaces without a stable common ABI

## Affected Documents

- [Native IPC ABI Note](../../en/native-ipc-abi-note.md)
- [C ABI Reference Header And Error-Code Catalog](../../en/c-abi-reference-header-and-error-code-catalog.md)
- [Phase 05](../../../concepts/en/phase-05-native-integration.md)
