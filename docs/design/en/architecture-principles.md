# Architecture Principles

## 1. Backward Compatibility First

The platform should be "strict on output, tolerant on input."

- Output mode:
  - By default, emit standards-aligned OSC 1.0 packets over UDP.
  - Preserve original address strings and argument ordering.
  - Never force authentication, discovery, metadata, or schema envelopes into
    the raw OSC payload for legacy peers.
- Input mode:
  - Accept valid OSC 1.0 messages and bundles exactly as specified.
  - Be robust to older senders that omit the OSC type tag string.
  - Support optional 1.1-oriented behavior only when explicitly enabled.

Compatibility policy:

- Raw OSC 1.0 over UDP is the immutable baseline.
- 1.1-inspired additions are opt-in features or parallel transports.
- Any proprietary enhancement must be negotiable or isolated behind an adapter.

## 2. Treat OSC As An Encoding, Not The Whole Protocol

The 1.1 material is useful because it reframes OSC as a content format.
Therefore:

- Routing, security, discovery, schemas, dashboards, and permissions belong in
  higher layers.
- The core broker should normalize messages into an internal typed event model.
- Adapters should translate between external transport semantics and this
  internal model.

This keeps the implementation faithful to legacy OSC while still enabling
modern features.

## 3. Proposed Plugin Strategy

The user suggestion of an openFrameworks-like "install only what you need"
model is strong and should be adopted, but in layers rather than with one
plugin mechanism.

### Layer A: Compile-time feature presets

Use Cargo features and release profiles for bundled capabilities:

- `core-osc`
- `dashboard`
- `discovery-mdns`
- `adapter-websocket`
- `adapter-mqtt`
- `filters-wasm`
- `integration-ue5`
- `integration-touchdesigner`
- `sync-ableton-link`

This gives lean binaries and simple packaging.

### Layer B: Runtime Wasm filter plugins

Use Wasm for user-authored packet transforms:

- smoothing
- scaling
- remapping
- sensor cleanup
- custom trigger logic

Why Wasm:

- cross-platform
- sandboxable
- hot-reload friendly
- no Rust ABI instability

### Layer C: External adapter plugins

For larger extensions, define a plugin protocol over IPC or localhost transport.
These plugins run out of process and communicate with the broker using a stable
API.

Good candidates:

- proprietary hardware bridges
- cloud connectors
- custom control panels
- site-specific business logic

This is safer than loading native Rust dynamic libraries directly.

### Layer D: Product generator / distribution builder

To mirror the openFrameworks project generator idea, create a "distribution
builder" that assembles a targeted package:

- OSC-only build
- exhibition build
- browser-control build
- UE5 workstation build
- TouchDesigner kiosk build

This should be a packaging layer, not the only extensibility layer.

## 4. Why Zero-Trust Can Clash With OSC Culture

Zero-trust itself is not a bad idea. The friction comes from the way OSC is
commonly used in the field.

Reasons for friction:

- OSC culture assumes fire-and-forget interoperability, especially over UDP.
- Many existing tools have no concept of auth handshake, session state, or
  token refresh.
- The OSC spec does not define standard security semantics, capability
  negotiation, or identity exchange.
- Show environments often optimize for "change one IP and go live now," not
  for pre-registered credentials.
- Injecting auth fields into raw message payloads would break compatibility if
  not carefully isolated.

Recommended approach:

- Keep raw OSC fully usable with no security overlay.
- Add zero-trust as an optional transport or namespace gateway.
- Support project IDs, signed envelopes, ACLs, and rate limits in the broker or
  adapter layer.
- For legacy endpoints, terminate secure traffic at the broker and forward
  plain compatible OSC downstream.

This preserves compatibility while still making hostile or messy shared
networks survivable.

## 5. Address Pattern Policy

Support should be split into compatibility modes:

- `osc1_0_strict`
  - `?`, `*`, `[]`, `{foo,bar}`
- `osc1_0_legacy_tolerant`
  - accepts missing type tag strings
  - address-based routing stays available
  - argument-aware transforms require explicit route support
- `osc1_1_extended`
  - add `//` path-traversing wildcard

Reason:

- 1.0 behavior is the safest default for compatibility.
- The 1.1 paper notes that `//` clarifies behavior that was previously
  ambiguous, so it should be opt-in per route or namespace.

## 6. Transport Policy

Transport support should be additive:

- UDP datagram mode as the default and most important path.
- TCP stream support with OSC 1.0 size-prefix framing for compatibility mode.
- SLIP framing support for stream-oriented transports to align with 1.1
  recommendations.
- Serial, USB, and IPC transports should sit behind adapters, not distort the
  core message model.

## 7. Cross-Platform Policy

Windows, macOS, and Linux are realistic targets.

What is straightforward in Rust:

- parser / encoder
- routing core
- UDP / TCP / WebSocket / MQTT
- Wasm runtime
- CLI
- most of the dashboard backend

What requires extra platform-specific work:

- mDNS / DNS-SD behavior differences
- shared memory IPC details
- installer packaging and code signing
- system service integration
- UI shell integration
- Ableton Link packaging and native dependency management

Conclusion:

- Cross-platform is practical.
- Cross-platform release engineering is a real project phase, not a checkbox.
