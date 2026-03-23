# Operator Workflow And Recovery Playbook

## Purpose

This document describes how operators should interact with the broker during
normal operation, degraded operation, and recovery.

The goal is to make advanced behavior operable under pressure.

## Operator Questions The Product Must Answer

- what is happening right now
- what is unhealthy
- what is being dropped
- what changed
- how do I recover safely

## Normal Workflow

### Startup

Operator should be able to choose:

- normal mode
- safe mode
- recovery mode

### Preflight

Preflight should show:

- listening endpoints
- route count
- disabled routes
- discovery status
- security status
- stale cache warnings

### Live Monitor

Live monitor should show:

- overall state: healthy / pressured / degraded / emergency
- top queue growth
- drop reasons
- breaker events
- destination health
- plugin health

## Incident Playbooks

### Playbook A: Slow Destination

Symptoms:

- rising egress queue
- repeated destination timeout

Actions:

1. inspect destination health
2. confirm breaker state
3. isolate destination if needed
4. verify healthy routes remain stable

### Playbook B: Sensor Flood

Symptoms:

- rising pressure on sensor routes
- control traffic at risk

Actions:

1. inspect traffic classes
2. confirm shedding policy engaged
3. sample or coalesce sensor stream if configured
4. verify critical control remains protected

### Playbook C: Malformed Traffic Storm

Symptoms:

- parse errors
- quarantine triggers

Actions:

1. inspect offending source
2. confirm quarantine or rate limit action
3. verify healthy traffic continues
4. capture sample if needed for later analysis

### Playbook D: Node Restart Recovery

Symptoms:

- downstream node reconnect
- missing state after restart

Actions:

1. inspect route cache policy
2. trigger route or namespace rehydrate
3. verify state restored
4. avoid replay unless debugging is required

### Playbook E: Plugin Failure

Symptoms:

- transform timeout
- plugin disconnect

Actions:

1. inspect plugin status
2. disable plugin if repeated failures occur
3. continue with core routing
4. re-enable only after verification

## Recovery Controls

The UI or operator surface should provide:

- isolate route
- disable destination
- resend latest cached state
- resend snapshot set
- start sandbox replay
- invalidate cache
- enter safe mode
- acknowledge fault

## Safe Mode

Safe mode should:

- disable experimental plugins
- disable optional risky transforms
- preserve core compatible routing
- expose clearly that the broker is running in reduced capability mode

## Replay Safety

Replay should default to:

- sandbox target
- marked lineage
- operator-confirmed scope

Replay should never masquerade as ordinary live traffic.

## Audit And History

Operators should be able to inspect:

- config change history
- who triggered recovery actions
- which routes were isolated
- when safe mode was entered
- what was replayed

## Non-Negotiable Invariants

- operator actions must be visible and auditable
- recovery actions must be clearly distinct from live routing
- safe mode must preserve the smallest useful compatible system
- the product should guide recovery, not require memorized tribal knowledge
