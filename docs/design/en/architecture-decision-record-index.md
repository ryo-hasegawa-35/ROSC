# Architecture Decision Record Index

## Purpose

This document defines how architecture decisions should be recorded and tracked.

The project already has strong design notes. ADRs exist so future code changes,
repository setup, and operational tradeoffs do not quietly reinterpret those
notes without leaving a decision trail.

Related documents:

- [Architecture Principles](./architecture-principles.md)
- [Implementation Readiness Checklist](./implementation-readiness-checklist.md)
- [GitHub Foundation And Collaboration Plan](../../concepts/en/github-foundation-and-collaboration-plan.md)

## ADR Policy

An ADR should exist when a decision:

- changes long-term compatibility posture
- changes packet or routing semantics
- changes fault containment or recovery guarantees
- changes telemetry meaning or evidence standards
- changes plugin, adapter, IPC, or distributed trust boundaries
- changes repository-wide development discipline in a lasting way

## Expected ADR Status Values

- `proposed`
- `accepted`
- `superseded`
- `rejected`
- `withdrawn`

## Minimum ADR Fields

Every ADR should include:

- ADR ID
- title
- status
- date
- context
- decision
- consequences
- rejected alternatives
- affected documents

## Storage Convention

ADR files live in mirrored language trees:

- `docs/design/adr/en/`
- `docs/design/adr/ja/`

## Current Accepted ADR Set

- [ADR-0001 Compatibility Mode Contract](../adr/en/adr-0001-compatibility-mode-contract.md)
- [ADR-0002 Dual Packet Representation](../adr/en/adr-0002-dual-packet-representation.md)
- [ADR-0003 Route Semantic Model Before File Format](../adr/en/adr-0003-route-semantic-model-before-file-format.md)
- [ADR-0004 Traffic Classes And Isolation Rules](../adr/en/adr-0004-traffic-classes-and-isolation-rules.md)
- [ADR-0005 Recovery Contract](../adr/en/adr-0005-recovery-contract.md)
- [ADR-0006 Telemetry Levels And Cardinality Budget](../adr/en/adr-0006-telemetry-levels-and-cardinality-budget.md)
- [ADR-0007 Plugin Boundary And Trust Tiers](../adr/en/adr-0007-plugin-boundary-and-trust-tiers.md)
- [ADR-0008 Security Overlay Is Additive](../adr/en/adr-0008-security-overlay-is-additive.md)
- [ADR-0009 Native IPC ABI Stability And Fallback](../adr/en/adr-0009-native-ipc-abi-stability-and-fallback.md)
- [ADR-0010 Broker Identity, Federation, And Failover](../adr/en/adr-0010-broker-identity-federation-and-failover.md)
- [ADR-0011 Benchmark Gate And Release Evidence](../adr/en/adr-0011-benchmark-gate-and-release-evidence.md)
- [ADR-0012 GitHub Protection And Docs-First Collaboration](../adr/en/adr-0012-github-protection-and-docs-first-collaboration.md)
- [ADR-0013 Phase 00 Foundation Completion Gate](../adr/en/adr-0013-phase-00-foundation-completion.md)

## When To Create A New ADR

Create a new ADR when a change would otherwise require future contributors to
reverse-engineer intent from old commits, chat logs, or benchmark spreadsheets.

## Review Rule

If a change touches a normative design document and substantially changes its
meaning, the change should either:

- reference an accepted ADR
- or create a proposed ADR in the same planning window

## Index Maintenance Rule

The index should be updated when:

- a new ADR ID is reserved
- an ADR changes status
- a decision is superseded by a later ADR

## Non-Negotiable Invariants

- ADRs should clarify design intent, not replace the design documents.
- Accepted ADRs should remain discoverable from the docs tree.
- Repository process decisions that materially shape implementation quality
  deserve the same traceability as packet-format decisions.
