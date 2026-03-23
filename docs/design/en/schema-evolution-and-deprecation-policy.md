# Schema Evolution And Deprecation Policy

## Purpose

This document defines how optional schemas should evolve over time without
causing silent breakage or chaos across generated code and live deployments.

## Design Goals

- keep schema versioning explicit
- distinguish additive change from breaking change
- give operators and integrators clear migration windows
- avoid silent semantic drift

## Evolution Principles

- schema remains optional
- version changes must be visible
- deprecation is a communication tool, not merely a warning field
- generated code must reflect version intent

## Change Classes

### Additive Change

Examples:

- new optional address
- new optional argument with explicit compatibility note
- new enum value where consumers can tolerate it

### Behavioral Change

Examples:

- unit interpretation change
- state vs trigger reinterpretation
- caching or recovery safety reinterpretation

These should be treated with greater caution than ordinary additive changes.

### Breaking Change

Examples:

- address removal
- argument order change
- type change
- required constraint tightening that invalidates old senders

## Deprecation States

Suggested states:

- `active`
- `deprecated`
- `scheduled_removal`
- `removed`

Each deprecated element should include:

- since version
- replacement guidance if any
- removal target if known

## Versioning Policy

Every schema package should declare:

- format version
- schema version
- compatibility statement

Recommended semantic policy:

- additive change increments minor version
- breaking change increments major version
- pure documentation change may increment patch version

## Generated Code Implications

Schema changes should specify whether generated bindings must:

- add new optional fields
- mark items deprecated
- refuse generation for incompatible targets
- provide migration hints

## Operational Migration Rules

When a schema changes, the system should be able to answer:

- can old senders still operate
- can old receivers still operate
- do routes need review
- do recovery and cache policies need review

## Validation Policy

Schema-aware tooling should surface:

- incompatible schema use
- deprecated element use
- migration recommendations

## Documentation Expectations

A deprecation or breaking change should document:

- what changed
- why it changed
- what should replace it
- what risks remain during transition

## Non-Negotiable Invariants

- schema evolution must never be silent
- deprecation metadata must be machine-readable
- breaking change must be clearly marked
- optional schema tooling must not pretend compatibility where it does not exist
