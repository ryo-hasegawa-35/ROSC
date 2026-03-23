# ADR-0010: Broker Identity, Federation, And Failover

- Status: accepted
- Date: 2026-03-23

## Context

Federated or standby deployments need explicit ownership and failover rules to
avoid split-brain behavior and ambiguous authority.

## Decision

- assign explicit broker identity
- define replication scope and ownership boundaries
- use active/standby semantics before considering more complex topologies
- require explicit failover authority and split-brain prevention

## Consequences

- operations stay understandable under fault conditions
- replication logic remains subordinate to declared scope
- multi-broker deployments must accept stricter coordination rules

## Rejected Alternatives

- implicit multi-master behavior
- discovery-only failover authority

## Affected Documents

- [Federation And High-Availability Model](../../en/federation-and-high-availability-model.md)
- [Profile-Specific Operator Guides](../../en/profile-specific-operator-guides.md)
- [Release Checklist And Operational Runbook](../../en/release-checklist-and-operational-runbook.md)
