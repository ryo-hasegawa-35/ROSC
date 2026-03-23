# ADR-0013: Phase 00 Foundation Completion Gate

- Status: accepted
- Date: 2026-03-23

## Context

The project needs an explicit statement of what counts as "ready to start
implementation" so the first code does not outrun the architecture base.

## Decision

- Phase 00 is complete only when licensing, ADRs, workspace planning,
  conformance planning, benchmark planning, and CI scaffolding exist
- the first implementation work starts only after those artifacts are present
- remaining Phase 00 changes after that point should be treated as incremental
  governance improvements, not blockers for core coding

## Consequences

- the project has a shared definition of implementation readiness
- future contributors can verify readiness without reconstructing chat history
- repository and design readiness become auditable from committed artifacts

## Rejected Alternatives

- starting implementation based on implicit verbal agreement
- leaving readiness criteria distributed only across issue text

## Affected Documents

- [Implementation Readiness Checklist](../../en/implementation-readiness-checklist.md)
- [Detailed Delivery Plan](../../../concepts/en/detailed-delivery-plan.md)
- [GitHub Foundation And Collaboration Plan](../../../concepts/en/github-foundation-and-collaboration-plan.md)
