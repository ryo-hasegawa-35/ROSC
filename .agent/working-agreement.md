# ROSC Working Agreement

## Purpose

This file defines how AI agents should work in this repository without creating
drift, ambiguity, or unsafe shortcuts.

## Before You Start

- identify the related issue, milestone, or roadmap area
- identify the affected design or concept documents
- check whether the work changes repository policy, architecture, compatibility,
  telemetry meaning, recovery behavior, or plugin trust boundaries
- if yes, update or consult the relevant docs before claiming the task is done

## Documentation Rules

- project-level docs must remain bilingual
- `docs/concepts/` is for intent, roadmap, governance, and planning
- `docs/design/` is for normative behavior, interfaces, semantics, and
  operational rules
- do not move content between these families casually
- keep links healthy when renaming or adding files

## AI Entry-Point Rules

- `.agent/` and `.agents/` are compatibility mirrors
- `.skill/` and `.skills/` are compatibility mirrors
- if one tree changes, update the matching tree in the same pull request
- if a short AI-facing instruction changes meaning, update the formal docs
  under `docs/` when needed

## Safety And Quality Boundaries

- do not weaken OSC backward compatibility without explicit design approval
- do not make optional security overlays mandatory for raw OSC interoperability
- do not claim performance improvements without reproducible evidence
- do not remove observability or rollback paths in the name of speed
- do not blur the distinction between rehydrate and replay
- do not bypass PR-based review to land changes on `main`

## Implementation Boundary For Now

The repository is still pre-implementation.

Allowed high-value work now:

- documentation improvements
- issue and backlog refinement
- CI scaffolding for docs and repository hygiene
- fixture inventories and reproducibility notes
- implementation planning, crate planning, and contract definition

Avoid for now unless explicitly requested:

- production runtime implementation
- speculative code generation not grounded in the approved docs
- benchmark claims without a harness and evidence trail

## Change Scope Rule

- prefer the smallest coherent change that moves one topic forward
- avoid mixing unrelated cleanup into the same branch
- keep PR summaries concrete, searchable, and linked to issues/docs

## Completion Checklist

- English and Japanese pairs updated
- mirror directories kept in sync
- links verified
- affected docs and issues referenced
- open questions recorded
- handoff written with next recommended step
