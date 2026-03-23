# Transport And Adapter Contract

## 目的

この文書は、broker core とすべての ingress / egress adapter の間の contract
を定義します。

この contract は、core を transport-neutral に保ちつつ、performance、
safety、recovery に関わる transport 差分を明示的に扱えるようにする必要が
あります。

## 設計目標

- すべての transport をまたぐ 1 つの core routing model
- 明示的な transport capability
- bounded な failure domain
- additive な security と metadata
- transport 固有挙動が route semantics に漏れないこと

## Adapter Role

各 adapter は次のいずれか、または両方の役割を持ちます。

- ingress adapter
- egress adapter

両方を持ってもよいですが、core は別契約として扱うべきです。

## Core Adapter Contract

各 adapter が宣言すべきもの:

- `adapter_id`
- `adapter_kind`
- `protocol_family`
- `direction`
- `capabilities`
- `state`
- `health`
- `version`

## Ingress Contract

ingress adapter が届けるべきもの:

- immutable な raw packet bytes、または同等の canonical payload
- ingress metadata
- source endpoint identity
- transport identity
- receive timestamp

追加で届けてもよいもの:

- authenticated source identity
- discovery metadata
- transport session metadata

ingress adapter がしてはならないこと:

- broker route configuration の変更
- 宣言されていない packet transform の暗黙適用
- broker security policy の bypass

## Egress Contract

egress adapter が受け取るべきもの:

- derived または pass-through の packet record
- egress metadata
- destination reference
- send policy hint

egress adapter が報告すべきもの:

- send success
- send failure reason
- timeout
- disconnect
- backpressure state

## Transport Capability Model

すべての adapter は capability を明示的に広告すべきです。

想定 capability flag:

- message-oriented
- stream-oriented
- preserves packet boundaries
- supports secure identity
- supports discovery metadata
- supports bidirectional session state
- supports ordered delivery
- supports best-effort delivery
- supports native binary payloads

## Endpoint Model

broker は endpoint を transport-neutral な reference に正規化すべきです。

想定 endpoint field:

- `endpoint_id`
- `adapter_id`
- `protocol`
- `address`
- `port`
- `path_or_topic`
- `scope`
- `identity_requirement`

例:

- UDP host / port
- WebSocket session ID
- MQTT topic
- IPC channel name

## Session Model

transport には stateless なものと sessionful なものがあります。

stateless の例:

- UDP datagram

sessionful の例:

- TCP
- WebSocket
- MQTT
- IPC link

core は session state を hidden な route dependency にせず、metadata と
health signal としてのみ扱うべきです。

## Connection State

sessionful adapter では明示的な state を使います。

- `Init`
- `Connecting`
- `Ready`
- `Degraded`
- `Disconnected`
- `Recovering`

## Framing Policy

transport framing は adapter boundary で明示されるべきです。

対応 framing family:

- raw UDP packet
- size-prefixed stream packet
- SLIP-framed stream packet
- non-OSC protocol 向け adapter-defined envelope

ingress 後に route や packet model が framing を推測するのは避けるべきです。

## Metadata Contract

adapter は metadata を付与してよいですが、分類を明確にする必要があります。

想定 metadata group:

- ingress transport metadata
- discovery metadata
- security metadata
- session metadata
- adapter diagnostics

metadata は additive であり、raw OSC payload semantics を黙って変えては
なりません。

## Backpressure Contract

adapter は fault model と噛み合う必要があります。

ingress adapter が報告すべきもの:

- receive pressure
- rate limit application
- malformed input count

egress adapter が報告すべきもの:

- queue pressure
- send timeout
- destination stall
- retry exhaustion

broker は telemetry がないことから backpressure を推測してはなりません。

## Retry And Delivery Semantics

adapter contract は、その transport が保証できないことを約束してはなりません。

例:

- UDP egress は best effort
- TCP / WebSocket は ordered delivery を保てても session level で失敗しうる
- MQTT の delivery semantics は設定や broker behavior に依存する

delivery policy は adapter code の hidden property ではなく、明示的な route
または destination configuration であるべきです。

## Security Interaction

security は次のいずれかで提供されえます。

- transport context
- outer secure envelope
- broker policy

adapter が verified identity を surface してもよいですが、authorization は
broker の責務です。

## Discovery Interaction

discovery-capable adapter が出すべきもの:

- discovered endpoint identity
- freshness / TTL
- capability advertisement
- 可能なら human-readable label

discovery result は operator UX を助けるべきであって、explicit route policy を
bypass してはなりません。

## Versioning

adapter contract は core crate layout とは独立に versioning されるべきです。

各 adapter が宣言すべきもの:

- contract version
- adapter version
- supported capability set
- supported framing set

## Observability Requirement

core が確認できるべきもの:

- adapter state
- adapter health
- session count
- ingress rate
- egress success / failure count
- disconnect count
- reconnect count

## 非交渉の不変条件

- undocumented adapter behavior に route semantics を依存させない
- adapter failure は局所化する
- transport framing は明示的であること
- security identity は adapter が提供しても authorization は broker に残す
- broker boundary で raw OSC compatibility を保つ
