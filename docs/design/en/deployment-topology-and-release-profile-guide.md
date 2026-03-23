# Deployment Topology And Release Profile Guide

## Purpose

This document defines the major deployment shapes the product should support and
the release profiles that should be built for them.

The goal is to avoid shipping one bloated binary for every use case while still
keeping the system coherent.

## Design Goals

- make common deployment shapes explicit
- align package contents with real operator needs
- keep compatibility-first profiles available
- preserve safe fallback paths

## Topology Levels

### Topology A: Localhost Sidecar

Use when:

- one machine runs UE5, TouchDesigner, or another OSC tool
- the broker is inserted without changing project logic

Characteristics:

- localhost transports dominate
- lowest-friction adoption path
- ideal first deployment model

### Topology B: Single Workstation Hub

Use when:

- one machine routes among multiple local and network peers

Characteristics:

- mixed local and network transports
- dashboard often enabled
- light discovery useful

### Topology C: Dual-Machine Show Pair

Use when:

- one machine runs creative software
- another machine handles routing, observability, or operator control

Characteristics:

- explicit network boundary
- stronger health visibility needed
- clean route separation valuable

### Topology D: Segmented Installation Network

Use when:

- multiple devices, sensors, operator consoles, and media nodes cooperate

Characteristics:

- discovery and service metadata matter more
- security scope may matter more
- route and namespace management become central

### Topology E: Active / Standby Pair

Use when:

- continuity matters more than minimal complexity

Characteristics:

- replicated config and selected state
- explicit failover handling

### Topology F: Federated Brokers

Use when:

- multiple broker nodes intentionally share selected traffic or state across
  sites or network segments

## Release Profiles

### `core-osc`

Contents:

- OSC routing core
- compatibility modes
- metrics basics

Use when:

- the smallest strong pipe is the goal

### `ops-console`

Contents:

- core-osc
- dashboard
- capture and replay
- operator recovery tools

Use when:

- visibility and recovery matter most

### `browser-control`

Contents:

- ops-console
- WebSocket / JSON adapter

Use when:

- browser-facing monitoring or control surfaces are required

### `ue5-workstation`

Contents:

- ops-console
- localhost performance presets
- optional native IPC pieces where mature

Use when:

- UE5-centered local workflows dominate

### `touchdesigner-kiosk`

Contents:

- ops-console
- high-rate stream tuning presets
- strong recovery defaults

Use when:

- TouchDesigner-centric sensor or show work dominates

### `secure-installation`

Contents:

- ops-console
- security overlay
- stronger audit defaults
- controlled discovery profile

Use when:

- shared or semi-hostile networks are involved

### `lab-dev`

Contents:

- broad feature set
- diagnostics-heavy defaults
- experimental capabilities visible

Use when:

- development, benchmarking, and exploration are the priority

## Profile Rules

- every profile must state what it excludes
- every profile must preserve a compatibility-first fallback story
- risky experimental features should not appear in production-first profiles by
  accident

## Operator Guidance

Each profile should document:

- intended use case
- included adapters
- included observability features
- security posture
- recommended topology

## Upgrade And Rollback

Release guidance should define:

- which profiles are upgrade-compatible
- how config migration is handled
- what the rollback path is

## Non-Negotiable Invariants

- release profiles must not hide compatibility requirements
- topology guidance must remain simpler than the architecture itself
- every advanced profile needs a clear fallback path
- the smallest useful compatible deployment remains first-class
