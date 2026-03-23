# ADR-0004: Traffic Classes And Isolation Rules

- Status: accepted
- Date: 2026-03-23

## Context

The broker only becomes trustworthy if overload and destination failure stay
localized and explicit.

## Decision

- use explicit traffic classes to drive queue, drop, and recovery policy
- isolate egress work per destination
- keep queues bounded
- implement breaker and quarantine behavior as first-class runtime states

## Consequences

- a slow destination should not destabilize healthy peers
- overload behavior becomes auditable and testable
- route authors must be explicit about traffic importance

## Rejected Alternatives

- one shared unbounded queue
- best-effort egress without breaker semantics

## Affected Documents

- [Fault Model And Overload Behavior](../../en/fault-model-and-overload-behavior.md)
- [Operator Workflow And Recovery Playbook](../../en/operator-workflow-and-recovery-playbook.md)
- [Metrics And Telemetry Schema](../../en/metrics-and-telemetry-schema.md)
