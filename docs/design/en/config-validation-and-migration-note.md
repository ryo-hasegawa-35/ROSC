# Config Validation And Migration Note

## Purpose

This document defines how configuration should be validated before apply and how
configuration versions should migrate over time.

## Design Goals

- reject bad config before it reaches live routing
- preserve operator confidence during hot reload
- make upgrades explicit and auditable
- support rollback to the last known-good config

## Validation Stages

### Stage 1: Syntax Validation

Checks:

- parseable file
- expected top-level structure
- supported schema version field present

### Stage 2: Schema Validation

Checks:

- required fields present
- enum values valid
- field types valid
- duplicate IDs rejected

### Stage 3: Semantic Validation

Checks:

- compatible mode and pattern combinations
- destination references resolve
- cache and recovery policy combination is safe
- security policy is internally coherent
- plugin references are valid for the build profile

### Stage 4: Runtime Validation

Checks:

- ports are available where required
- adapters exist and are enabled
- dangerous live transitions are flagged

## Validation Output

Validation should classify findings as:

- error
- warning
- advisory

Errors block apply.
Warnings require explicit acknowledgement if policy demands it.

## Migration Model

Each config should include:

- schema version
- compatibility profile version
- optional migration history

## Migration Types

### Automatic Safe Migration

Use when:

- field rename is mechanical
- default insertion is safe

### Assisted Migration

Use when:

- semantics changed
- operator choice is required

### Manual Migration

Use when:

- automation could change behavior dangerously

## Last-Known-Good Policy

The system should preserve:

- last applied good config
- validation report
- apply timestamp
- operator identity where applicable

## Hot Reload Rules

- validate fully before cutover
- apply atomically where possible
- failed apply leaves last-known-good active
- operator sees config diff before risky changes

## Compatibility Profile

Config validation should understand compatibility intent:

- strict routing profile
- legacy tolerant profile
- extended feature profile
- secure profile
- safe mode profile

## Auditability

Record at least:

- who applied config
- what changed
- validation result
- rollback events

## Non-Negotiable Invariants

- config apply must not be partial and silent
- migration must never rewrite behavior without visibility
- rollback path must remain available
- validation must include semantics, not just syntax
