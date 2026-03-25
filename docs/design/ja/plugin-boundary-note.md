# Plugin Boundary Note

## 目的

この文書は、broker core を不安定にせずに extensibility を実現する境界を
定義します。

中心ルール:

- 柔軟性は edge に置く
- data plane は小さく予測可能に保つ

## Extension Layer

このプロジェクトは、信頼度と能力の異なる 4 層の extension を持つべきです。

### Layer 1: Compile-Time Feature

例:

- dashboard
- discovery
- mqtt adapter
- websocket adapter
- ue5 integration

向いている場合:

- product-level capability
- trusted capability
- static packaging が許容できる

### Layer 2: Wasm Transform

例:

- smoothing
- scaling
- remapping
- custom filtering

向いている場合:

- user logic を hot reload したい
- deterministic sandboxing が重要
- packet-level transform で足りる

### Layer 3: External Process Plugin

例:

- proprietary hardware bridge
- cloud connector
- heavyweight analysis module

向いている場合:

- feature が大きい
- failure containment が重要
- in-process ABI より stable IPC contract が望ましい

### Layer 4: Native Host Integration

例:

- UE5 plugin
- TouchDesigner bridge

向いている場合:

- local performance または UX のため deeper integration が必要
- host application に近い機能であることを意図している

## 主たる Plugin Model にすべきでないもの

in-process な native Rust dynamic library loading を主な extension model に
するのは避けるべきです。ABI stability も fault containment も弱いからです。

## Plugin Trust Tier

### Trusted

- product に同梱される
- production profile で有効化可能

### Approved

- operator policy で導入
- monitored かつ bounded

### Experimental

- default で無効
- safe mode で自動的に切られうる

## Wasm Plugin Contract

入力として渡しうるもの:

- safely available な normalized packet view
- limited metadata
- route context

出力として許すもの:

- pass through unchanged
- derived packet を emit
- reason 付きで drop

制約:

- direct network I/O 禁止
- bounded memory
- bounded execution time
- broker state の direct mutation 禁止

## Wasm Hot Path Guardrail

Wasm は portability と containment に優れますが、最も latency-sensitive な
control path の default 実行モデルにしてはいけません。

ルール:

- critical low-jitter route は native core transform または compile-time feature
  を既定にする
- Wasm transform は route ごとの opt-in とし、全 traffic に暗黙適用しない
- host/Wasm 境界では copy を最小化し、安全に可能な範囲で borrowed view や
  shared packet view を優先する
- Wasm 利用 route は throughput だけでなく added latency と jitter の
  benchmark evidence を持つ
- deterministic performance を示せないなら、その機能は hot path に置かず
  operator warning 付きで扱う

## External Plugin Contract

推奨特性:

- explicit protocol version
- capability advertisement
- bounded request / response size
- timeout policy
- disconnect handling
- health telemetry

external plugin は arbitrary shared memory や undocumented host call ではなく、
stable contract 越しに broker と通信すべきです。

## Host Integration Boundary

native host integration が broker semantics を再定義してはいけません。

やってよいこと:

- transport の高速化
- embedding 改善
- host-specific discovery / tooling

やってはいけないこと:

- route semantics の再定義
- broker authorization の bypass
- compatibility fallback の破壊

## Failure Policy

- plugin failure は局所化する
- failure が続けば breaker を開く
- safe mode では plugin を切っても broker core は起動できる
- plugin telemetry は operator view に見えるようにする

## Data Access Policy

plugin には必要なものだけを渡すべきです。

候補 access tier:

- address only
- normalized packet
- normalized packet plus limited metadata
- route-local cache access

global broker internals への unrestricted access は避けるべきです。

## Versioning Policy

すべての plugin contract は最低限次を持つべきです。

- contract version
- minimum broker version
- declared capability
- declared resource limit

## Observability Requirement

broker が見せるべきもの:

- plugin load status
- plugin version
- plugin latency
- plugin timeout count
- plugin error count
- plugin disable event

## 非交渉の不変条件

- optional plugin を全停止しても core routing が有用であること
- plugin failure で packet lineage が壊れないこと
- plugin が raw-packet replay capability を消してはならないこと
- in-process extensibility より safety と containment を優先すること
