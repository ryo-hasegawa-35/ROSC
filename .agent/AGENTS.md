# ROSC Agent Brief

## Project Status

ROSC is a docs-first Rust project for a next-generation OSC routing bus and
message broker.

The repository is still in the pre-implementation phase. Production Rust crates
do not exist yet. At this stage, agents should prefer documentation,
architecture clarification, backlog hygiene, fixtures, CI scaffolding, and
implementation planning over shipping runtime code.

## Mission

Build a routing core that is:

- fast under sustained real-time pressure
- deterministic and observable
- recoverable after overload or restart
- backward-compatible with existing OSC workflows
- extensible without bloating the core
- usable across Windows, macOS, and Linux

## Read Before Action

Read these before making substantial changes:

1. `../README.md`
2. `../README.ja.md`
3. `../docs/design/ja/reading-order.md`
4. `../docs/design/ja/glossary.md`
5. `../docs/design/ja/implementation-readiness-checklist.md`
6. `../docs/concepts/ja/github-foundation-and-collaboration-plan.md`
7. `./project-map.md`
8. `./working-agreement.md`

## Non-Negotiable Rules

- Preserve backward compatibility with existing OSC 1.0 traffic.
- Treat advanced behavior as additive overlays, not mandatory changes to raw
  OSC.
- Keep observability, recovery, and rollback paths first-class.
- Prefer docs-first changes when architecture, compatibility, or operator
  behavior changes.
- Keep project-level documentation available in both English and Japanese.
- Do not bypass pull-request review or the repository protection around `main`.
- Assume future contributors and agents will rely on clear issue context and
  explicit handoff notes.

## Working Model

1. Start from an issue when practical, or create one if the work introduces a
   new stream of effort.
2. Identify whether the task belongs in `docs/concepts/` or `docs/design/`.
3. Update English and Japanese files as a pair.
4. If the task changes architecture intent, compatibility meaning, fault
   handling, recovery semantics, telemetry interpretation, or plugin trust
   boundaries, update the design docs first or in the same PR.
5. If the task changes one of the AI entry directories, mirror the change in
   `.agent/` and `.agents/`, or in `.skill/` and `.skills/`.
6. Keep changes small, scoped, and easy to review.
7. Validate links and repository checks after edits.
8. Hand off using `./handoff-template.md`.

## If Implementation Starts Later

When the repository moves into code implementation:

- align work to the approved design docs and ADRs
- preserve the compatibility modes and recovery contract
- use the conformance and benchmark fixture plans as the baseline evidence set
- do not claim performance wins without reproducible evidence
- keep Windows, macOS, and Linux in scope from the first build/test pipeline

## Where To Look Next

- For the repository map: `./project-map.md`
- For workflow and safety rules: `./working-agreement.md`
- For role-specific execution guides: `../.skill/SKILL.md`
- For the formal AI collaboration policy:
  `../docs/concepts/en/ai-collaboration-and-agent-interop-plan.md`
