# ADR-0008: Security Overlay Is Additive

- Status: accepted
- Date: 2026-03-23

## Context

project には hostile / noisy network でも安全性が必要ですが、
raw OSC interoperability は壊したくありません。

## Decision

- source verification と project scope enforcement は broker edge で行う
- raw OSC payload semantics は変更しない
- legacy bridge は explicit かつ observable に扱う
- security は base OSC format の mutation ではなく overlay として載せる

## Consequences

- 既存 OSC peer も explicit な broker policy を通して相互運用できる
- security behavior が payload hack ではなく visible になる
- operator は verified path と bridged path を分けて考える必要がある

## Rejected Alternatives

- raw OSC payload に authentication を埋め込むこと
- すべての legacy OSC peer に security handshake を要求すること

## Affected Documents

- [Security Overlay Model](../../ja/security-overlay-model.md)
- [Phase 06](../../../concepts/ja/phase-06-security-sync-release.md)
- [Transport And Adapter Contract](../../ja/transport-and-adapter-contract.md)
