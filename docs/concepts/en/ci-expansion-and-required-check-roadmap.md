# CI Expansion And Required-Check Roadmap

## Purpose

This document defines the repository automation baseline that should exist
before production Rust code appears, and the next required-check evolution once
code starts landing.

## Current Baseline

The repository currently uses:

- `Docs Quality`
- `PR Governance`
- `Repository Sanity Matrix`

These checks protect documentation integrity, pull request structure, and
cross-platform repository readiness without pretending the Rust implementation
already exists.

## Pre-Implementation Required Checks

`main` should require:

- `docs-consistency`
- `pr-body-policy`

The cross-platform matrix should run on pull requests, but it does not need to
be a required gate until the maintainer decides the signal is stable enough.

## Cross-Platform Readiness Rule

Before Rust code begins, CI should still prove that:

- the repository can be checked out on Windows, macOS, and Linux
- required governance files exist
- bilingual root documents exist
- fixture manifests and ADR trees are present and parse cleanly

## First Code-Era Workflow Split

When the first Rust workspace files are added, expand CI into these lanes:

1. repository and docs quality
2. formatting and lint
3. unit and integration tests
4. conformance corpus validation
5. benchmark and fuzz evidence
6. release packaging and signing evidence

## Required-Check Evolution

After the first code phase stabilizes, `main` should require:

- `docs-consistency`
- `pr-body-policy`
- `repo-sanity (ubuntu-latest)`
- `repo-sanity (windows-latest)`
- `repo-sanity (macos-latest)`
- the first Rust formatting and test jobs once they exist

## Naming Rule

Workflow and job names should stay stable once they become required checks.
Renaming a required status context should be treated as a repository-governance
change, not casual cleanup.

## Non-Goals

This roadmap does not yet define:

- benchmark thresholds
- release signing implementation
- secret-management policy for future package publishing

Those belong to later implementation and release phases.
