# Compatibility Matrix

## Purpose

This document defines what the broker promises to accept, inspect, transform,
forward, cache, and replay across its compatibility modes and transports.

The goal is to preserve backward compatibility with ordinary OSC practice while
making enhanced behavior explicit instead of accidental.

## Interpretation Baseline

- OSC 1.0 is the compatibility baseline.
- Older senders that omit the type tag string are supported in tolerant mode.
- OSC 1.1-oriented behavior is treated as additive and opt-in.
- Security, discovery, schema, and recovery behavior are layered on top of raw
  OSC compatibility.

## Compatibility Modes

### `osc1_0_strict`

Use when:

- standards correctness matters most
- packets should be safely inspectable before advanced handling

Key behavior:

- requires valid OSC 1.0 message or bundle structure
- requires explicit type tag string
- uses OSC 1.0 address pattern behavior only
- treats unsupported tagged values as non-transformable

### `osc1_0_legacy_tolerant`

Use when:

- integrating older implementations
- preserving compatibility matters more than deep inspection

Key behavior:

- accepts older messages that omit type tag string
- allows address-based routing
- preserves opaque payloads for forwarding and replay
- disallows argument-aware transforms unless explicitly decoded by route policy

### `osc1_1_extended`

Use when:

- both sides intentionally opt into extended behavior
- richer pattern matching, types, or metadata are useful

Key behavior:

- preserves OSC 1.0 baseline compatibility
- may enable `//` path traversal wildcard
- may enable richer type support and stream metadata
- still does not force enhanced behavior into legacy raw OSC payloads

## Transport Matrix

| Transport | Status | Notes |
| --- | --- | --- |
| UDP datagram | First-class | Default compatibility path |
| TCP size-prefix framing | Supported | Compatibility mode for stream transport |
| SLIP framing | Supported | Additive stream framing aligned with 1.1 guidance |
| WebSocket / JSON adapter | Additive | Adapter path, not raw OSC baseline |
| MQTT adapter | Additive | Adapter path, not raw OSC baseline |
| Shared memory IPC | Additive | Local acceleration path, never mandatory |

## Address Pattern Matrix

| Feature | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| `/foo/bar` literal path | Yes | Yes | Yes |
| `?` single-character wildcard | Yes | Yes | Yes |
| `*` wildcard within part | Yes | Yes | Yes |
| `[]` character set | Yes | Yes | Yes |
| `{foo,bar}` alternation | Yes | Yes | Yes |
| `//` path traversal wildcard | No | No | Optional |

## Message Structure Matrix

| Packet Shape | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| Well-formed message with type tags | Accept | Accept | Accept |
| Well-formed bundle | Accept | Accept | Accept |
| Message missing type tags | Reject | Accept as legacy opaque | Optional, same as tolerant unless stricter route policy |
| Malformed packet | Reject | Reject or quarantine | Reject or quarantine |
| Nested bundles | Accept | Accept | Accept |

## Value Support Matrix

The broker distinguishes four levels:

- `Accept`
- `Inspect`
- `Transform`
- `Cache`

### Required OSC 1.0 Types

| Type | Accept | Inspect | Transform | Cache |
| --- | --- | --- | --- | --- |
| `i` int32 | Yes | Yes | Yes | Route policy dependent |
| `f` float32 | Yes | Yes | Yes | Route policy dependent |
| `s` string | Yes | Yes | Yes | Route policy dependent |
| `b` blob | Yes | Yes | Cautious | Usually no by default |

### Legacy / Optional / Extended Types

| Type | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| `h` int64 | Accept, inspect when implemented | Address-route only if opaque | Accept, inspect |
| `t` timetag argument | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `d` double | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `S` symbol | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `c` char | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `r` rgba | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `m` MIDI | Accept, inspect when implemented | Opaque if not safely decoded | Accept, inspect |
| `T`, `F`, `N`, `I` | Accept if implemented | Opaque if not safely decoded | Accept, inspect |
| `[` `]` arrays | Accept if implemented | Opaque if not safely decoded | Accept, inspect |
| Unknown tagged value | Forward if packet is intact, no transform | Forward if packet is intact | Forward if packet is intact, no transform unless route-specific decoder exists |

## Capability Matrix

| Capability | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| Byte-exact forwarding | Yes | Yes | Yes |
| Address-based routing | Yes | Yes | Yes |
| Argument-aware routing | Yes | Limited | Yes |
| Value transform | Yes when safely decoded | No by default | Yes when safely decoded |
| Stateful cache candidate | Yes by route policy | No by default for opaque legacy payloads | Yes by route policy |
| Replay from raw bytes | Yes | Yes | Yes |
| Rehydrate from decoded state | Yes when cacheable | No by default for opaque legacy payloads | Yes when cacheable |

## Security Overlay Matrix

| Behavior | Raw OSC compatibility routes | Secure overlay routes |
| --- | --- | --- |
| Authentication required | No | Yes |
| Authorization enforced | Optional broker policy | Yes |
| Payload changed for legacy peer | No | No, broker terminates secure envelope before forwarding |
| Fail on verification error | Route drop policy | Fail closed |

## Discovery And Metadata Matrix

| Feature | Compatibility baseline | Additive support |
| --- | --- | --- |
| Static manual endpoint config | Yes | Yes |
| DNS-SD / mDNS discovery | Optional | Yes |
| Stream metadata publication | Optional | Yes |
| Service URI metadata | Optional | Yes |

## Replay And Recovery Matrix

| Behavior | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| Byte-exact replay | Yes | Yes | Yes |
| State rehydrate from cache | Yes when decoded state exists | Usually no by default | Yes when decoded state exists |
| Late joiner sync | Yes by route policy | Only with explicit route decoder or raw resend policy | Yes by route policy |

## Examples

### Example A: Standard UE5 Control Message

Packet:

- address `/ue5/camera/fov`
- type tags `,f`
- float argument

Result:

- accepted in all three modes
- inspectable
- transformable
- cacheable if route policy allows

### Example B: Legacy Packet With No Type Tags

Packet:

- address `/legacy/position`
- no type tag string

Result:

- rejected in `osc1_0_strict`
- accepted as `LegacyUntypedMessage` in `osc1_0_legacy_tolerant`
- forwardable and replayable
- not argument-transformable unless a route decoder is declared

### Example C: Extended Path Traversal

Pattern:

- `//spherical`

Result:

- unavailable in `osc1_0_strict`
- unavailable in `osc1_0_legacy_tolerant`
- optional in `osc1_1_extended`

## Non-Negotiable Compatibility Rules

- UDP raw OSC 1.0 remains first-class.
- Type-tag-less legacy packets are tolerated only in the tolerant path.
- Enhanced behavior must never silently rewrite raw OSC expectations.
- Unknown tagged payloads may be forwardable without becoming transformable.
- Security overlays must not break downstream legacy interoperability.
