# Native IPC ABI Note

## 目的

この文書は、UE5 などの host-native integration で使う shared-memory と
C ABI path の境界を定義します。

## 設計目標

- stable な host-facing ABI
- 明確な ownership rule
- Rust internal layout への hidden dependence を作らない
- standard transport への graceful fallback
- cross-platform viability

## Layered Boundary

native integration path は 2 層に分けるべきです。

### Layer 1: Stable C ABI

用途:

- lifecycle
- configuration call
- capability query
- error reporting
- handle management

### Layer 2: IPC Data Path

用途:

- high-throughput な local message movement
- shared memory ring buffer または同等 channel

## ABI Rule

- Rust 固有の memory layout を直接 expose しない
- opaque handle を使う
- explicit size field を持つ
- explicit version field を持つ
- ownership transfer rule を単純に保つ

## Core Handle Type

想定 opaque handle:

- broker handle
- route handle
- endpoint handle
- shared memory channel handle

## Versioning

ABI が定義すべきもの:

- ABI version
- minimum compatible broker core version
- feature bitset

version mismatch は crash ではなく明確な失敗にするべきです。

## Data Path Contract

IPC data path が定義すべきもの:

- packet frame header
- payload region
- producer / consumer ownership
- sequence number
- overflow signaling
- health counter

## Shared Memory Channel Model

推奨する概念モデル:

- 1 つ以上の ring buffer
- explicit な producer / consumer role
- bounded capacity
- lifecycle と diagnostics 用の separate control channel

## Memory Ownership

ルール:

- shared memory payload ownership は明示する
- host は release 後に pointer を握り続けない
- broker は contract を超えて host-side lifetime を仮定しない
- error path でも resource を安全に解放する

## Session Lifecycle

典型的な流れ:

1. ABI version を問い合わせる
2. broker handle を作る
3. capability を negotiate する
4. IPC channel を作る
5. transport loop を開始する
6. stop して handle を destroy する

## Fallback Policy

native IPC が unavailable または unhealthy なら:

- localhost OSC または他の supported local transport へ fallback
- logical route behavior は維持
- degraded state を明確に報告

## Error Model

error は次で表すべきです。

- explicit error code
- 必要に応じた human-readable message
- recoverable / unrecoverable classification

## Security And Scope

local IPC でも次を保つべきです。

- project scope
- route authorization
- local host integration の clear identity

## Observability

ABI path が出すべきもの:

- channel occupancy
- overflow count
- reconnect count
- fallback transition
- local latency metric

## 非交渉の不変条件

- native IPC が唯一の usable path になってはならない
- 内部機能の露出より ABI stability を優先する
- shared memory failure で broker state を壊さない
- fallback でも compatibility expectation を守る
