# Recovery Model And Cache Semantics

## Purpose

This document defines how the broker restores continuity after restart,
disconnect, partial failure, or operator intervention.

The goal is not just "replay packets." The goal is to restore correct state
safely and quickly.

## Recovery Principles

- Recovery must be explicit, not magical.
- Recovery policy should be opt-in per route or namespace.
- Recovery should prefer correctness over aggressiveness.
- Replay and rehydration are different tools and must stay separate.
- Trigger-like traffic should never be resent automatically by accident.

## Recovery Layers

### Layer 1: Config Recovery

Restore:

- route graph
- transport bindings
- adapter settings
- security scopes
- cache policy

This should happen before dynamic runtime rehydration.

### Layer 2: Transport Recovery

Restore:

- listening sockets
- adapter connections
- broker identity and service metadata

This brings the broker back to a receiving and sending state.

### Layer 3: State Recovery

Restore:

- selected cached values
- selected journals
- route-local working state where explicitly supported

### Layer 4: Incident Recovery

Support:

- replay from capture
- resend cached state
- route-by-route recovery
- safe fallback to minimal mode

## Cacheable State Classes

Not every packet stream should be cached the same way.

### `NoCache`

Use for:

- impulses
- one-shot triggers
- unsafe or ambiguous traffic

### `LastValuePerAddress`

Use for:

- continuous control values
- scalar state
- parameters where the latest value fully defines current state

### `LastValuePerKey`

Use for:

- messages where a logical object key is embedded in the address or arguments
- per-tracker, per-light, or per-device state

### `SnapshotSet`

Use for:

- a bounded set of addresses that together define a scene or state snapshot

### `JournalWindow`

Use for:

- short recent history
- debugging
- carefully controlled catch-up behavior

### `DurableJournal`

Use for:

- selected critical streams where persistence across broker restart matters

## Safe Defaults For Cacheability

By default, a route should be considered cacheable only if:

- it represents state rather than an action
- replaying the latest value is idempotent or close to idempotent
- values are safely inspectable in the internal model
- the namespace owner has not marked the route as `NoCache`

By default, a route should not be auto-replayed if:

- it is trigger-like
- it is security-sensitive
- it carries opaque legacy payloads without safe semantics
- it controls destructive or irreversible actions

## Rehydrate vs Replay

These must be distinct in product language and implementation.

### Rehydrate

- restores current state
- usually based on cache or snapshot
- typically sends the latest known value only
- intended for restart recovery and late joiners

### Replay

- re-emits historical traffic
- may preserve timing relationships
- intended for debugging, testing, or controlled reconstruction
- should default to sandbox mode

## Cache Keys

Cache keys should be explicit and configurable.

Possible key components:

- namespace
- address
- source identity
- route identifier
- extracted logical key

Examples:

- last value for `/ue5/camera/fov`
- last value per performer ID
- last value per lighting fixture ID

## Freshness And Expiry

Caches must have freshness rules.

Recommended controls:

- max age
- namespace TTL
- explicit invalidation message or control action
- restart persistence policy

Rules:

- expired cache entries must not be used for automatic rehydrate
- stale but retained entries may still be available for manual operator review

## Persistence Levels

Different state should persist to different depths.

### Ephemeral

- memory only
- lost on broker restart

### Warm

- restored on broker restart from local snapshot
- suitable for selected last-value caches and config

### Durable

- persisted intentionally
- suitable for selected journals or critical state snapshots

## Recovery Triggers

Recovery may be initiated by:

- downstream node reconnect
- adapter reconnect
- broker restart
- operator action
- failover to standby broker

Each trigger should map to a policy, not an ad hoc resend flood.

## Recovery Order

Recommended order:

1. start minimal broker core
2. restore config and route graph
3. restore transports and adapters
4. restore security and policy state
5. restore selected warm caches
6. allow route-specific rehydrate
7. enable optional plugins and advanced behaviors

This order reduces the chance of replaying state into an unready graph.

## Late Joiner Semantics

Late joiner recovery should be explicit per namespace or route.

Useful policies:

- no automatic sync
- send latest value on subscribe or connect
- send bounded snapshot set
- wait for operator-triggered rehydrate

## Warm Restart

Warm restart should aim to preserve continuity without pretending nothing
happened.

Recommended warm restart restoration:

- config
- route graph
- selected caches
- selected adapter sessions where practical

Do not restore blindly:

- transient breaker states unless configured
- replay sessions
- unsafe trigger streams

## Active / Standby Continuity

If a standby broker exists, it should not guess state from live traffic alone.

Recommended strategy:

- replicate config changes
- replicate selected cache state
- replicate selected journals
- preserve clear lineage so standby-originated traffic is identifiable

## Security And Recovery

Recovery must respect security scope.

Rules:

- cache entries retain their security scope
- secure namespaces do not leak into insecure rehydrate paths
- operator-triggered resend across scopes requires explicit authorization

## Operator Experience

Useful recovery operations:

- resend latest cached state for route
- resend snapshot set for namespace
- replay captured window in sandbox
- compare current live state to cached state
- invalidate stale cache
- force route into no-rehydrate mode

## Metrics And Audit

The broker should surface:

- cache entry count
- cache hit count
- rehydrate events
- replay events
- stale entry count
- invalidation events
- durable journal usage
- restore time after restart

## Non-Negotiable Invariants

- automatic recovery must never silently resend dangerous trigger traffic
- replay and rehydrate must stay distinct
- stale state must be visible as stale
- recovery policy must be route-aware, not global by accident
- durable recovery must remain opt-in

## Follow-On Documents

This model should align directly with:

- internal packet and metadata model
- fault model and overload behavior
- security overlay model
- route configuration grammar
