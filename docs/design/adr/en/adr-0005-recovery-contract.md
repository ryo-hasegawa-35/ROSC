# ADR-0005: Recovery Contract

- Status: accepted
- Date: 2026-03-23

## Context

Recovery is one of the project’s core differentiators, but unsafe replay can be
more dangerous than packet loss.

## Decision

- separate `rehydrate` from `replay`
- define cache classes explicitly
- require route-level policy for automatic versus manual recovery
- treat trigger-like traffic as non-automatic by default

## Consequences

- late joiner recovery becomes powerful without pretending all traffic is safe
- operator UI and logs must distinguish recovery paths clearly
- capture and replay tooling must respect route policy

## Rejected Alternatives

- automatic replay of all recent traffic
- inferring recoverability from address names alone

## Affected Documents

- [Recovery Model And Cache Semantics](../../en/recovery-model-and-cache-semantics.md)
- [Operator Workflow And Recovery Playbook](../../en/operator-workflow-and-recovery-playbook.md)
- [Route Rule Cookbook And Worked Examples](../../en/route-rule-cookbook-and-worked-examples.md)
