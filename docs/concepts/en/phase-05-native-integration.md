# Phase 05: Native Integration

## Goal

Bypass UDP for local high-performance workflows through shared memory IPC and
host-native integrations.

## Deliverables

- C ABI wrapper around the Rust core
- Shared memory transport for local machine communication
- Lock-free or low-contention ring buffer design for IPC
- UE5 native plugin
- TouchDesigner native bridge strategy
- Fallback path to standard OSC when native transport is unavailable
- Validation tooling for local latency and jitter measurement

## Design Constraints

- Native integration must never be the only supported path.
- The shared memory path must preserve the same logical routing semantics as the
  network path.
- Failure of native integration must degrade gracefully to standard OSC or local
  IPC alternatives.

## Suggested Rollout

1. Shared memory proof of concept
2. C ABI stabilization
3. UE5 plugin integration
4. TouchDesigner integration
5. Performance and reliability soak tests

## Non-Goals

- No removal of UDP code paths
- No assumption that all users can install native plugins
- No cross-process memory tricks without crash recovery planning

## Exit Criteria

- The same project can run in standard OSC mode or native IPC mode.
- UE5 can exchange messages with materially lower local jitter than UDP.
- Native plugins survive restart, reconnection, and fallback scenarios.

## Rough Effort

220-400 hours

## Value

This phase is where the project becomes genuinely different from ordinary OSC
routers, but it is also where platform-specific maintenance starts to matter.
