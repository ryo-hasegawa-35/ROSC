# Internal Packet And Metadata Model

## Purpose

This document defines how the broker should represent packets internally while
preserving raw OSC compatibility and enabling higher-level behavior such as
routing, observability, recovery, and security.

The model should satisfy two competing needs:

- preserve the original packet faithfully
- provide a normalized internal view for safe processing

## Design Goals

- Preserve raw OSC bytes so pass-through behavior stays possible.
- Avoid needless copies in the hot path.
- Keep routing metadata outside the OSC payload.
- Support strict, tolerant, and extended compatibility modes.
- Distinguish "inspectable" packets from "opaque but forwardable" packets.
- Make transform and replay lineage explicit.

## Core Representation Layers

The broker should treat packet handling as layered views over one immutable
packet record.

### Layer 1: Raw Packet Record

This is the canonical ingress artifact.

Suggested fields:

- `packet_id`
- `raw_bytes`
- `transport`
- `source_endpoint`
- `received_at`
- `compatibility_mode`
- `raw_size`

Rules:

- `raw_bytes` are immutable after ingress.
- Every downstream representation refers back to this record.
- Replay and forensics should always be able to recover the exact original
  packet bytes when retention is enabled.

### Layer 2: Parse Result

This records what the broker understood about the packet.

Suggested states:

- `WellFormedMessage`
- `WellFormedBundle`
- `LegacyUntypedMessage`
- `MalformedPacket`
- `WellFormedButOpaque`

Interpretation:

- `LegacyUntypedMessage` means the packet is accepted in tolerant mode even
  though the type tag string is missing.
- `WellFormedButOpaque` means the packet structure is valid enough to forward,
  but one or more payload details are not safely inspectable by the broker.

### Layer 3: Normalized View

This is the typed internal view used by routing and transforms when safe.

Possible normalized structures:

- `MessageView`
- `BundleView`
- `OpaqueView`

`MessageView` should expose:

- address
- type tag source
- argument list
- argument spans or owned decoded values
- optional original byte span references

`BundleView` should expose:

- timetag
- ordered element list
- nested message or bundle views

`OpaqueView` should expose:

- enough information to forward or log the packet
- capability flags indicating that deep inspection or transform is unsafe

## Compatibility Modes

These names should be used consistently across documents.

### `osc1_0_strict`

- requires a valid OSC 1.0 message or bundle
- requires an explicit type tag string
- supports 1.0 pattern syntax only
- unsupported type tags are not considered transformable

### `osc1_0_legacy_tolerant`

- accepts older messages that omit the type tag string
- preserves them as legacy packets
- allows routing by address
- forbids argument-aware transforms unless a route explicitly opts into a
  decoder for that legacy stream

### `osc1_1_extended`

- starts from the 1.0 baseline
- may enable `//` path traversal wildcard
- may enable extended type support and metadata-aware transports
- still preserves raw packet bytes and additive compatibility behavior

## Packet Capability Flags

Every parsed packet should carry capability flags so the broker can avoid unsafe
processing.

Suggested flags:

- `forwardable`
- `inspectable_address`
- `inspectable_arguments`
- `transformable`
- `cacheable_candidate`
- `replayable`
- `security_checked`

Example:

- a legacy message without type tags may be `forwardable` and
  `inspectable_address`, but not `inspectable_arguments`

## Argument Model

The internal argument model should separate known decoded values from opaque
payload spans.

Suggested categories:

- required OSC 1.0 value kinds
- optional or extended OSC value kinds
- legacy untyped byte region
- unknown tagged argument

Recommended decoded value set:

- `Int32`
- `Float32`
- `String`
- `Blob`
- `Int64`
- `Timetag`
- `Double64`
- `Symbol`
- `Char`
- `Rgba`
- `Midi4`
- `True`
- `False`
- `Nil`
- `Impulse`
- `Array`
- `UnknownTagged`

Policy:

- Unknown tagged arguments must never silently coerce into a known type.
- Unknown tagged arguments may remain forwardable if the raw packet is intact.
- Transform logic should refuse packets with non-transformable arguments.

## Legacy Untyped Messages

Older OSC senders may omit the type tag string. The broker should handle these
defensively.

Recommended policy:

- preserve raw bytes
- parse address if safely possible
- do not pretend argument types are known
- treat argument payload as opaque
- allow address-based routing and byte-exact forwarding
- disallow value-aware transforms unless a route defines a custom decoder

This keeps tolerant mode honest instead of relying on risky heuristics.

## Metadata Model

Metadata should be additive and external to the OSC payload.

### Ingress Metadata

- `source_endpoint`
- `transport`
- `received_at`
- `interface_id`
- `source_identity` if security mode applies
- `compatibility_mode`
- `parse_status`

### Routing Metadata

- `route_matches`
- `qos_class`
- `priority`
- `drop_preference`
- `cache_policy`
- `security_scope`

### Lineage Metadata

- `parent_packet_id`
- `derived_from_transform`
- `replay_session_id`
- `capture_session_id`

### Timing Metadata

- `source_timetag` when present
- `ingress_observed_at`
- `routed_at`
- `egress_enqueued_at`
- `egress_sent_at`

### Diagnostics Metadata

- `correlation_id`
- `warning_flags`
- `drop_reason`
- `quarantine_reason`

## Ownership And Memory Policy

The broker should prefer immutable shared ownership for packet bytes and
borrowed views for decoding.

Recommended policy:

- ingress allocates one immutable packet buffer
- parse views borrow from that buffer where practical
- decoded values are materialized only when needed
- transforms create a new packet record with lineage metadata
- diagnostics and replay refer to packet records, not ad hoc copied blobs

## Transform Model

Transforms should never mutate the original packet record in place.

Transform rules:

- input packet remains immutable
- transform emits a derived packet
- derived packet carries `parent_packet_id`
- original raw bytes remain available for forensics
- failed transforms cannot corrupt the parent packet

## Security Model Interaction

Security information belongs in metadata and broker policy, not in the raw OSC
payload of legacy traffic.

The internal model should therefore support:

- authenticated source identity
- verified or unverified ingress state
- project scope
- route authorization decision

This keeps enhanced security additive.

## Recovery Model Interaction

The internal packet model should support recovery features without making the
hot path fragile.

Important interactions:

- cache entries refer to normalized values only when they are safely decoded
- replay may use raw bytes for byte-exact resend
- rehydration may use normalized state snapshots for selected routes
- packet lineage must distinguish live traffic from replayed traffic

## Non-Negotiable Invariants

- raw ingress bytes remain available while retention policy allows
- normalized views never invent certainty that the parser does not have
- metadata never changes packet payload semantics silently
- security and diagnostics remain additive
- transforms operate on derived packets, not in-place mutation

## Open Follow-On Documents

This model should be followed by:

- compatibility matrix
- route configuration grammar
- fault model and overload behavior
- recovery model and cache semantics
