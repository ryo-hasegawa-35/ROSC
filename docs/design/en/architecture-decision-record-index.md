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

When ADR files are added, store them in mirrored language trees:

- `docs/design/adr/en/`
- `docs/design/adr/ja/`

The index may exist before the individual ADR files do.

## Initial ADR Backlog

These are the first decisions that should become formal ADRs before or during
initial implementation.

### ADR-0001: Compatibility Mode Contract

Scope:

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`
- legacy missing-type-tag policy

### ADR-0002: Dual Packet Representation

Scope:

- raw byte retention
- normalized internal view
- parse failure and unknown-tag behavior

### ADR-0003: Route Semantic Model Before File Format

Scope:

- semantic route fields
- TOML as first external format
- validation before apply

### ADR-0004: Traffic Classes And Isolation Rules

Scope:

- traffic class vocabulary
- per-destination isolation
- breaker and quarantine expectations

### ADR-0005: Recovery Contract

Scope:

- rehydrate versus replay
- cache classes
- automatic versus manual recovery boundaries

### ADR-0006: Telemetry Levels And Cardinality Budget

Scope:

- `metrics_level` semantics
- canonical metric names
- bounded-label policy

### ADR-0007: Plugin Boundary And Trust Tiers

Scope:

- plugin trust tiers
- Wasm versus external process boundaries
- broker-owned safety semantics

### ADR-0008: Security Overlay Is Additive

Scope:

- source verification at the broker edge
- legacy bridge treatment
- raw OSC backward compatibility

### ADR-0009: Native IPC ABI Stability And Fallback

Scope:

- IPC acceleration remains optional
- UDP fallback remains first-class
- ABI versioning expectations

### ADR-0010: Broker Identity, Federation, And Failover

Scope:

- broker identity
- replication scope
- split-brain prevention
- failover authority

### ADR-0011: Benchmark Gate And Release Evidence

Scope:

- mandatory benchmark context
- interpretation classes
- release evidence requirements

### ADR-0012: GitHub Protection And Docs-First Collaboration

Scope:

- protected branch baseline
- review expectations
- documentation gate before risky code

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
