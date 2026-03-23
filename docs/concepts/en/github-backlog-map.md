# GitHub Backlog Map

## Purpose

This document records how the current GitHub backlog is structured so future
contributors can navigate the repository plan without reconstructing it from the
Issues tab alone.

Related documents:

- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [Design Reading Order](../../design/en/reading-order.md)

## Current Repository Governance Baseline

The repository currently has these baseline policies in place:

- `main` exists and is the active default branch
- pull requests are required before merging to `main`
- stale reviews are dismissed on new pushes
- code owner review is required for protected-branch merges
- `CODEOWNERS` currently points to `@ryo-hasegawa-35`
- branch conversation resolution is required
- force-push and deletion are disabled on `main`
- `Docs Quality` and `PR Governance` workflows are present

Important nuance:

- branch protection does not enforce admin restrictions
- this keeps final merge control with the repository owner while still
  requiring owner approval for ordinary pull requests via code-owner review

## Current Project Status

Issue tracking and project tracking are both seeded and usable today.

The active project is:

- [ROSC Delivery Board](https://github.com/users/ryo-hasegawa-35/projects/3)

Current project baseline:

- the repository is linked to the project
- all current backlog issues are added to the project
- active pull requests can also be tracked in the same project
- `Phase`, `Priority`, and `Area` fields are populated from the repository
  taxonomy
- the repository docs record the recommended roadmap, active-work, and
  blocked-work filters for future maintainers

Remaining nuance:

- [Issue #6](https://github.com/ryo-hasegawa-35/ROSC/issues/6) remains open
  while we decide whether documented filter-based views are sufficient or if
  named saved views should also be created manually in the GitHub UI

## Milestone Map

- `Phase 00 - Foundation And Governance`
- `Phase 01 - Core Proxy And Routing`
- `Phase 02 - Observability And Recovery`
- `Phase 03 - Adapters And Discovery`
- `Phase 04 - Extensibility And Schema`
- `Phase 05 - Native Integration`
- `Phase 06 - Security, Sync, And Release`

## Epic Map

- [Issue #34](https://github.com/ryo-hasegawa-35/ROSC/issues/34)
  `Phase 00 foundation and governance`
- [Issue #35](https://github.com/ryo-hasegawa-35/ROSC/issues/35)
  `Phase 01 core proxy and routing`
- [Issue #36](https://github.com/ryo-hasegawa-35/ROSC/issues/36)
  `Phase 02 observability and recovery`
- [Issue #37](https://github.com/ryo-hasegawa-35/ROSC/issues/37)
  `Phase 03 adapters and discovery`
- [Issue #38](https://github.com/ryo-hasegawa-35/ROSC/issues/38)
  `Phase 04 extensibility and schema`
- [Issue #39](https://github.com/ryo-hasegawa-35/ROSC/issues/39)
  `Phase 05 native integration`
- [Issue #40](https://github.com/ryo-hasegawa-35/ROSC/issues/40)
  `Phase 06 security, sync, and release`

## Task Map By Phase

### Phase 00

- [Issue #1](https://github.com/ryo-hasegawa-35/ROSC/issues/1)
  `Decide repository license and contributor policy`
- [Issue #3](https://github.com/ryo-hasegawa-35/ROSC/issues/3)
  `Materialize the initial ADR set from the design index`
- [Issue #8](https://github.com/ryo-hasegawa-35/ROSC/issues/8)
  `Define the Rust workspace and crate boundaries for the broker core`
- [Issue #2](https://github.com/ryo-hasegawa-35/ROSC/issues/2)
  `Build the OSC conformance corpus from the 1.0 and 1.1 references`
- [Issue #7](https://github.com/ryo-hasegawa-35/ROSC/issues/7)
  `Create benchmark fixtures and reproducible workload inputs`
- [Issue #5](https://github.com/ryo-hasegawa-35/ROSC/issues/5)
  `Expand GitHub Actions into cross-platform repository quality and future Rust CI scaffolding`
- [Issue #6](https://github.com/ryo-hasegawa-35/ROSC/issues/6)
  `Enable a GitHub Project board and seed the delivery views`

### Phase 01

- [Issue #11](https://github.com/ryo-hasegawa-35/ROSC/issues/11)
  `Implement the OSC compatibility parser and encoder core`
- [Issue #10](https://github.com/ryo-hasegawa-35/ROSC/issues/10)
  `Implement ingress transport bindings and bounded intake queues`
- [Issue #9](https://github.com/ryo-hasegawa-35/ROSC/issues/9)
  `Implement the route matcher and routing engine`
- [Issue #12](https://github.com/ryo-hasegawa-35/ROSC/issues/12)
  `Implement destination workers, circuit breakers, and fault isolation`
- [Issue #14](https://github.com/ryo-hasegawa-35/ROSC/issues/14)
  `Implement configuration loading, semantic validation, and safe hot reload`
- [Issue #13](https://github.com/ryo-hasegawa-35/ROSC/issues/13)
  `Implement the minimal metrics endpoint and health-reporting surface`

### Phase 02

- [Issue #15](https://github.com/ryo-hasegawa-35/ROSC/issues/15)
  `Implement the operations dashboard shell and core runtime pages`
- [Issue #33](https://github.com/ryo-hasegawa-35/ROSC/issues/33)
  `Implement cache classes and the late-joiner rehydrate engine`
- [Issue #16](https://github.com/ryo-hasegawa-35/ROSC/issues/16)
  `Implement bounded capture, replay, and operator recovery auditing`

### Phase 03

- [Issue #19](https://github.com/ryo-hasegawa-35/ROSC/issues/19)
  `Implement the WebSocket / JSON adapter`
- [Issue #18](https://github.com/ryo-hasegawa-35/ROSC/issues/18)
  `Implement the MQTT adapter`
- [Issue #20](https://github.com/ryo-hasegawa-35/ROSC/issues/20)
  `Implement the discovery and service-metadata runtime`
- [Issue #21](https://github.com/ryo-hasegawa-35/ROSC/issues/21)
  `Implement adapter interoperability and conformance harnesses`

### Phase 04

- [Issue #23](https://github.com/ryo-hasegawa-35/ROSC/issues/23)
  `Implement the plugin capability registry and external plugin boundary`
- [Issue #24](https://github.com/ryo-hasegawa-35/ROSC/issues/24)
  `Implement the Wasm transform runtime and hot-reload lifecycle`
- [Issue #22](https://github.com/ryo-hasegawa-35/ROSC/issues/22)
  `Implement the schema parser, validator, and compatibility-aware type model`
- [Issue #25](https://github.com/ryo-hasegawa-35/ROSC/issues/25)
  `Implement code generation targets for UE5 and TouchDesigner`

### Phase 05

- [Issue #27](https://github.com/ryo-hasegawa-35/ROSC/issues/27)
  `Implement the stable C ABI and shared-memory IPC transport`
- [Issue #28](https://github.com/ryo-hasegawa-35/ROSC/issues/28)
  `Build the UE5 native integration package`
- [Issue #26](https://github.com/ryo-hasegawa-35/ROSC/issues/26)
  `Build the TouchDesigner native integration package`

### Phase 06

- [Issue #31](https://github.com/ryo-hasegawa-35/ROSC/issues/31)
  `Implement the security overlay and verified-source enforcement`
- [Issue #29](https://github.com/ryo-hasegawa-35/ROSC/issues/29)
  `Implement timing diagnostics and advanced sync groundwork`
- [Issue #30](https://github.com/ryo-hasegawa-35/ROSC/issues/30)
  `Implement broker identity, federation, and active-standby control plane`
- [Issue #32](https://github.com/ryo-hasegawa-35/ROSC/issues/32)
  `Implement packaging, release profiles, and the CI evidence pipeline`

## Reading Recommendation

If someone is new to the repo, the most efficient onboarding order is:

1. `README.md`
2. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
3. [Design Reading Order](../../design/en/reading-order.md)
4. the relevant epic issue for the phase they will touch
5. the child task issue they plan to execute

## Maintenance Rule

Whenever a backlog item is added, split, merged, or closed in a way that
changes the planning shape materially, this map should be updated.
