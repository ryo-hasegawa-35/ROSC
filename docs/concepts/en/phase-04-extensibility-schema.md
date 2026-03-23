# Phase 04: Extensibility, Schema, And Code Generation

## Goal

Make advanced behavior pluggable so the system can grow without turning the core
into an oversized monolith.

## Deliverables

- Feature-flagged product presets
- Runtime Wasm filter engine
- Hot reload for approved filter modules
- Stable external plugin protocol for out-of-process extensions
- Schema definition format for:
  - addresses
  - argument types
  - units
  - constraints
  - documentation
- Validation and lint tooling
- Code generation targets:
  - Rust bindings
  - C ABI descriptors
  - UE5-oriented C++ wrappers
  - TouchDesigner-oriented Python helpers

## Plugin Strategy

- Use Wasm for packet transforms and computational extensions.
- Use external process plugins for protocol bridges and heavyweight connectors.
- Avoid relying on unstable native Rust plugin ABI for the main extension story.

## Schema Strategy

- Start with documentation and validation.
- Add code generation once the schema proves useful on real projects.
- Keep schema optional so ad-hoc OSC remains possible.

## Non-Goals

- No forced schema registration for legacy users
- No requirement that every message be predeclared
- No attempt to replace all ad-hoc creative workflows

## Exit Criteria

- Users can add a custom packet transform without recompiling the entire broker.
- Schemas can detect common type mismatches before runtime.
- Generated bindings reduce repetitive integration code in at least one real
  UE5 or TouchDesigner workflow.

## Rough Effort

160-300 hours

## Value

This phase creates the platform effect. The system stops being only "our router"
and starts becoming an extensible infrastructure layer.
