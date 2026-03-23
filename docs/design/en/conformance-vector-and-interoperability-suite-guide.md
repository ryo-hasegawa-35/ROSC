# Conformance Vector And Interoperability Suite Guide

## Purpose

This document defines how the project should maintain conformance vectors and an
interoperability suite once implementation begins.

The aim is to protect compatibility claims with repeatable evidence, not with
memory or confidence.

## Design Goals

- make compatibility claims testable
- distinguish protocol correctness from product behavior
- keep regression evidence durable
- support both specification conformance and field interoperability

## Two Different Validation Families

### Conformance Vectors

Use for:

- deterministic reference cases
- packet and framing examples
- expected parser or encoder behavior
- compatibility mode behavior

### Interoperability Suite

Use for:

- communication with real tools or reference implementations
- transport behavior checks
- migration confidence for actual workflows

## Conformance Vector Categories

### OSC 1.0 Canonical Vectors

Should include:

- standard messages
- standard bundles
- padding examples
- big-endian numeric examples
- address pattern examples

### Legacy Tolerant Vectors

Should include:

- missing type tag examples
- forwardable opaque cases
- expected limitations on inspection and transform

### Extended Compatibility Vectors

Should include:

- `//` wildcard behavior
- extended type acceptance behavior
- framing edge cases

### Fault Behavior Vectors

Should include:

- malformed packet rejection
- unknown type handling
- route drop behavior expectations
- cache and replay safety expectations

## Vector Structure

Each vector should describe:

- identifier
- purpose
- input bytes or source artifact
- compatibility mode
- expected parse result
- expected routing eligibility
- expected transform eligibility
- expected cache or replay eligibility

## Interoperability Targets

The suite should eventually cover:

- UE5-side OSC workflows
- TouchDesigner-side OSC workflows
- browser control path via adapter
- at least one MQTT path
- at least one secure overlay path

## Interoperability Scenario Shape

Each scenario should define:

- participating components
- transport path
- compatibility mode
- expected success criteria
- known limitations

## Golden Output Policy

For stable conformance cases, preserve:

- input artifact
- expected normalized interpretation
- expected output or state

Golden artifacts should be reviewable by humans and machines.

## Field Regression Policy

Whenever a real interoperability bug is fixed, add:

- a conformance vector if the bug was deterministic
- an interoperability scenario if the bug involved tool interaction
- a replay artifact if timing or ordering mattered

## Versioning

The conformance and interoperability suite should version:

- vector schema version
- suite version
- associated broker design revision where useful

## Reporting

The suite should report at least:

- pass / fail
- environment summary
- profile used
- compatibility mode used
- artifact identifiers exercised

## Non-Negotiable Invariants

- conformance vectors must remain stable enough for regression comparison
- interoperability claims must be tied to concrete scenarios
- field bugs must strengthen the suite, not just close an issue
- compatibility language in product docs should map to the suite
