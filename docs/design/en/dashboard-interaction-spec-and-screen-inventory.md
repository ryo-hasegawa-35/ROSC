# Dashboard Interaction Spec And Screen Inventory

## Purpose

This document extends the dashboard information architecture by defining the
interaction expectations and the main screen inventory.

If the information architecture answers "what information belongs where," this
document answers "how operators should move through it."

## Design Goals

- support fast incident handling
- minimize operator confusion under pressure
- keep high-impact actions explicit
- make navigation reflect the operational model

## Screen Inventory

### Screen 1: Overview

Primary use:

- instant system status

Must show:

- current global state
- top incidents
- packet throughput summary
- active breaker and quarantine counts
- quick links to unhealthy entities

### Screen 2: Topology

Primary use:

- understanding system structure

Must show:

- ingress nodes
- routes
- transforms
- destinations
- adapter relationships

### Screen 3: Route Detail

Primary use:

- route-level diagnosis and action

Must show:

- match definition
- mode and class
- queue state
- fault policy
- cache / recovery policy
- destination fan-out

Actions:

- isolate route
- resend cached state
- inspect recent history

### Screen 4: Destination Detail

Primary use:

- diagnose destination-specific issues

Must show:

- transport and adapter
- current health
- queue pressure
- breaker state
- recent errors

Actions:

- disable destination
- inspect retries or failures

### Screen 5: Traffic / Forensics

Primary use:

- traffic inspection and replay setup

Must show:

- filtered timeline
- packet detail
- lineage markers
- capture session controls

Actions:

- create filtered capture
- open sandbox replay

### Screen 6: Recovery

Primary use:

- controlled restoration and cache management

Must show:

- cache namespaces
- stale warnings
- rehydrate candidates
- recent recovery actions

Actions:

- rehydrate route
- rehydrate namespace snapshot
- invalidate stale cache

### Screen 7: Security

Primary use:

- trust and access visibility

Must show:

- active identities
- denied events
- scope mismatches
- secure route status

### Screen 8: Config / Change Review

Primary use:

- review and apply configuration safely

Must show:

- pending diff
- validation findings
- risk summary
- rollback target

## Navigation Model

Recommended primary navigation:

- Overview
- Topology
- Routes
- Destinations
- Traffic
- Recovery
- Security
- Config

## High-Impact Interaction Rules

The UI should require explicit confirmation for:

- isolate route
- disable destination
- resend state
- enter safe mode
- start replay
- apply risky config change

## Replay Interaction Rules

Replay UI should make clear:

- replay target
- replay scope
- replay lineage marking
- whether replay is sandboxed

## Change Review Interaction Rules

Config apply flow should show:

- what changed
- what validation said
- what risks are present
- what rollback path exists

## Status Consistency

All screens should use the same top-level state vocabulary:

- `Healthy`
- `Pressured`
- `Degraded`
- `Emergency`
- `SafeMode`

## Non-Negotiable Invariants

- the dashboard must remain action-oriented
- dangerous actions must be obvious and confirmable
- replay and live operations must never look identical
- operators must always know what scope an action affects
