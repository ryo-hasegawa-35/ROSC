# Dashboard Information Architecture

## Purpose

This document defines how the dashboard should organize information so operators
can understand the broker quickly under normal and abnormal conditions.

The dashboard is not decoration. It is part of the operating model.

## Primary Questions

The dashboard should answer these questions first:

- is the system healthy
- where is pressure building
- what is being dropped
- which routes or destinations are unhealthy
- what changed recently
- how do I recover safely

## Information Hierarchy

Top-level information order should be:

1. system health
2. active incidents
3. route and destination status
4. recent changes
5. deeper forensic tools

## Core Views

### Overview

Shows:

- global state
- packet rates
- queue pressure summary
- top warnings
- active breakers
- active quarantines

### Topology

Shows:

- ingress nodes
- routes
- transforms
- destinations
- adapter state

### Routes

Shows:

- route class
- mode
- match pattern
- queue depth
- drop count
- cache policy
- recovery actions

### Destinations

Shows:

- destination health
- transport type
- egress queue
- breaker state
- last error

### Traffic / Forensics

Shows:

- packet timeline
- filtered capture view
- replay session status
- correlation lookup

### Recovery

Shows:

- cache state
- rehydrate candidates
- last recovery actions
- safe mode state

### Security

Shows:

- active identities
- denied events
- scope mismatches
- secure route status

## Status Model

The dashboard should use the same top-level states as the fault model:

- `Healthy`
- `Pressured`
- `Degraded`
- `Emergency`
- `SafeMode`

## Entity Model

The dashboard should treat these as first-class entities:

- ingress
- adapter
- route
- transform
- destination
- packet capture session
- replay session
- cache namespace
- security scope

## Event Timeline

Operators need a single time-ordered event view for:

- config changes
- breaker events
- quarantine events
- replay actions
- rehydrate actions
- adapter disconnects
- safe mode entry

## Action Design Rules

Operator actions should be:

- visible
- reversible when possible
- scoped
- audited

High-impact actions:

- isolate route
- disable destination
- resend cached state
- start replay
- enter safe mode

These should require clear confirmation and scope display.

## Progressive Disclosure

The dashboard should not show every detail at once.

Recommended pattern:

- overview first
- drill into unhealthy items
- open forensic tools only when needed

## Data Freshness Rules

The dashboard should clearly distinguish:

- live values
- cached values
- stale values
- replay traffic

## Failure Visibility Rules

The dashboard must highlight:

- hidden queue growth risk
- pressure on critical routes
- adapter disconnect storms
- repeated transform failure
- stale cache that would affect recovery

## Non-Negotiable Invariants

- the dashboard must guide action, not merely display metrics
- health state must be visible before forensic detail
- replay and live traffic must be visually distinct
- operator actions must leave audit trails
- dashboard refresh must not destabilize the hot path
