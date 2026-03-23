# Discovery And Service Metadata Model

## Purpose

This document defines how brokers, adapters, endpoints, and services should be
discovered and described without weakening manual control or backward
compatibility.

Discovery is a convenience layer. It must never become a hidden control plane
that operators cannot reason about.

## Design Goals

- preserve manual configuration as the baseline
- make discovery additive, not mandatory
- describe services in a transport-neutral way
- distinguish discovered facts from trusted facts
- keep discovery results visible and auditable

## Discovery Principles

- discovery may suggest configuration, but must not silently create it
- discovery freshness must be explicit
- identity and trust are separate concerns
- discovery should help the operator, not replace the operator
- legacy OSC endpoints should remain usable even if they publish no metadata

## Discovery Modes

### Manual Only

Use when:

- deterministic operation matters most
- the environment is small or stable
- discovery traffic is undesirable

### Passive Discovery

Use when:

- endpoints announce themselves
- the broker should observe and catalog without active probing

Examples:

- DNS-SD / mDNS advertisement
- broker peer advertisement

### Active Discovery

Use when:

- the broker intentionally probes the environment
- operators want guided setup

Examples:

- mDNS browse queries
- adapter-specific endpoint enumeration

### Registered Discovery

Use when:

- service metadata is provided through a registry or explicit operator input
- environments are larger or more controlled

## Discovery Entities

The model should distinguish the following entities:

### Broker

The runtime instance itself.

### Adapter

A transport- or protocol-specific boundary owned by a broker.

### Service

A logical endpoint capability exposed to the network or local environment.

Examples:

- OSC UDP receiver
- WebSocket control service
- MQTT bridge

### Endpoint

A concrete addressable target such as a host, port, topic, path, session, or
channel.

### Capability

A declared property of a service, such as transport type, framing, supported
patterns, or security requirements.

## Service Metadata Model

Each discovered service should be representable as a transport-neutral record.

Suggested fields:

- `service_id`
- `service_kind`
- `broker_id` where applicable
- `adapter_id` where applicable
- `display_name`
- `protocol_family`
- `transport`
- `framing`
- `version`
- `endpoint_refs`
- `capabilities`
- `scope`
- `security_mode`
- `metadata_source`
- `first_seen_at`
- `last_seen_at`
- `ttl`

## Endpoint Metadata Model

Suggested endpoint fields:

- `endpoint_id`
- `host`
- `port`
- `path_or_topic`
- `interface`
- `locality`
- `session_requirement`
- `identity_requirement`

## Capability Advertisement

Service metadata should be able to describe at least:

- supported protocol family
- supported framing
- compatibility modes supported
- security expectation
- whether discovery is passive or active
- whether operator approval is required before use

## Trust Levels

Discovery results should carry trust classification.

Suggested levels:

- `Observed`
- `Claimed`
- `Verified`
- `OperatorApproved`

Interpretation:

- `Observed`
  - seen on the network, not trusted
- `Claimed`
  - self-advertised capability
- `Verified`
  - identity or capability checked through a trusted mechanism
- `OperatorApproved`
  - explicitly accepted for use

## Freshness And Expiry

Discovery data must not live forever.

Suggested rules:

- every discovered record has `last_seen_at`
- records with TTL expire visibly
- expired records are not silently reused as active configuration
- stale records may remain visible for operator history

## Discovery And Routing

Discovery should not create route behavior by itself.

Allowed interactions:

- suggest destination candidates
- populate labels and metadata in the UI
- help build route configuration

Disallowed interactions:

- silently attach a discovered destination to a live route
- bypass explicit security policy
- redefine compatibility mode implicitly

## Discovery And Security

Discovery and trust must remain separate.

Rules:

- discovered service identity is not the same as verified identity
- security-sensitive routes should require explicit approval or verified trust
- anonymous legacy discovery may still be useful for visibility

## Discovery Failure Modes

The system should tolerate:

- no discovery available
- noisy discovery environments
- stale advertisements
- conflicting advertisements
- spoofed advertisements in insecure environments

Failure handling:

- manual configuration remains available
- stale or conflicting discovery is surfaced visibly
- secure deployment profiles may ignore untrusted discovery results

## Operator Experience

Operators should be able to:

- browse discovered services
- inspect trust and freshness
- approve or reject a discovered service
- convert discovery results into explicit configuration
- see when a discovered service disappears

## Inter-Broker Discovery

Brokers may advertise:

- broker ID
- supported profiles
- replication or federation capability
- health visibility endpoints

But broker discovery must not create federation automatically without explicit
policy.

## Non-Negotiable Invariants

- discovery never replaces manual configuration
- stale discovery must be visible as stale
- discovery trust is never assumed from visibility alone
- discovery must not silently modify live routes
- legacy OSC without discovery metadata remains first-class
