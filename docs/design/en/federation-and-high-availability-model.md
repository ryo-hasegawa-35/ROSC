# Federation And High-Availability Model

## Purpose

This document defines how multiple brokers can cooperate for redundancy,
segmentation, and larger deployments.

## Design Goals

- support local installations first
- scale to multi-node systems without rewriting semantics
- distinguish federation from failover
- avoid split-brain ambiguity

## Two Distinct Models

### Federation

Use when:

- multiple brokers intentionally exchange selected traffic
- different network segments or sites must cooperate

### High Availability

Use when:

- one broker should continue service if another fails
- route continuity matters more than horizontal expansion

## Broker Identity

Each broker should have:

- stable broker ID
- instance ID
- deployment scope
- advertised capabilities

## Federation Link

A federation link should define:

- peer identity
- allowed scopes
- replicated routes or namespaces
- transport security mode
- health state

## Replication Scope

Not everything should replicate.

Possible replication classes:

- config only
- selected cache state
- selected journal state
- selected live traffic
- discovery metadata

## Active / Standby Model

Recommended first HA model:

- active broker
- standby broker
- replicated config
- replicated selected cache
- explicit failover trigger

## Failover Triggers

Possible triggers:

- active broker health loss
- transport loss beyond threshold
- operator action
- host process crash

## Split-Brain Prevention

The system should avoid two brokers claiming the same active role silently.

Recommended controls:

- explicit role state
- lease or heartbeat mechanism
- operator-visible arbitration state
- safe fail-closed behavior when authority is uncertain

## State Continuity

To make failover useful, the standby should have enough state to continue
safely.

Good candidates:

- route graph
- selected cache entries
- selected journals
- security scope state where appropriate

## Recovery After Failover

After failover:

- the new active broker should mark continuity transition clearly
- rehydrate rules should remain route-aware
- replay should not be triggered automatically unless configured

## Federation Security

Broker-to-broker traffic should preserve:

- peer identity
- authorized scope
- auditability

Federation should never act as an unbounded bypass around route security.

## Observability

Operators should be able to inspect:

- active and standby roles
- peer health
- replication lag
- failover history
- split-brain risk indicators

## Non-Negotiable Invariants

- federation and failover must be explicit, not accidental
- replicated state must preserve lineage and scope
- uncertain authority should not silently produce duplicate active brokers
- single-broker deployments must remain first-class and simple
