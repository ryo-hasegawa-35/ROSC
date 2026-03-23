# Glossary

## Purpose

This glossary keeps terminology consistent across concept documents, design
specs, and later implementation work.

## Core Terms

### Broker

The central runtime that receives traffic, normalizes it, routes it, observes
it, and forwards it. The broker is the core system, not merely one adapter.

### Ingress

The receiving side of the broker. Ingress is where packets or messages enter
the system from transports or adapters.

### Egress

The sending side of the broker. Egress is where packets or derived messages
leave the system toward destinations.

### Route

A declarative rule that matches traffic, optionally transforms it, and sends it
to one or more destinations under explicit fault, cache, recovery, and security
policies.

### Destination

A concrete output target for a route. A destination is smaller than a route and
should have its own queueing and health state.

### Adapter

A transport- or protocol-specific boundary module that translates between an
external communication system and the broker core model.

### Service

A logical capability exposed by a broker or adapter, such as an OSC receiver, a
WebSocket control endpoint, or a discovery-visible bridge.

### Transport

The delivery mechanism or communications substrate, such as UDP, TCP, WebSocket,
MQTT, or shared memory IPC.

## Compatibility Terms

### `osc1_0_strict`

The mode that accepts standards-aligned OSC 1.0 packets with explicit type tag
strings and 1.0 semantics only.

### `osc1_0_legacy_tolerant`

The mode that accepts older packets that omit the type tag string, while
preserving them honestly as limited-inspection traffic.

### `osc1_1_extended`

An additive compatibility mode that starts from the OSC 1.0 baseline and allows
carefully selected extended behavior such as `//` path traversal.

### Legacy Untyped Message

A packet that omits the type tag string and therefore cannot safely support the
same degree of argument-aware processing as a typed message.

### Opaque Packet

A packet that may still be forwardable or replayable, but cannot be safely
inspected or transformed in full by the broker.

## Runtime Terms

### Normalized View

The internal typed representation the broker uses after parsing, when the packet
is safe enough to inspect and reason about.

### Raw Packet Record

The immutable ingress record containing the original bytes and ingress metadata.

### Capability Flag

A marker on a packet or decoded view that tells the broker what operations are
safe, such as forwarding, inspecting, transforming, or caching.

### Packet Lineage

The relationship between original traffic and any derived, transformed, replayed,
or rehydrated traffic.

## Failure Terms

### Fault Domain

The smallest boundary inside which a failure should be contained, such as a
sender, route, destination, adapter, or broker.

### Circuit Breaker

A protective mechanism that opens when failures repeat beyond a threshold, in
order to protect the wider system from continued damage.

### Quarantine

A stronger protective action that isolates a sender or source of repeated bad
traffic for a cooling-off period.

### Overload State

An explicit broker-wide operating state such as `Healthy`, `Pressured`,
`Degraded`, `Emergency`, or `SafeMode`.

### Shed

To deliberately reduce work or traffic handling under pressure so that more
important flows remain healthy.

## Recovery Terms

### Cache

Stored state derived from selected traffic for later rehydrate, recovery, or
inspection.

### Rehydrate

To restore current state, usually from cache or snapshot, without replaying full
historical traffic.

### Replay

To resend historical traffic from capture or journal for debugging, testing, or
controlled reconstruction.

### Late Joiner

A node or client that connects after traffic has already been flowing and may
need state catch-up.

### Warm Restart

A restart that intentionally restores selected runtime state such as caches and
configuration, while still making the restart visible in system history.

## Security Terms

### Security Overlay

An additive layer of identity, scope, verification, and authorization applied by
the broker without requiring raw OSC payload changes for legacy peers.

### Scope

A security or policy boundary such as a project, venue, workstation, or
namespace.

### Verified Source

A sender or peer whose identity has been authenticated by secure transport
context or secure envelope rules.

### Legacy Bridge

The broker behavior that terminates secure ingress and forwards plain compatible
OSC downstream to legacy tools when policy allows it.

## Discovery Terms

### Service Metadata

A transport-neutral description of a discovered or configured service,
including identity, capability, endpoint references, trust, and freshness.

### Trust Level

A classification such as observed, claimed, verified, or operator approved that
describes how much confidence the system has in discovered information.

## Operations Terms

### Safe Mode

A reduced-capability broker startup or runtime mode in which risky optional
features are disabled so the smallest useful compatible system can remain alive.

### Topology View

The dashboard view that shows how ingress, routes, transforms, and destinations
are connected.

### Playbook

A predefined operator response pattern for a class of incidents, such as slow
destination, malformed traffic storm, or restart recovery.

### Release Profile

A packaged product shape such as `core-osc`, `ops-console`, or
`secure-installation`, defining what features are included for a deployment.

### Deployment Topology

A recurring deployment shape such as localhost sidecar, single workstation hub,
active / standby pair, or federated broker network.

## Distributed Terms

### Federation

A mode where multiple brokers intentionally exchange selected traffic or state
while remaining distinct peers.

### High Availability

A mode where another broker is prepared to continue service if the active broker
fails.

### Active / Standby

A high-availability pair in which one broker actively serves traffic while the
other is prepared to take over under defined conditions.

## Tooling Terms

### Schema

An optional typed description of intended message meaning, constraints, and
tooling hints layered on top of raw OSC compatibility.

### Conformance Vector

A known reference input or output used to check whether behavior remains aligned
with specification or agreed compatibility rules.

### Interoperability Suite

A scenario-based validation set that checks whether the product behaves
correctly with real tools, transports, and integration paths.

### Adapter SDK

The supported integration surface used by transport or protocol adapter authors
to connect external systems to the broker safely.
