# ROSC Skill: Design Guardian

## Use This Skill When

- a task changes the meaning of a technical rule
- compatibility semantics, recovery behavior, telemetry semantics, or trust
  boundaries are being clarified
- an ADR relationship may need to be referenced or extended

## Goal

Protect the normative design from accidental drift while still improving its
precision.

## Workflow

1. Find the existing spec or ADR that already governs the topic.
2. Decide whether the change belongs in `docs/concepts/` or `docs/design/`.
3. If the change is normative, prefer `docs/design/` and link the relevant ADR.
4. Update English and Japanese specs in parallel.
5. Check whether indexes, reading order, or glossary entries need adjustment.
6. Call out any unresolved design decisions explicitly rather than burying them.

## Review Questions

- does this change weaken compatibility expectations
- does it make operator-visible behavior less predictable
- does it blur the meaning of recovery vs replay
- does it create a hidden cross-platform assumption
- should there be a new ADR or issue
