# GitHub Foundation And Collaboration Plan

## Purpose

This document defines the repository and collaboration baseline that should be
in place before major implementation begins.

The project is not only building software. It is building a trustworthy
cross-platform infrastructure product. GitHub setup should therefore reinforce
compatibility, evidence, review clarity, and documentation discipline from the
beginning.

Related documents:

- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [Effort And Risks](./effort-and-risks.md)
- [Implementation Readiness Checklist](../../design/en/implementation-readiness-checklist.md)
- [Architecture Decision Record Index](../../design/en/architecture-decision-record-index.md)

## Collaboration Principles

- docs-first before risky implementation
- compatibility-first before feature expansion
- evidence-first before performance claims
- cross-platform-first before workflow hardening
- rollback-first before release confidence

## Recommended Repository Baseline

### Default Branch

- use `main` as the protected default branch

### Working Branches

- use short-lived branches
- prefer descriptive names such as `feature/<topic>`, `docs/<topic>`, or
  `fix/<topic>` so human contributors can understand intent immediately
- if an automation tool uses its own branch prefix, treat that as a tool-level
  exception rather than the project-wide naming rule

### Protected Branch Rules

- pull request required for merges
- force-push disabled on protected branches
- required status checks before merge
- review required for architecture-affecting changes
- branch must be up to date with required checks when practical

## Repository Structure Expectations

At minimum, the repository should clearly separate:

- source code
- docs
- benchmarks
- test assets and conformance vectors
- examples and sample configurations
- GitHub policy files and templates

The existing `docs/concepts` and `docs/design` split should be preserved.

## Required GitHub Baseline Files

Before major coding begins, plan to add:

- `README.md`
- `LICENSE`
- `CONTRIBUTING.md`
- `CODEOWNERS`
- pull request template
- issue templates
- security policy
- changelog or release note policy

These files should align with the architecture docs rather than repeat them
loosely.

## Issue Taxonomy

At minimum, issues should distinguish:

- design
- implementation
- bug
- compatibility
- performance
- observability
- recovery
- security
- documentation
- research

This helps future planning avoid mixing product exploration with regression
work.

## Label Families

Useful label families include:

- `type:*`
- `area:*`
- `priority:*`
- `status:*`
- `compat:*`
- `profile:*`
- `risk:*`

Examples:

- `type:design`
- `area:routing-core`
- `compat:legacy-tolerant`
- `profile:secure-installation`
- `risk:operator-visible`

## Milestone Strategy

Milestones should map to the project phases rather than arbitrary dates:

- Phase 00 foundation
- Phase 01 core proxy
- Phase 02 observability and recovery
- Phase 03 adapters and discovery
- Phase 04 extensibility and schema
- Phase 05 native integration
- Phase 06 security and sync

This keeps issue tracking aligned with the roadmap already documented.

## Pull Request Expectations

Every significant PR should answer:

- what changed
- why the change is needed
- which design docs it touches
- what compatibility risk exists
- what evidence supports the change
- how rollback or fallback works

For doc-only PRs, evidence may be design consistency rather than runtime tests.

## Review Ownership

Review expectations should be explicit for:

- routing core and compatibility
- recovery and cache semantics
- observability and benchmark claims
- security overlays
- packaging and release process

`CODEOWNERS` should reflect these architecture boundaries.

## Initial CI Plan Before Implementation

Before code-heavy CI exists, the repository should still validate:

- markdown formatting or consistency
- internal documentation links
- obvious broken paths to local references
- optional spelling or terminology checks where helpful

The first CI stage should protect document quality, not pretend code quality
already exists.

## CI Expansion After Coding Starts

Later stages should add:

- formatting and lint checks
- unit and integration tests
- fuzz or regression entry points
- benchmark or conformance reporting hooks
- release artifact verification

## Release And Tagging Policy

- tags should map to meaningful release or preview states
- release notes should link to evidence, not only summaries
- compatibility-sensitive changes should be highlighted explicitly

## Security And Disclosure Baseline

Before public distribution grows, define:

- where security reports should go
- how long reports are acknowledged within
- how fixes are coordinated with releases

This matters more once secure overlays and network-facing adapters land.

## Documentation Gate Before Major Code Work

Implementation should pause if:

- a major design area lacks both English and Japanese docs
- an architecture-changing decision has no ADR trail
- GitHub review rules would allow risky compatibility changes without review

## Immediate Setup Checklist

Before the first major implementation sprint, prepare:

- protected `main`
- branch naming convention
- PR template
- issue templates
- label taxonomy
- milestone plan
- CODEOWNERS draft
- docs quality CI

## Non-Negotiable Rules

- Repository process must reinforce architecture, not undermine it.
- GitHub automation should surface compatibility and operational risk, not hide
  it.
- Documentation quality is part of the product foundation.
