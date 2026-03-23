# ROSC Skill Catalog

## Purpose

Use these local skills to keep multi-agent work consistent across planning,
documentation, backlog management, and future implementation preparation.

## Common Constraints

- preserve OSC 1.0 backward compatibility
- treat advanced capabilities as additive overlays
- keep English and Japanese project docs paired
- keep `.agent/` mirrored with `.agents/`
- keep `.skill/` mirrored with `.skills/`
- prefer docs-first changes when semantics or governance change
- do not claim performance without evidence
- do not skip issue context, design references, or handoff notes

## Available Skills

### `docs-maintainer.md`

Use when the task is mainly about:

- writing or refining documents
- keeping English and Japanese files aligned
- updating indexes, reading order, and cross-links
- adding new repository guidance without changing runtime behavior

### `design-guardian.md`

Use when the task changes or clarifies:

- normative design behavior
- compatibility semantics
- routing, recovery, telemetry, plugin, or security boundaries
- ADR relationships and design-governance expectations

### `issue-curator.md`

Use when the task is about:

- creating or refining issues
- ensuring issues are actionable for future contributors or agents
- aligning backlog items to phases, milestones, risks, and documents

### `implementation-planner.md`

Use when the task is about:

- turning specs into execution slices
- deciding what should happen before coding
- defining crate boundaries, acceptance criteria, or validation gates
- sequencing the first engineering milestones without writing runtime code

### `compatibility-reviewer.md`

Use when the task requires a focused review of:

- OSC compatibility and extension handling
- operator-visible risk
- recovery and observability semantics
- performance claims and evidence quality

## Selection Heuristic

- docs update only: start with `docs-maintainer.md`
- design meaning changes: start with `design-guardian.md`
- backlog / GitHub / issue quality: start with `issue-curator.md`
- roadmap-to-work breakdown: start with `implementation-planner.md`
- risk review or compatibility check: start with `compatibility-reviewer.md`

If a task spans more than one area, use the most restrictive skill first, then
bring in the others.
