# Release Checklist And Operational Runbook

## Purpose

This document defines the minimum release checklist and operating runbook shape
for future production profiles.

The goal is to prevent releases from being treated as only a build artifact.
For infrastructure software, release quality also includes rollout clarity,
rollback clarity, and incident readiness.

## Release Checklist

## Section A: Build Integrity

Confirm:

- correct profile was built
- version metadata is present
- release notes are prepared
- expected design documents are current

## Section B: Configuration Safety

Confirm:

- config schema version is known
- migrations are reviewed
- last-known-good rollback path exists
- risky config deltas are identified

## Section C: Compatibility Safety

Confirm:

- compatibility modes in the profile are documented
- legacy tolerant behavior is still intentional where enabled
- release notes call out any compatibility risk

## Section D: Operational Safety

Confirm:

- safe mode path is known
- operator-facing warnings are documented
- recovery actions remain available
- diagnostics level matches profile expectations

## Section E: Security Safety

Confirm where applicable:

- trust defaults are documented
- secure overlays are profile-appropriate
- audit expectations are documented
- insecure fallback behavior is understood

## Section F: Verification

Confirm:

- relevant benchmark or regression evidence exists
- profile-specific test expectations were met
- known limitations are documented honestly

## Operational Runbook

The runbook should help an operator answer:

- how do I start this profile safely
- how do I tell if it is healthy
- what do I do first in an incident
- how do I recover or roll back

## Runbook Sections

### Startup

Should include:

- intended topology
- required dependencies
- first health checks
- how to start in safe mode

### Live Operation

Should include:

- what normal looks like
- what warnings matter
- where to find route and destination health

### Incident Response

Should include:

- slow destination procedure
- malformed traffic procedure
- node restart recovery procedure
- plugin failure procedure

### Recovery

Should include:

- rehydrate procedure
- replay safety procedure
- cache invalidation procedure
- fallback procedure

### Rollback

Should include:

- how to revert config
- how to revert release profile
- how to return to last-known-good state

## Ownership And Signoff

Every release should define:

- technical owner
- operational owner
- rollback owner

## Non-Negotiable Invariants

- no release is complete without rollback clarity
- no operational profile should ship without a safe mode story
- known limitations must be documented, not implied
- releases must be understandable to operators, not just developers
