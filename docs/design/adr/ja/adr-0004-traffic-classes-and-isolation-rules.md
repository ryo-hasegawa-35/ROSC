# ADR-0004: Traffic Classes And Isolation Rules

- Status: accepted
- Date: 2026-03-23

## Context

broker が trust できるものになるには、overload と destination failure が
局所化され、明示されていなければいけません。

## Decision

- traffic class を explicit に定義し、queue / drop / recovery policy を決める
- egress work は destination ごとに isolate する
- queue は bounded に保つ
- breaker と quarantine を first-class runtime state として扱う

## Consequences

- 遅い destination が healthy peer を巻き込みにくくなる
- overload behavior が audit 可能で testable になる
- route author は traffic importance を明示する必要がある

## Rejected Alternatives

- 1 本の shared unbounded queue
- breaker semantics のない best-effort egress

## Affected Documents

- [Fault Model And Overload Behavior](../../ja/fault-model-and-overload-behavior.md)
- [Operator Workflow And Recovery Playbook](../../ja/operator-workflow-and-recovery-playbook.md)
- [Metrics And Telemetry Schema](../../ja/metrics-and-telemetry-schema.md)
