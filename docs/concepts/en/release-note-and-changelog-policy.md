# Release Note And Changelog Policy

## Purpose

This document defines how ROSC should describe releases, previews, and major
repository milestones before and after implementation begins.

The goal is to keep release communication evidence-based, compatibility-aware,
and useful to operators, not only developers.

## Pre-Implementation Rule

Until production code exists, the changelog should record major repository and
design-foundation milestones rather than pretend runtime releases already
exist.

That means the changelog may include:

- governance milestones
- design-baseline milestones
- CI and fixture-readiness milestones
- major documentation and workflow hardening

## Changelog Purpose

The root changelog should summarize user-relevant milestones, not every commit.

It should help a future reader answer:

- what changed in this repository phase
- why it mattered
- what the next implementation era inherited

## Release Notes Purpose

Once runnable artifacts exist, release notes should explain:

- what changed
- what compatibility impact exists
- what operator-visible impact exists
- what evidence supports the claims
- what rollback or fallback path exists

## Required Release Note Sections

Future release notes should include:

- summary of the release
- affected profiles or deployment topologies
- compatibility impact
- observability or operator workflow impact
- recovery and rollback notes
- evidence links
- known limitations

## Evidence Rule

Release notes should link to actual evidence where applicable:

- benchmarks
- conformance results
- CI runs
- design docs
- issue or ADR trails

ROSC should not publish marketing-style claims without traceable evidence.

## Compatibility Rule

Any release note touching compatibility should say explicitly whether the
change affects:

- strict behavior
- legacy-tolerant behavior
- extended behavior
- no compatibility semantics at all

## Changelog Maintenance Rule

- keep entries concise
- prefer milestone-oriented language
- avoid commit-by-commit noise
- update English and Japanese changelog files together

## GitHub Release Automation Rule

GitHub-generated release notes can help with categorization, but they are not
the final editorial layer.

Human curation should still verify:

- compatibility-sensitive items are highlighted honestly
- operator-visible risk is not buried
- release notes match the actual evidence trail

## Non-Negotiable Rule

ROSC release communication should make the project easier to trust, not merely
easier to advertise.
