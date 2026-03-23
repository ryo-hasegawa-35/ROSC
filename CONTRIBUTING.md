# Contributing To ROSC

## Purpose

This repository is being built as a documentation-led foundation for a
high-reliability OSC routing and broker platform. Contribution rules exist to
protect compatibility, design clarity, and future operational trust.

## First Read

Before opening a substantial pull request, read:

1. [README.md](./README.md)
2. [Documentation Index](./docs/README.md)
3. [Design Reading Order](./docs/design/en/reading-order.md)
4. [Implementation Readiness Checklist](./docs/design/en/implementation-readiness-checklist.md)
5. [GitHub Foundation And Collaboration Plan](./docs/concepts/en/github-foundation-and-collaboration-plan.md)

## Contribution Workflow

1. Start from an existing issue or create a new issue with enough context to
   execute the work.
2. Align the work with the relevant design and planning documents.
3. Create a short-lived branch from `main`.
4. Use a short branch name that describes the work, such as
   `feature/<topic>`, `docs/<topic>`, or `fix/<topic>`.
5. Make the smallest coherent change that advances the issue.
6. Open a pull request using the repository PR template.
7. Wait for final approval from `@ryo-hasegawa-35` before merging to `main`.

## Pull Request Expectations

Every substantial PR should explain:

- what changed
- why the change is needed
- which issue it closes or advances
- which design documents are affected
- what compatibility risk exists
- what evidence supports the change
- how to roll back or fall back safely

## Docs-First Rule

If a change affects architecture, compatibility, fault handling, recovery,
telemetry meaning, or repository-wide development policy, update the documents
first or in the same pull request.

Relevant examples:

- route semantics
- cache and recovery semantics
- plugin trust boundary
- security overlay behavior
- benchmark interpretation
- GitHub governance policy

## Bilingual Documentation Rule

Project-level documents should exist in both English and Japanese.

At minimum, when adding or changing documents under `docs/concepts/` or
`docs/design/`, contributors should preserve the English and Japanese pair.

## AI Collaboration Files

If a change touches `.agent/`, `.agents/`, `.skill/`, or `.skills/`:

- update the mirrored tree in the same pull request
- keep `AGENT.md` and `AGENTS.md` aligned in meaning
- keep `SKILL.md` and `SKILLS.md` aligned in meaning
- update the relevant `docs/` files if repository policy or workflow meaning
  changed

## Design Governance Rule

If a PR changes the meaning of a normative design document, it should reference
an accepted ADR or propose a new ADR in line with:

- [Architecture Decision Record Index](./docs/design/en/architecture-decision-record-index.md)

## Compatibility Rule

Backward compatibility with existing OSC traffic is a core value of the
project. Performance work must not quietly weaken compatibility behavior.

## Safe Change Rule

Avoid landing changes that:

- make experimental acceleration paths mandatory
- couple observability to critical routing liveness
- erase rollback paths
- blur the distinction between rehydrate and replay

## Repository Hygiene

- Keep pull requests focused.
- Prefer updating the exact documents your change touches.
- Do not mix unrelated cleanup into a task-oriented PR.
- Keep issue and PR titles concrete and searchable.

## Security Reporting

Do not open a public issue for suspected vulnerabilities that could put users
or deployments at risk. Follow [SECURITY.md](./SECURITY.md) instead.
