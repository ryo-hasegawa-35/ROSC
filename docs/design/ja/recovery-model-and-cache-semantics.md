# Recovery Model と Cache Semantics

## 目的

この文書は、restart、disconnect、partial failure、operator intervention の
後に、broker がどう continuity を回復するかを定義します。

目標は単に「packet を replay すること」ではありません。正しい状態を安全かつ
素早く戻すことです。

## Recovery 原則

- recovery は magical にせず explicit にする
- recovery policy は route または namespace 単位の opt-in にする
- aggressiveness より correctness を優先する
- replay と rehydrate は別の道具として分ける
- trigger 的 traffic を事故で auto-resend しない

## Recovery Layer

### Layer 1: Config Recovery

戻すもの:

- route graph
- transport binding
- adapter setting
- security scope
- cache policy

これは dynamic runtime rehydrate より先に行うべきです。

### Layer 2: Transport Recovery

戻すもの:

- listening socket
- adapter connection
- broker identity と service metadata

これで broker は receive / send 可能な状態へ戻ります。

### Layer 3: State Recovery

戻すもの:

- selected cached value
- selected journal
- 明示的に対応した route-local working state

### Layer 4: Incident Recovery

支えるもの:

- capture からの replay
- cached state の resend
- route 単位の recovery
- minimal mode への safe fallback

## Cacheable State Class

すべての packet stream を同じ方法で cache してはいけません。

### `NoCache`

用途:

- impulse
- one-shot trigger
- unsafe または曖昧な traffic

### `LastValuePerAddress`

用途:

- continuous control value
- scalar state
- 最新値だけで current state が定まる parameter

### `LastValuePerKey`

用途:

- logical object key が address または argument に埋め込まれている message
- tracker ごと、light ごと、device ごとの state

### `SnapshotSet`

用途:

- scene や state snapshot を構成する bounded な address 集合

### `JournalWindow`

用途:

- 直近の短い履歴
- debugging
- carefully controlled な catch-up

### `DurableJournal`

用途:

- broker restart をまたぐ persistence が必要な selected critical stream

## Cacheability の Safe Default

デフォルトで route を cacheable とみなしてよいのは、次を満たす場合です。

- action ではなく state を表している
- 最新値の replay が idempotent、またはそれに近い
- internal model で safely inspectable である
- namespace owner が `NoCache` を指定していない

デフォルトで auto-replay してはいけない route:

- trigger 的なもの
- security-sensitive なもの
- safe semantics を持たない opaque legacy payload
- destructive または irreversible な action を制御するもの

## Rehydrate と Replay

これは product language でも実装でも明確に分けるべきです。

### Rehydrate

- current state を戻す
- 通常は cache または snapshot に基づく
- 基本的に latest known value のみを送る
- restart recovery や late joiner 向け

### Replay

- historical traffic を再送する
- timing relationship を保持することもある
- debugging、testing、controlled reconstruction 向け
- デフォルトは sandbox mode にする

## Cache Key

cache key は明示的かつ設定可能であるべきです。

候補要素:

- namespace
- address
- source identity
- route identifier
- extracted logical key

例:

- `/ue5/camera/fov` の last value
- performer ID ごとの last value
- lighting fixture ID ごとの last value

## Freshness と Expiry

cache には freshness rule が必要です。

推奨 control:

- max age
- namespace TTL
- explicit invalidation message または control action
- restart persistence policy

ルール:

- expiry した cache entry を automatic rehydrate に使わない
- stale だが retained な entry は manual review 用に残してよい

## Persistence Level

state によって persistence depth を分けるべきです。

### Ephemeral

- memory only
- broker restart で失われる

### Warm

- local snapshot から broker restart 後に戻る
- selected last-value cache や config 向け

### Durable

- 意図的に永続化する
- selected journal や critical state snapshot 向け

## Recovery Trigger

recovery は次を契機に起こりえます。

- downstream node reconnect
- adapter reconnect
- broker restart
- operator action
- standby broker への failover

重要なのは、ad hoc な resend flood ではなく policy に紐づくことです。

## Recovery Order

推奨順序:

1. minimal broker core を起動
2. config と route graph を復元
3. transport と adapter を復元
4. security と policy state を復元
5. selected warm cache を復元
6. route ごとの rehydrate を許可
7. optional plugin と advanced behavior を有効化

この順序なら、未準備の graph に state を流し込む危険を減らせます。

## Late Joiner Semantics

late joiner recovery は namespace または route 単位で explicit にするべきです。

有用な policy:

- automatic sync なし
- subscribe / connect 時に latest value を送る
- bounded snapshot set を送る
- operator-triggered rehydrate を待つ

## Warm Restart

warm restart は continuity を保つことを目指すべきですが、
「何も起きなかったふり」をしてはいけません。

戻してよいもの:

- config
- route graph
- selected cache
- practical な範囲の selected adapter session

blind restore してはいけないもの:

- 設定がない transient breaker state
- replay session
- unsafe trigger stream

## Active / Standby Continuity

standby broker がある場合、live traffic から勝手に state を推測すべきでは
ありません。

推奨戦略:

- config change を replicate
- selected cache state を replicate
- selected journal を replicate
- standby 起点の traffic と分かる lineage を残す

## Security と Recovery

recovery は security scope を尊重する必要があります。

ルール:

- cache entry は security scope を保持する
- secure namespace を insecure rehydrate path へ漏らさない
- scope 越えの operator-triggered resend は explicit authorization を要する

## Operator Experience

有用な recovery operation:

- route の最新 cached state を resend
- namespace の snapshot set を resend
- captured window を sandbox で replay
- current live state と cached state を比較
- stale cache を invalidate
- route を no-rehydrate mode に強制

## Metrics と Audit

見えるべきもの:

- cache entry count
- cache hit count
- rehydrate event
- replay event
- stale entry count
- invalidation event
- durable journal usage
- restart 後の restore time

## 非交渉の不変条件

- automatic recovery が危険な trigger traffic を黙って resend してはならない
- replay と rehydrate は分離する
- stale state は stale と見えるようにする
- recovery policy は偶然 global ではなく route-aware にする
- durable recovery は opt-in のまま保つ

## 後続文書

この model は次の文書と直接整合している必要があります。

- internal packet / metadata model
- fault model と overload behavior
- security overlay model
- route configuration grammar
