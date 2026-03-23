# Native IPC ABI Note

## Purpose

This document defines the boundary for native local-process integration,
especially the shared-memory and C ABI path used for UE5 and other host-native
integration.

## Design Goals

- stable host-facing ABI
- clear ownership rules
- no hidden dependence on Rust internal layout
- graceful fallback to standard transports
- cross-platform viability

## Layered Boundary

The native integration path should be split into two layers:

### Layer 1: Stable C ABI

Use for:

- lifecycle
- configuration calls
- capability query
- error reporting
- handle management

### Layer 2: IPC Data Path

Use for:

- high-throughput local message movement
- shared memory ring buffer or equivalent channel

## ABI Rules

- never expose Rust-specific memory layout directly
- use opaque handles
- use explicit size fields
- use explicit version fields
- keep ownership transfer rules simple

## Core Handle Types

Suggested opaque handles:

- broker handle
- route handle
- endpoint handle
- shared memory channel handle

## Versioning

The ABI should define:

- ABI version
- minimum compatible broker core version
- feature bitset

Version mismatch should fail clearly, not crash.

## Data Path Contract

The IPC data path should define:

- packet frame header
- payload region
- producer / consumer ownership
- sequence numbering
- overflow signaling
- health counters

## Shared Memory Channel Model

Recommended conceptual model:

- one or more ring buffers
- explicit producer and consumer roles
- bounded capacity
- separate control channel for lifecycle and diagnostics

## Memory Ownership

Rules:

- shared memory payload ownership is explicit
- host must not keep borrowing pointers after release
- broker must not assume host-side lifetime beyond contract
- error paths must still release resources safely

## Session Lifecycle

Typical lifecycle:

1. query ABI version
2. create broker handle
3. negotiate capabilities
4. create IPC channels
5. start transport loop
6. stop and destroy handles

## Fallback Policy

If native IPC is unavailable or unhealthy:

- fall back to localhost OSC or other supported local transport
- preserve logical route behavior
- report degraded state clearly

## Error Model

Errors should be represented as:

- explicit error code
- human-readable message where useful
- recoverable or unrecoverable classification

## Security And Scope

Even local IPC should preserve:

- project scope
- route authorization
- clear identity of the local host integration

## Observability

The ABI path should surface:

- channel occupancy
- overflow count
- reconnect count
- fallback transitions
- local latency metrics

## Non-Negotiable Invariants

- native IPC must never become the only usable path
- ABI stability matters more than exposing every internal feature
- shared memory failure must not corrupt broker state
- fallback must preserve compatibility expectations
