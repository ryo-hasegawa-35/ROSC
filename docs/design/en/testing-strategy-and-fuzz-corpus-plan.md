# Testing Strategy And Fuzz Corpus Plan

## Purpose

This document defines how the project should be tested once implementation
begins and how fuzz targets should be organized.

The goal is not only correctness. It is also confidence under stress, malformed
input, and long-running operation.

## Testing Priorities

- protocol correctness
- compatibility correctness
- failure containment
- recovery correctness
- performance predictability
- security behavior

## Test Layers

### Layer 1: Unit Tests

Use for:

- parser pieces
- encoding
- route matching
- value normalization
- config validation

### Layer 2: Property Tests

Use for:

- encode / decode roundtrip properties
- ordering guarantees where applicable
- invariants across compatibility modes

### Layer 3: Integration Tests

Use for:

- ingress-to-egress routing
- route policy behavior
- adapter interaction
- cache and recovery behavior

### Layer 4: Fault Injection Tests

Use for:

- slow destination
- malformed traffic storm
- plugin timeout
- adapter disconnect
- degraded mode transitions

### Layer 5: Soak Tests

Use for:

- long-running reliability
- memory growth detection
- queue stability
- recovery after repeated disruption

## Fuzzing Strategy

Fuzzing should target both byte-level and semantic-level failure surfaces.

## Fuzz Corpus Families

### Packet Parsing Corpus

Include:

- valid OSC 1.0 messages
- valid bundles
- nested bundles
- truncated packets
- misaligned padding
- malformed type tag strings

### Legacy Compatibility Corpus

Include:

- missing type tag packets
- ambiguous legacy payloads
- opaque but forwardable edge cases

### Extended Type Corpus

Include:

- optional tagged values
- arrays
- unknown tags
- mixed typed and unsupported content

### Framing Corpus

Include:

- size-prefixed stream edge cases
- SLIP framing errors
- broken packet boundary sequences

### Config Corpus

Include:

- duplicate route IDs
- invalid compatibility combinations
- unsafe cache and recovery combinations
- invalid profile combinations

### Security Corpus

Include:

- malformed secure envelope fields
- expired tokens
- mismatched scopes
- replay-like sequences

## Golden Reference Material

The project should maintain golden vectors for:

- OSC 1.0 examples from the specification
- chosen 1.1-oriented compatibility examples
- known legacy tolerant cases
- cache and rehydrate policy cases

## Regression Policy

Every real bug should add one or more of:

- a unit or integration regression test
- a fuzz corpus seed
- a golden replay or capture artifact if relevant

## Environment Matrix

Testing should eventually cover:

- Windows
- macOS
- Linux
- multiple release profiles where behavior differs

## Non-Negotiable Invariants

- compatibility bugs must produce permanent regression tests
- malformed input must never be treated as a niche concern
- recovery behavior must be tested as behavior, not assumed from code shape
- fuzzing must include config and security surfaces, not only packet bytes
