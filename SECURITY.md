# Security Policy

## Scope

ROSC is intended to become network-facing infrastructure software. Security
issues therefore include more than memory safety bugs. They may also include:

- source verification bypasses
- identity or scope confusion
- replay or recovery abuse paths
- unsafe default configuration
- plugin or adapter trust boundary escapes
- cross-broker failover or federation safety flaws

## Reporting A Vulnerability

Please do not open a public issue for a suspected vulnerability that could be
exploited in real deployments.

Preferred path:

1. Use GitHub private vulnerability reporting for this repository, if available.
2. If private reporting is unavailable, contact the repository owner through
   GitHub before publishing details.

## What To Include

Please include as much of the following as possible:

- affected component or design area
- reproduction steps
- expected versus actual behavior
- deployment context
- whether the issue affects compatibility, safety, or data isolation
- whether a workaround exists

## Disclosure Expectations

- Reports should be acknowledged as quickly as practical.
- Fix coordination should prioritize safe remediation over public speed.
- Public disclosure should wait until the maintainer has had a reasonable
  opportunity to investigate and respond.

## Current Project State

This repository is currently pre-implementation. Some security issues may exist
at the design or process level before production code exists. Those are still
worth reporting privately when they materially affect safety.
