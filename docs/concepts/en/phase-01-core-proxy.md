# Phase 01: Core Proxy

## Goal

Ship a production-credible local OSC proxy and routing engine that already
solves the main performance pain point.

## Deliverables

- Multi-port UDP ingress
- Localhost proxy workflow for UE5, TouchDesigner, and general OSC apps
- Lock-conscious or lock-minimized ingress queueing
- Address-based routing engine
- Fan-out to multiple destinations without head-of-line blocking
- Independent egress tasks per destination or destination group
- Strict serializer for outbound OSC
- Configurable route rules:
  - forward
  - drop
  - duplicate
  - rename address
  - static transform
- Metrics for:
  - packets in / out
  - drops
  - queue depth
  - route hit counts
  - per-destination send latency

## Compatibility Requirements

- UDP OSC 1.0 is the default transport mode.
- The parser remains tolerant of missing type tag strings from older senders.
- Address pattern matching defaults to 1.0 semantics.
- 1.1-only behavior is disabled by default.

## Engineering Notes

- The routing core should operate on an internal normalized packet view.
- Parsing and routing should be separable from transport handling for testability.
- Backpressure behavior must be explicit, not accidental.
- The system must prefer dropping or isolating bad routes over stalling the bus.

## Non-Goals

- No browser dashboard beyond simple metrics endpoints
- No MQTT
- No WebSocket control plane
- No native plugin integration

## Exit Criteria

- The broker can replace direct localhost OSC paths in a real application chain.
- One slow destination cannot stall unrelated destinations.
- Packet loss behavior under overload is measurable and intentional.
- A stress test with bursty sensor traffic remains stable over long runs.

## Rough Effort

120-200 hours

## Value

This is the first version that has standalone user value. Even without advanced
features, it can already be a "stronger pipe" than current scripting-based
routers.
