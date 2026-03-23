# Profile-Specific Operator Guides

## Purpose

This document translates architecture into operator behavior by deployment
profile.

The same broker core can be safe or unsafe depending on how it is operated. The
goal here is to define what operators should prioritize in each major profile
before incidents happen.

Related documents:

- [Deployment Topology And Release Profile Guide](./deployment-topology-and-release-profile-guide.md)
- [Dashboard Interaction Spec And Screen Inventory](./dashboard-interaction-spec-and-screen-inventory.md)
- [Operator Workflow And Recovery Playbook](./operator-workflow-and-recovery-playbook.md)
- [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

## Questions Every Operator Should Be Able To Answer

- Which traffic classes are mission-critical in this profile?
- Which destinations may be sacrificed first under pressure?
- What is the safe-mode path?
- What evidence should be captured if recovery is needed?
- Which actions are automatic and which require operator confirmation?

## Guide 1: `core-osc` On A Localhost Sidecar

Use when:

- one workstation runs the creative tool and the broker
- adoption friction must stay minimal
- raw OSC compatibility is the top priority

Primary operator concern:

- added latency on the main control path

Healthy signals:

- low steady queue depth
- near-zero parse failures
- no unexpected breaker events
- stable localhost latency under load

First actions during incident:

1. check ingress parse failures and destination breaker state
2. disable non-essential taps or dashboard subscribers
3. switch to the minimal safe profile if observability overhead is suspected

Safe mode:

- core routing only
- no heavy capture
- minimal metrics

Do not optimize first:

- discovery
- schema tooling
- dashboard cosmetics

## Guide 2: `ops-console` On A Single Workstation Hub

Use when:

- one machine routes among multiple local and network peers
- operators rely on the browser console during rehearsals or shows

Primary operator concern:

- keeping diagnostics helpful without making them part of the outage

Healthy signals:

- control routes remain green even if telemetry routes degrade
- dashboard tap latency stays below operator tolerance
- route diff events are explainable and auditable

First actions during incident:

1. identify whether the unhealthy state is ingress-side or destination-side
2. reduce diagnostics level on high-rate routes if needed
3. isolate slow consumers before touching critical control routes

Safe mode:

- disable non-essential screens
- keep route health and breaker state visible
- keep replay manual

## Guide 3: `ue5-workstation` For Local Or Dual-Machine Show Work

Use when:

- UE5 is the dominant runtime
- camera, scene, and cue traffic must remain tight and predictable

Primary operator concern:

- preserving timing and state continuity during engine restart or hot reload

Healthy signals:

- stateful control cache remains current
- rehydrate operations are fast and bounded
- IPC or localhost routes show no unexplained spikes

First actions during incident:

1. verify whether the issue is engine-side, broker-side, or IPC boundary-side
2. confirm cache freshness before rehydrate
3. prefer controlled rehydrate over blind replay

Safe mode:

- fall back to UDP path if IPC is suspected
- disable experimental transforms
- keep only state recovery features that are already validated

## Guide 4: `touchdesigner-kiosk` For High-Rate Sensor Work

Use when:

- TouchDesigner or a similar environment consumes dense sensor streams
- bounded loss is acceptable, but instability is not

Primary operator concern:

- protecting control traffic from sensor storms

Healthy signals:

- sensor routes may sample or drop within policy
- critical control routes stay stable while sensor routes degrade explicitly
- capture remains bounded

First actions during incident:

1. confirm which traffic class is affected
2. reduce optional monitors or browser subscriptions
3. check whether the route should move from detailed to minimal metrics

Safe mode:

- retain critical control and stateful routes
- degrade sensor observability first
- quarantine malformed sources aggressively

## Guide 5: `secure-installation` On A Segmented Network

Use when:

- the network is shared, semi-hostile, or operationally noisy
- source verification and auditability matter

Primary operator concern:

- keeping trust boundaries explicit without breaking legacy peers

Healthy signals:

- rejected-source counts are explainable
- verified and legacy bridges are clearly separated
- discovery state matches approved topology

First actions during incident:

1. distinguish authentication failures from routing failures
2. confirm whether the source is unknown, stale, or revoked
3. preserve audit evidence before broad rollback

Safe mode:

- keep secure ingress enforcement
- disable optional discovery
- preserve compatibility-only bridges that are already approved

## Guide 6: Active / Standby Pair

Use when:

- continuity matters more than minimal complexity
- configuration and selected recovery state are replicated

Primary operator concern:

- avoiding ambiguous ownership during failover

Healthy signals:

- primary and standby identity are unambiguous
- replication lag stays inside declared tolerance
- failover state transitions are explicit events

First actions during incident:

1. determine whether the primary is unhealthy or merely partitioned
2. confirm standby freshness before promoting it
3. capture a failover reason code and operator note

Safe mode:

- freeze risky config changes
- prefer manual failover over automatic promotion when identity is unclear
- keep replay disabled unless explicitly validated for the namespace

## Guide 7: Federated Brokers

Use when:

- multiple brokers intentionally exchange selected traffic or state
- sites or segments must stay partly autonomous

Primary operator concern:

- preventing one site's disorder from becoming everyone's disorder

Healthy signals:

- replication scope is selective rather than universal
- remote lag and breaker states are visible per peer
- local safety policies survive remote degradation

First actions during incident:

1. identify whether the problem is local, remote, or on the federation link
2. narrow replication scope before widening local degradation
3. keep local critical traffic authoritative if cross-site confidence is low

Safe mode:

- local-only routing for critical namespaces
- suspend non-critical federation links
- keep forensic evidence bounded and tagged by broker identity

## Cross-Profile Invariants

- Critical control must never share the same failure budget as optional
  telemetry.
- Safe mode must preserve operator comprehension, not only system liveness.
- Recovery steps should prefer rehydrate over replay whenever the namespace is
  stateful.
- Incident evidence must include route, destination, broker identity, and
  compatibility mode.

## Hand-Off Rule

If a profile-specific runbook conflicts with the normative design documents, the
operator guide should be corrected. Profiles can tune behavior, but they should
not redefine core safety semantics.
