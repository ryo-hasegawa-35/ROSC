# C ABI Reference Header And Error-Code Catalog

## 目的

この文書は、host-native integration と external tooling が使う public C ABI
surface の想定形を定義します。

これは design reference であり、実装 header ではありません。

## 設計目標

- stable な ABI boundary
- minimal surface area
- opaque ownership
- 明確な error reporting
- explicit な version negotiation

## Naming Convention

推奨 symbol prefix:

- public C ABI function は `rosc_`
- constant と error code は `ROSC_`

推奨 handle 名:

- `rosc_broker_t`
- `rosc_route_t`
- `rosc_endpoint_t`
- `rosc_channel_t`

## 基本 ABI Rule

- stable layout を持つ public struct には explicit size または version を含める
- exposed struct より opaque handle を優先する
- string は explicit pointer-plus-length、または ownership rule を定義する
- caller / callee ownership はすべての buffer で明記する

## ABI Version Negotiation

ABI surface が expose すべきもの:

- library ABI version
- minimum compatible caller version
- feature bitset

version mismatch は explicit error を返すべきで、undefined behavior にしては
なりません。

## Function Family

reference surface が持つべき family:

- version / feature query
- broker lifecycle
- configuration load / validate / apply
- endpoint / route inspection
- channel / IPC lifecycle
- diagnostics / health query
- last-error retrieval

## Result Model

compact で explicit な result model を推奨します。

- success code
- stable error code
- optional な detail retrieval function

避けるべきもの:

- exception
- thread rule のない hidden global mutable state
- undocumented error side channel

## Error Code Catalog

想定 stable code:

| Code | Meaning |
| --- | --- |
| `ROSC_OK` | success |
| `ROSC_ERR_UNKNOWN` | unspecified failure |
| `ROSC_ERR_INVALID_ARGUMENT` | caller が invalid argument を渡した |
| `ROSC_ERR_UNSUPPORTED_VERSION` | ABI または feature version mismatch |
| `ROSC_ERR_NOT_INITIALIZED` | init 前に operation した |
| `ROSC_ERR_ALREADY_INITIALIZED` | duplicate init または start |
| `ROSC_ERR_INVALID_HANDLE` | stale または invalid opaque handle |
| `ROSC_ERR_BUFFER_TOO_SMALL` | caller buffer が不足 |
| `ROSC_ERR_CONFIG_INVALID` | config validation 失敗 |
| `ROSC_ERR_CONFIG_APPLY_FAILED` | config apply 失敗 |
| `ROSC_ERR_IO_FAILURE` | I/O または OS-level failure |
| `ROSC_ERR_TIMEOUT` | operation timeout |
| `ROSC_ERR_BACKPRESSURE` | downstream または channel pressure で block |
| `ROSC_ERR_CHANNEL_UNAVAILABLE` | IPC channel unavailable |
| `ROSC_ERR_SECURITY_DENIED` | security verification または authorization failed |
| `ROSC_ERR_UNSUPPORTED_FEATURE` | compile / enable されていない feature |
| `ROSC_ERR_STATE_CONFLICT` | current state と operation が不整合 |

## Error Detail Model

より豊かな detail が必要なら expose してよいもの:

- last error code
- last error message
- optional structured diagnostic snapshot

ただし stable contract はまず error code に依存すべきです。

## Threading Expectation

ABI は少なくとも次を文書化すべきです。

- broker handle が thread-safe か
- multiple reader を許すか
- configuration apply に exclusive access が必要か
- callback registration を許すならその rule

## Buffer Convention

推奨 pattern:

- caller が buffer と size を渡す
- 不足時は callee が required size を返す
- ownership は explicit に保つ

## State Machine Expectation

ABI は stable lifecycle state を定義すべきです。

- created
- configured
- running
- degraded
- stopped
- destroyed

間違った state で呼ばれた operation は明確に失敗するべきです。

## 非交渉の不変条件

- public ABI は internal Rust API より小さく単純であること
- すべての ownership rule は明示的であること
- error handling は machine-readable であること
- version mismatch が undefined behavior にならないこと
- surface convenience より fallback-friendly design を優先すること
