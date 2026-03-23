# ADR-0003: Route Semantic Model Before File Format

- Status: accepted
- Date: 2026-03-23

## Context

The project needs a stable route meaning before external configuration format
details become entrenched.

## Decision

- treat route semantics as normative and external file syntax as secondary
- use TOML as the first external configuration format
- require semantic validation before apply
- keep route and destination IDs stable and explicit

## Consequences

- future config formats can change without changing route meaning
- hot reload and last-known-good safety stay grounded in semantics
- documentation and examples can stay format-aware without being format-owned

## Rejected Alternatives

- making TOML structure the normative definition
- embedding route behavior directly in runtime code paths

## Affected Documents

- [Route Configuration Grammar](../../en/route-configuration-grammar.md)
- [Route Rule Cookbook And Worked Examples](../../en/route-rule-cookbook-and-worked-examples.md)
- [Config Validation And Migration Note](../../en/config-validation-and-migration-note.md)
