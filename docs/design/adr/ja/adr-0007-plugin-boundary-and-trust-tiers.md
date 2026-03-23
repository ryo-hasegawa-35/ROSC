# ADR-0007: Plugin Boundary And Trust Tiers

- Status: accepted
- Date: 2026-03-23

## Context

platform には extensibility が必要ですが、unrestricted plugin access は
broker の safety と determinism を弱めます。

## Decision

- built-in、Wasm、external-process extension に trust tier を定義する
- broker-owned safety semantics は plugin に委ねない
- unrestricted internal access ではなく stable plugin / adapter boundary を公開する
- convenience より containment を優先する

## Consequences

- broker を shared heap にせず extensibility を確保できる
- plugin failure を isolate しやすくなる
- 一部 integration は internal access ではなく SDK support が必要になる

## Rejected Alternatives

- unrestricted in-process plugin access
- plugin に security や routing invariant を持たせること

## Affected Documents

- [Plugin Boundary Note](../../ja/plugin-boundary-note.md)
- [Adapter SDK API Reference](../../ja/adapter-sdk-api-reference.md)
- [Security Overlay Model](../../ja/security-overlay-model.md)
