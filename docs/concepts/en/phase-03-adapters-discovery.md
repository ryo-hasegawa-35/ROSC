# Phase 03: Adapters And Discovery

## Goal

Expand the broker into a multi-protocol hub without compromising raw OSC
compatibility.

## Deliverables

- WebSocket / JSON adapter
- MQTT adapter
- Adapter SDK interfaces
- Service metadata model inspired by OSC 1.1 stream metadata:
  - version
  - framing
  - URI
  - supported type tags
- mDNS / DNS-SD discovery
- Preset device and application profiles
- Stream transport support:
  - TCP compatibility framing
  - SLIP framing for stream-oriented transports
- Adapter health and reconnection management

## Design Rules

- Adapters may enrich semantics, but the core message model remains transport
  neutral.
- No adapter is allowed to weaken the correctness of raw OSC handling.
- Discovery should improve UX, but manual static configuration must always
  remain available.

## Why This Phase Comes After Observability

Once multiple protocols are involved, troubleshooting becomes harder. The system
needs visibility before the protocol surface area expands.

## Non-Goals

- No mandatory auth for all endpoints
- No UE5 shared memory yet
- No Wasm user filter runtime yet

## Exit Criteria

- A browser client can observe or control the broker through WebSocket.
- An MQTT-connected device can exchange translated messages through the hub.
- Devices and services can be discovered automatically on the local network.
- Discovery failures do not block manual operation.

## Rough Effort

160-260 hours

## Value

This phase is where the project graduates from "excellent OSC router" to
"message broker for realtime media systems."
