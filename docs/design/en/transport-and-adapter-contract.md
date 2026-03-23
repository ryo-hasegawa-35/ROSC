# Transport And Adapter Contract

## Purpose

This document defines the contract between the broker core and all ingress /
egress adapters.

The contract must let the core stay transport-neutral while still preserving the
behavioral differences that matter for performance, safety, and recovery.

## Design Goals

- one core routing model across transports
- explicit transport capabilities
- bounded failure domains
- additive security and metadata
- no accidental transport-specific behavior leaking into route semantics

## Adapter Roles

Every adapter should play one or both of these roles:

- ingress adapter
- egress adapter

An adapter may support both, but the core should treat them as separate
contracts.

## Core Adapter Contract

Each adapter should declare:

- `adapter_id`
- `adapter_kind`
- `protocol_family`
- `direction`
- `capabilities`
- `state`
- `health`
- `version`

## Ingress Contract

An ingress adapter must deliver:

- immutable raw packet bytes or equivalent canonical payload
- ingress metadata
- source endpoint identity
- transport identity
- receive timestamp

Ingress adapters may additionally deliver:

- authenticated source identity
- discovery metadata
- transport session metadata

Ingress adapters must not:

- mutate broker route configuration
- silently apply packet transforms that are not declared
- bypass broker security policy

## Egress Contract

An egress adapter must accept:

- derived or pass-through packet record
- egress metadata
- destination reference
- send policy hints

Egress adapters must report:

- send success
- send failure reason
- timeout
- disconnect
- backpressure state

## Transport Capability Model

Every adapter should advertise capabilities explicitly.

Suggested capability flags:

- message-oriented
- stream-oriented
- preserves packet boundaries
- supports secure identity
- supports discovery metadata
- supports bidirectional session state
- supports ordered delivery
- supports best-effort delivery
- supports native binary payloads

## Endpoint Model

The broker should normalize endpoints into a transport-neutral reference.

Suggested endpoint fields:

- `endpoint_id`
- `adapter_id`
- `protocol`
- `address`
- `port`
- `path_or_topic`
- `scope`
- `identity_requirement`

Examples:

- UDP host and port
- WebSocket session ID
- MQTT topic
- IPC channel name

## Session Model

Some transports are stateless and some are sessionful.

Stateless examples:

- UDP datagram

Sessionful examples:

- TCP
- WebSocket
- MQTT
- IPC links

The core should expose session state only as metadata and health signals, not
as a hidden route behavior dependency.

## Connection State

For sessionful adapters, use explicit states:

- `Init`
- `Connecting`
- `Ready`
- `Degraded`
- `Disconnected`
- `Recovering`

## Framing Policy

Transport framing must be explicit at the adapter boundary.

Supported framing families:

- raw UDP packet
- size-prefixed stream packet
- SLIP-framed stream packet
- adapter-defined envelope for non-OSC protocols

The route and packet models should never have to guess framing after ingress.

## Metadata Contract

Adapters may attach metadata, but must classify it clearly.

Suggested metadata groups:

- ingress transport metadata
- discovery metadata
- security metadata
- session metadata
- adapter diagnostics

Metadata must remain additive and must not silently change raw OSC payload
semantics.

## Backpressure Contract

Adapters must integrate with the fault model.

Ingress adapters should report:

- receive pressure
- rate limit application
- malformed input counts

Egress adapters should report:

- queue pressure
- send timeout
- destination stall
- retry exhaustion

The broker should never infer backpressure from missing telemetry alone.

## Retry And Delivery Semantics

The adapter contract must not promise more than the transport can guarantee.

Examples:

- UDP egress is best effort
- TCP and WebSocket may preserve order but still fail at the session level
- MQTT delivery semantics depend on configuration and broker behavior

Delivery policy should be an explicit route or destination configuration, not a
hidden property of adapter code.

## Security Interaction

Security may be provided by:

- transport context
- outer secure envelope
- broker policy

Adapters may surface verified identity, but authorization remains a broker
decision.

## Discovery Interaction

Discovery-capable adapters should expose:

- discovered endpoint identity
- freshness / TTL
- capability advertisement
- human-readable labels where available

Discovery results should feed operator UX, not bypass explicit route policy.

## Versioning

The adapter contract should be versioned independently from the core crate
layout.

Every adapter should declare:

- contract version
- adapter version
- supported capability set
- supported framing set

## Observability Requirements

The core should be able to inspect:

- adapter state
- adapter health
- session count where relevant
- ingress rate
- egress success / failure counts
- disconnect count
- reconnect count

## Non-Negotiable Invariants

- route semantics must not depend on undocumented adapter behavior
- adapter failure must remain local
- transport framing must be explicit
- security identity may be provided by the adapter, but authorization stays in
  the broker
- raw OSC compatibility remains preserved at the broker boundary
