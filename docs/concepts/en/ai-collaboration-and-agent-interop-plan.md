# AI Collaboration And Agent Interop Plan

## Purpose

This document defines how ROSC should prepare for a repository where multiple
AI systems may contribute over time.

The goal is not merely to make assistants "work", but to make them converge on
the same architecture, compatibility promises, review discipline, and handoff
quality.

## Why This Exists

Different AI tools discover repository context in different ways.

Some look for:

- `.agent/`
- `.agents/`
- `.skill/`
- `.skills/`
- `AGENT.md`
- `AGENTS.md`
- `SKILL.md`
- `SKILLS.md`

If the repository only supports one discovery convention, future contributors
can easily miss critical constraints and produce conflicting work. ROSC should
therefore provide a compatibility layer for agent discovery without letting the
short AI-facing files become a second, drifting design system.

## Canonical Source Model

The canonical project meaning still lives in `docs/` and the root repository
policy files.

The AI entry directories exist to:

- shorten onboarding time for new agents
- expose the must-read rules in one place
- define handoff and collaboration discipline
- advertise reusable local skills

They do not replace the formal documentation stack.

## Required AI Entry Trees

ROSC should keep these mirrored directory families:

- `.agent/`
- `.agents/`
- `.skill/`
- `.skills/`

### Agent Trees

The agent trees are for project-wide context and safe workflow expectations.

They should answer:

- what the project is
- what stage the repository is in
- what must be read first
- what must not be broken
- how work should be handed off

### Skill Trees

The skill trees are for role-specific execution guidance.

They should answer:

- which skill to use for docs work
- which skill to use for design changes
- which skill to use for backlog shaping
- which skill to use for pre-code planning
- which skill to use for compatibility-focused review

## Mirror Policy

- `.agent/` and `.agents/` should stay textually aligned
- `.skill/` and `.skills/` should stay textually aligned
- compatibility alias files such as `AGENT.md` / `AGENTS.md` and
  `SKILL.md` / `SKILLS.md` should not diverge in meaning
- when these files change, the related formal docs should be updated if the
  underlying policy changed

## Recommended Content Shape

The project should maintain both short entry files and supporting files.

### Short Entry Files

These should be optimized for first contact:

- mission
- project status
- must-read list
- non-negotiable rules
- handoff expectations

### Supporting Files

These should cover:

- repository map
- workflow and safety rules
- role-based skill guides
- handoff template

## Governance Expectations

All AI systems should be steered toward the same governance rules:

- docs-first for architecture and semantics
- bilingual project documentation
- pull-request-based review on protected `main`
- final approval reserved for the repository owner
- compatibility and rollback discipline before acceleration claims

## Handoff Contract

Every substantial AI contribution should leave a compact handoff that states:

- what changed
- what documents or issues governed the work
- what was validated
- what remains unresolved
- what the next best step is

This is important because ROSC is intentionally being shaped for
multi-contributor and multi-agent continuity, not one-off sessions.

## Relationship To The Existing Docs

This plan complements, but does not replace:

- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [GitHub Backlog Map](./github-backlog-map.md)
- [Implementation Readiness Checklist](../../design/en/implementation-readiness-checklist.md)

## Non-Negotiable Outcome

The repository should make it easier for different AI systems to produce
consistent work, not easier for them to bypass design discipline.
