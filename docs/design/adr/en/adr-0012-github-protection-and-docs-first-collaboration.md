# ADR-0012: GitHub Protection And Docs-First Collaboration

- Status: accepted
- Date: 2026-03-23

## Context

The project is still pre-implementation, so repository process is part of the
architecture foundation rather than an administrative afterthought.

## Decision

- keep `main` protected and PR-only
- require maintainer approval before changes land on `main`
- treat design and ADR updates as prerequisites for risky semantic changes
- maintain English and Japanese project documentation as a repository rule

## Consequences

- implementation work stays traceable to planning and design decisions
- repository automation becomes part of the quality story
- contributors need to work more explicitly, but the architecture stays safer

## Rejected Alternatives

- direct pushes for major changes
- code-first semantic changes without documentation or ADR updates

## Affected Documents

- [GitHub Foundation And Collaboration Plan](../../../concepts/en/github-foundation-and-collaboration-plan.md)
- [GitHub Backlog Map](../../../concepts/en/github-backlog-map.md)
- [Implementation Readiness Checklist](../../en/implementation-readiness-checklist.md)
