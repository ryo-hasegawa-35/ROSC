# Adapter SDK API Reference

## 目的

この文書は、adapter SDK の意図する API surface を定義します。

まだ実装 package の reference ではなく、将来の SDK 実装が従うべき
architectural reference です。これにより transport や deployment profile が
違っても adapter の振る舞いを予測可能かつ安全に保ちます。

## 設計目標

- adapter SDK を broker internals より小さく保つ
- capability を明示的にする
- broker core の transport neutrality を守る
- adapter failure を routing correctness から隔離する
- SDK usage を可視かつ監査可能にする

## SDK Scope

adapter SDK が助けるべきこと:

- adapter identity と capability の registration
- ingress packet の broker への publish
- broker からの egress work の consume
- health、state、error の report
- discovery result と metadata の expose

adapter SDK が促してはいけないこと:

- route semantics の変更
- broker security policy の bypass
- undocumented global broker state の保持
- internal Rust crate layout への依存

## Conceptual API Surface

SDK が持つべき概念モジュール:

- adapter registration
- ingress publishing
- egress subscription
- health reporting
- discovery reporting
- diagnostics reporting
- lifecycle control

## Registration API

adapter が登録すべきもの:

- `adapter_id`
- `adapter_kind`
- `protocol_family`
- `sdk_contract_version`
- `capabilities`
- `supported_profiles`

registration が明確に失敗すべき条件:

- capability declaration が不正
- contract version が不整合
- required field が欠けている

## Ingress Publishing API

ingress 側 API が扱うべきもの:

- immutable raw payload または canonical message payload
- ingress metadata
- source endpoint reference
- receive timestamp
- 必要に応じた security / discovery metadata

ルール:

- ingress payload は submit 時点で append-only
- adapter は annotate してよいが route meaning を書き換えてはならない
- ingress submit は explicit な backpressure / failure signal を返す

## Egress Consumption API

egress 側 API が受けるべきもの:

- packet reference または payload
- destination reference
- send policy hint
- delivery expectation
- 許可される範囲の correlation metadata

adapter が返すべきもの:

- success
- retryable failure
- terminal failure
- timeout
- backpressure condition

## Health Reporting API

adapter が少なくとも report すべきもの:

- adapter state
- adapter health class
- 必要に応じた session count
- disconnect count
- reconnect count
- local pressure signal

## Discovery Reporting API

discovery-capable adapter が submit できるべきもの:

- discovered service record
- freshness / TTL
- 分かるなら trust classification
- raw observation source

## Diagnostics API

SDK は structured reporting を支えるべきです。

- adapter error
- warning
- transport anomaly
- dropped / rejected traffic count
- degraded state transition

## Capability Declaration

capability は adapter logic に埋もれた ad hoc string ではなく、stable な name
または flag として宣言されるべきです。

例:

- message-oriented
- stream-oriented
- secure-identity
- discovery-capable
- ordered-delivery
- best-effort-delivery
- native-binary-payload

## State Model

SDK は lifecycle state を標準化すべきです。

- `Init`
- `Registering`
- `Ready`
- `Degraded`
- `Disconnected`
- `Recovering`
- `Stopped`

## Error Model

SDK は次を優先すべきです。

- stable error category
- machine-readable error code
- optional structured detail

避けるべきもの:

- public contract としての panic
- undocumented side effect
- semantics が見えない implicit retry

## Threading Expectation

SDK は次を文書化すべきです。

- 何が concurrent call 可能か
- 何が exclusive access を要求するか
- callback が synchronous / asynchronous / event-driven のどれか

## Resource Ownership

SDK は次を明確にすべきです。

- packet buffer の ownership
- session handle の ownership
- metadata snapshot がいつ immutable か
- failure 時にどう cleanup するか

## Versioning Policy

adapter SDK が定義すべきもの:

- SDK contract version
- feature negotiation mechanism
- deprecation window policy

## Reference Adapter Category

SDK が少なくとも支えるべき対象:

- UDP OSC adapter
- TCP / SLIP OSC adapter
- WebSocket / JSON adapter
- MQTT adapter
- shared-memory IPC adapter
- discovery-only adapter

## Observability Requirement

すべての SDK-based adapter が surface できるべきもの:

- version
- state
- health
- throughput
- failure count
- reconnect behavior
- capability declaration

## 非交渉の不変条件

- SDK は architecture が意図した以上の力を expose しない
- adapter author が private broker internals を必要としない
- adapter behavior は inspectable / diagnosable である
- capability と health signal は silence から推測させず explicit に出す
