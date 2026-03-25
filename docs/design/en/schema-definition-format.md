# Schema Definition Format

## Purpose

This document defines the semantic structure for optional typed OSC schemas used
for validation, tooling, and code generation.

The schema system is intentionally optional. It should improve reliability
without destroying the ad hoc flexibility that made OSC successful.

## Design Goals

- optional, never mandatory for raw OSC
- useful for validation before useful for bureaucracy
- language-neutral
- generation-friendly
- compatible with route-level and namespace-level use

## Scope

The schema format should be able to describe:

- namespaces
- addresses
- message arguments
- units
- constraints
- documentation
- versioning
- compatibility expectations

## Schema Layers

### Layer 1: Namespace Description

Defines:

- namespace owner
- namespace purpose
- compatibility mode expectation
- security scope hints

### Layer 2: Address Definition

Defines:

- concrete address or pattern
- message role
- argument sequence
- caching suitability
- recovery suitability

### Layer 3: Argument Definition

Defines:

- type
- optionality
- units
- range constraints
- enumeration or semantic meaning

### Layer 4: Tooling Metadata

Defines:

- documentation text
- generation hints
- deprecation markers
- migration notes

## Core Entities

Suggested top-level entities:

- `schema`
- `namespace`
- `message`
- `argument`
- `enum`
- `constraint`
- `profile`

## Type Model

The schema type model should map onto the supported broker value model.

Baseline types:

- `int32`
- `float32`
- `string`
- `blob`

Extended types:

- `int64`
- `double64`
- `timetag`
- `symbol`
- `char`
- `rgba`
- `midi4`
- `bool_literal`
- `nil`
- `impulse`
- `array`

## Constraint Model

The schema should support constraints such as:

- minimum / maximum
- allowed enum values
- regex-like textual restriction where appropriate
- array length bounds
- key extraction hint
- idempotent-state hint

## Semantic Hints

To support recovery and operations, the schema should be able to express:

- whether a message is state-like or trigger-like
- whether late joiner rehydrate is safe
- whether caching is recommended
- whether transform is safe

## Versioning

Each schema should declare:

- schema format version
- domain version
- compatibility statement

Recommended compatibility states:

- backward compatible
- additive change
- breaking change

## Generation Targets

The schema should support generation hints for:

- Rust value bindings
- C ABI descriptors
- C++ integration helpers
- Python integration helpers
- validation manifests

## Relationship To Raw OSC

Critical rule:

- the schema describes intended meaning
- the schema does not redefine raw OSC packet validity

That means:

- a message may be valid OSC but not schema-conformant
- schema validation should report this explicitly
- schema use remains opt-in by route, namespace, or tool

## Example Semantic Shape

A message definition should be able to answer:

- what address is this
- what does each argument mean
- what units are expected
- what ranges are safe
- is this state or trigger
- can this be cached
- can this be used for rehydrate

## Documentation Expectations

A schema entry should be useful to a human reader.

Minimum human-oriented fields:

- display name
- description
- example
- operational note where relevant

## Validation Levels

Schema tools should support at least:

- lint
- strict validation
- migration guidance

## Validation Cost Policy

Schema validation must not assume every route deserves the same depth of
checking.

Recommended validation depths:

- `off`: no schema validation on the data plane
- `shape_only`: cheap arity and basic type-family checks
- `typed`: full argument typing against the declared schema
- `strict`: typed validation plus range, enum, and semantic constraints

Recommended defaults:

- critical control routes may use `typed` or `strict`
- stateful but moderate-rate routes may use `typed`
- high-rate sensor and telemetry routes should default to `off` or
  `shape_only` unless an operator explicitly accepts the extra cost

The routing plan, benchmark plan, and operator UI should all make this
tradeoff visible.

## Non-Negotiable Invariants

- schema remains optional
- schema must not weaken raw OSC compatibility
- schema versioning must be explicit
- generated code must reflect schema intent, not invent additional semantics
- schema should reduce ambiguity rather than add ceremony
