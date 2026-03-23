# ADR-0009: Native IPC ABI Stability And Fallback

- Status: accepted
- Date: 2026-03-23

## Context

native IPC は local latency を下げられますが、
host-specific で不安定な integration story に project を閉じ込めてはいけません。

## Decision

- IPC acceleration は optional に保つ
- UDP fallback を first-class path として維持する
- host-specific integration の前に stable で versioned な C ABI を定義する
- shared memory の ownership と lifecycle を explicit にする

## Consequences

- UE5 / TouchDesigner integration を core の再定義なしで進めやすくなる
- local acceleration を available にしつつ mandatory にはしない
- ABI change には deliberate な versioning discipline が必要になる

## Rejected Alternatives

- mandatory shared-memory transport
- stable common ABI のない engine-specific native interface

## Affected Documents

- [Native IPC ABI Note](../../ja/native-ipc-abi-note.md)
- [C ABI Reference Header And Error-Code Catalog](../../ja/c-abi-reference-header-and-error-code-catalog.md)
- [Phase 05](../../../concepts/ja/phase-05-native-integration.md)
