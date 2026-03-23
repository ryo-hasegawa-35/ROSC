# ADR-0002: Dual Packet Representation

- Status: accepted
- Date: 2026-03-23

## Context

broker には、compatibility と forensic のための byte-level fidelity と、
core routing engine のための normalized internal view の両方が必要です。

## Decision

- raw packet byte と ingress metadata を保持する
- routing / operator tooling 用に normalized packet representation を導出する
- parse failure は partial な normalized state を作らず explicit に扱う
- unknown tag は compatibility mode に従って扱い、silent mutation をしない

## Consequences

- normalized-only 設計よりメモリ消費は増える
- replay、audit、conformance tooling が強くなる
- routing code を byte-level parsing detail から分離しやすくなる

## Rejected Alternatives

- normalized-only storage
- raw-only の opaque packet handling

## Affected Documents

- [Internal Packet And Metadata Model](../../ja/internal-packet-and-metadata-model.md)
- [Compatibility Matrix](../../ja/compatibility-matrix.md)
- [Recovery Model And Cache Semantics](../../ja/recovery-model-and-cache-semantics.md)
