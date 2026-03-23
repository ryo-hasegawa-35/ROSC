# OSC Conformance Corpus Plan

## Purpose

This document defines the structure and initial seed set for the OSC
conformance corpus that future parser, encoder, and interoperability tests will
 consume.

## Corpus Root

The repository fixture root is:

- `fixtures/conformance/`

Primary inventory files:

- `fixtures/conformance/catalog.json`
- `fixtures/conformance/vectors/*.hex.txt`

## Coverage Goals

The corpus must cover:

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`
- valid messages and bundles
- malformed alignment, truncation, and size cases
- legacy missing-type-tag traffic
- extension and unknown-tag behavior

## Catalog Fields

Each catalog entry should record:

- vector ID
- compatibility mode
- category
- expected disposition
- source reference
- fixture path
- short description

## Seed Categories

- basic scalar messages
- string and blob cases
- bundle framing
- legacy no-type-tag payloads
- malformed padding and truncation
- extended boolean and nil tags
- unknown extension tags

## Acceptance Rule

The conformance corpus is not a benchmark corpus. It exists to say whether
behavior is correct, rejected safely, or tolerated under an explicit legacy
mode.

## Initial Seed Inventory

The first seed inventory is stored in:

- `fixtures/conformance/catalog.json`

Those seeds are enough to bootstrap parser and encoder tests without deciding
the final automation format yet.
