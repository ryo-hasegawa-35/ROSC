# Fault Model と Overload Behavior

## 目的

この文書は、malformed traffic、traffic flood、slow consumer、broken
adapter、plugin crash、memory pressure、operator mistake など、環境が
不健全になったときに broker がどう振る舞うべきかを定義します。

中心原則はシンプルです。

- 局所的に失敗する
- 明示的に劣化する
- まず critical control path を守る

## Failure Domain

失敗は、可能な限り小さい境界に閉じ込めるべきです。

### Sender Domain

例:

- packet flood
- malformed payload
- spoofed traffic
- invalid namespace usage

期待する containment:

- sender 単位の quarantine
- sender 単位の rate limiting
- sender 単位の drop counter

### Route Domain

例:

- 重い transform
- bad route rule
- pathological fan-out
- route 固有の flood

期待する containment:

- route 単位の queue budget
- route 単位の circuit breaker
- route 単位の disable / isolate action

### Destination Domain

例:

- 遅い UDP sink
- 切断された TCP client
- 詰まった WebSocket peer
- failing MQTT broker

期待する containment:

- destination ごとの egress queue
- destination ごとの circuit breaker
- destination 固有の shed behavior

### Adapter / Plugin Domain

例:

- Wasm timeout
- adapter panic
- IPC plugin disconnect
- untrusted transform が CPU を食いすぎる

期待する containment:

- process または runtime boundary
- timeout と kill policy
- broker core からの adapter isolation

### Broker Domain

例:

- memory pressure
- allocator churn
- excessive diagnostics overhead
- configuration thrash

期待する containment:

- global degraded mode
- capture の削減
- nonessential feature の shedding
- safe mode restart path

## Fault Taxonomy

broker は operator が理解できる分類で fault を表すべきです。

想定カテゴリ:

- `MalformedInput`
- `LegacyOpaqueInput`
- `RateExceeded`
- `QueueOverflow`
- `DestinationStalled`
- `TransformFailed`
- `TransformTimedOut`
- `AdapterUnavailable`
- `SecurityDenied`
- `ConfigInvalid`
- `ClockUncertain`
- `ResourcePressure`

## Overload State

broker は静かに劣化するのではなく、明示的な運転状態を持つべきです。

### `Healthy`

- すべての queue が目標範囲内
- active shedding なし
- open な circuit breaker なし

### `Pressured`

- 一部 queue が成長中
- best-effort route では一時 drop が始まりうる
- operator warning を見えるようにする

### `Degraded`

- noncritical route が shed-load 中
- 1 つ以上の destination または transform が isolated
- core control path は保護されている

### `Emergency`

- broker が自己崩壊を防いでいる状態
- diagnostics、capture、optional transform を削ることがある
- 最優先 class だけを強く保護する

### `SafeMode`

- broker が risky extension を無効にして起動 / 再起動する状態
- まず最小の compatible routing core を守る

## Queueing 原則

- すべての queue は bounded にする
- queue の ownership は明示する
- 1 destination が無関係 destination を止めてはならない
- control traffic と telemetry traffic は同じ failure budget を共有しない

推奨 queue ownership:

- socket または ingress worker ごとの ingress queue
- route または route group ごとの route queue
- destination ごとの egress queue
- plugin boundary ごとの plugin inbox

## Traffic Class

overload 時にすべての packet を同じ扱いにするべきではありません。

想定 traffic class:

- `CriticalControl`
- `StatefulControl`
- `SensorStream`
- `Telemetry`
- `ForensicCapture`

意図:

- `CriticalControl`
  - latency を守る
  - freshest packet を優先
  - queue は極小
- `StatefulControl`
  - 最新の正しい state を守る
  - safe なら cache-aware coalescing を許可
- `SensorStream`
  - sampling や drop-old を許容
  - control route を飢餓させない
- `Telemetry`
  - best effort
  - pressure 時に最初に shed
- `ForensicCapture`
  - primary routing を絶対に block しない
  - bounded な side path のみ

## Drop / Shed Policy

drop behavior は意図的かつ可視であるべきです。

対応したい policy:

- `DropNewest`
- `DropOldest`
- `Sample`
- `CoalesceByAddress`
- `CoalesceByKey`
- `DisableRouteTemporarily`

推奨 default:

- `CriticalControl`
  - 極小 queue
  - `DropOldest`
- `StatefulControl`
  - safe な範囲で最新 state へ coalesce
- `SensorStream`
  - sample または `DropOldest`
- `Telemetry`
  - `DropNewest` または sustained overload 時に route disable

## Circuit Breaker

circuit breaker は少なくとも route と destination の境界に必要です。

想定状態:

- `Closed`
- `Open`
- `HalfOpen`

Open 条件の例:

- 送信失敗の連続
- transform failure の連続
- sustained timeout
- threshold 超えの queue overflow

Half-open 回復:

- 制限付き traffic で probe
- 一定の成功 window を満たしてから close

## Quarantine

quarantine は通常の shedding より強い対応です。

想定 trigger:

- malformed packet storm
- security violation の繰り返し
- namespace violation の繰り返し
- hard cap を超える sender flood

quarantine action:

- 一定 cooling-off interval の間、その source の traffic を全部 drop
- diagnostics に source を明示
- 必要なら repeat offender に operator acknowledgement を要求

## Transform / Plugin Failure Policy

transform や plugin は data plane を不安定化させてはなりません。

ルール:

- すべての transform に CPU / wall-time budget を持たせる
- transform failure はその route / plugin に局所化
- failure が続けば breaker を開く
- degraded mode では optional transform path を bypass できる
- broker core は無関係 traffic を流し続ける

## Pressure 時の Diagnostics

observability は重要ですが、hot path を壊しては意味がありません。

方針:

- metrics は最後まで残す
- capture は routing を犠牲にする前に削る
- replay は critical live traffic と競合させない
- dashboard refresh は sample または degrade できるようにする

## Pressure 時の Security

secure route と compatibility route は同じ壊れ方をすべきではありません。

推奨方針:

- verification に失敗した secure ingress は fail-closed
- legacy 互換 raw OSC route は route drop policy に従って扱う
- security check が無関係な nonsecure route を block しない

## Recovery との関係

fault handling と recovery は噛み合っている必要があります。

例:

- breaker が open しても cached good state は消さない
- replay traffic は marker を持ち、quarantine を誤発火させにくくする
- warm restart で breaker state を戻すかは明示設定にする

## Operator Control

operator が明快に介入できるべきです。

有用な control:

- isolate route
- drain route
- freeze replay
- disable plugin
- resend cached state
- enter safe mode
- acknowledge recurring fault

## Metrics と Alert

最低限見えるべきもの:

- domain ごとの queue depth
- reason ごとの drop 数
- quarantine event
- breaker open 回数
- transform timeout 回数
- adapter disconnect 回数
- degraded mode transition
- emergency mode transition

## 非交渉の不変条件

- 1 つの遅い consumer が無関係 consumer を止めてはならない
- malformed / hostile traffic で broker が crash してはならない
- optional feature は core data plane より先に shed される
- すべての overload action は測定可能であるべき
- 隠れた queue growth は許されない

## 後続文書

この model は次の文書と直接整合している必要があります。

- internal packet / metadata model
- recovery model と cache semantics
- route configuration grammar
- security overlay model
