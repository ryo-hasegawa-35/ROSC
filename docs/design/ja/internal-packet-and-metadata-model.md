# 内部パケット / メタデータモデル

## 目的

この文書は、生 OSC 互換を維持しつつ、routing、observability、recovery、
security などの上位機能を実現するために、broker が packet を内部でどう表現
するべきかを定義します。

必要なのは、次の 2 つを両立することです。

- 元の packet を忠実に保持すること
- 安全な処理のための正規化された内部 view を持つこと

## 設計目標

- raw OSC bytes を保持し、pass-through を可能にする
- hot path で不要な copy を避ける
- routing metadata は OSC payload の外側に置く
- strict / tolerant / extended の compatibility mode を支える
- 「inspect 可能な packet」と「opaque だが forward 可能な packet」を区別する
- transform や replay の lineage を明示する

## コア表現レイヤー

broker は、1 つの immutable な packet record に対する複数の view として
packet を扱うべきです。

### Layer 1: Raw Packet Record

これが ingress 時点の正本です。

想定フィールド:

- `packet_id`
- `raw_bytes`
- `transport`
- `source_endpoint`
- `received_at`
- `compatibility_mode`
- `raw_size`

ルール:

- `raw_bytes` は ingress 後に不変
- 以降の表現はすべてこの record を参照する
- retention が有効な限り、replay や forensic で元の bytes を復元できる

### Layer 2: Parse Result

broker が packet をどこまで理解できたかを表します。

想定状態:

- `WellFormedMessage`
- `WellFormedBundle`
- `LegacyUntypedMessage`
- `MalformedPacket`
- `WellFormedButOpaque`

解釈:

- `LegacyUntypedMessage` は、type tag string がないが tolerant mode では
  受理される packet
- `WellFormedButOpaque` は、forward には十分でも payload を安全に深く
  inspect できない packet

### Layer 3: Normalized View

routing や transform が安全に使うための typed な内部 view です。

想定構造:

- `MessageView`
- `BundleView`
- `OpaqueView`

`MessageView` が持つべきもの:

- address
- type tag の出所
- argument list
- argument span または decoded value
- 必要に応じて元 bytes への参照

`BundleView` が持つべきもの:

- timetag
- 順序付き element list
- nested された message / bundle view

`OpaqueView` が持つべきもの:

- forward や log に必要な最小情報
- deep inspect や transform が unsafe であることを示す capability flag

## Compatibility Mode

この名称は他文書でも統一して使います。

### `osc1_0_strict`

- valid な OSC 1.0 message / bundle を要求
- 明示的な type tag string を要求
- pattern syntax は 1.0 のみ
- 未対応 type tag は transform 対象にしない

### `osc1_0_legacy_tolerant`

- type tag string を省略した古い message を受理
- legacy packet として保持
- address ベースの routing は許可
- route が custom decoder を明示しない限り argument-aware transform は禁止

### `osc1_1_extended`

- 1.0 baseline を土台にする
- `//` path traversal wildcard を有効化できる
- extended type support や metadata-aware transport を扱える
- それでも raw packet bytes と additive compatibility は維持する

## Packet Capability Flag

各 parsed packet は、安全でない処理を避けるための capability flag を持つべき
です。

想定 flag:

- `forwardable`
- `inspectable_address`
- `inspectable_arguments`
- `transformable`
- `cacheable_candidate`
- `replayable`
- `security_checked`

例:

- type tag のない legacy message は `forwardable` と
  `inspectable_address` は true にできても、
  `inspectable_arguments` は false であるべき

## Argument Model

内部の argument model は、既知の decoded value と opaque payload span を
分けるべきです。

想定カテゴリ:

- 必須 OSC 1.0 value kind
- optional / extended OSC value kind
- legacy untyped byte region
- unknown tagged argument

推奨 decoded value set:

- `Int32`
- `Float32`
- `String`
- `Blob`
- `Int64`
- `Timetag`
- `Double64`
- `Symbol`
- `Char`
- `Rgba`
- `Midi4`
- `True`
- `False`
- `Nil`
- `Impulse`
- `Array`
- `UnknownTagged`

ポリシー:

- unknown tagged argument を既知型へ黙って coercion しない
- raw packet が保たれていれば unknown tagged argument でも forward は可能
- non-transformable argument を含む packet に transform は適用しない

## Legacy Untyped Message

古い OSC sender は type tag string を省略することがあります。ここは防御的に
扱うべきです。

推奨ポリシー:

- raw bytes を保持
- 安全な範囲で address を parse
- argument type が分かるふりをしない
- argument payload は opaque として扱う
- address ベース routing と byte-exact forwarding は許可
- custom decoder を定義しない限り value-aware transform は禁止

危険な heuristic に頼らず tolerant mode を正直に保つためです。

## Metadata Model

metadata は additive かつ OSC payload 外部に置くべきです。

### Ingress Metadata

- `source_endpoint`
- `transport`
- `received_at`
- `interface_id`
- security mode 時の `source_identity`
- `compatibility_mode`
- `parse_status`

### Routing Metadata

- `route_matches`
- `qos_class`
- `priority`
- `drop_preference`
- `cache_policy`
- `security_scope`

### Lineage Metadata

- `parent_packet_id`
- `derived_from_transform`
- `replay_session_id`
- `capture_session_id`

### Timing Metadata

- 存在する場合の `source_timetag`
- `ingress_observed_at`
- `routed_at`
- `egress_enqueued_at`
- `egress_sent_at`

### Diagnostics Metadata

- `correlation_id`
- `warning_flags`
- `drop_reason`
- `quarantine_reason`

## Ownership と Memory Policy

packet bytes は immutable shared ownership、decode view は borrowed 参照を
優先すべきです。

推奨方針:

- ingress で 1 つの immutable packet buffer を確保
- parse view は可能な限りその buffer を borrow
- decoded value は必要時にだけ materialize
- transform は lineage metadata 付きの新しい packet record を作る
- diagnostics や replay は packet record を参照し、場当たり copy に頼らない

## Transform Model

transform は元 packet record を in-place で書き換えてはなりません。

ルール:

- input packet は immutable
- transform は derived packet を出力
- derived packet は `parent_packet_id` を持つ
- original raw bytes は forensic のため残す
- transform failure は parent packet を壊さない

## Security Model との関係

security 情報は、legacy traffic の raw OSC payload ではなく metadata と
broker policy に属します。

そのため内部 model は少なくとも以下を支えるべきです。

- authenticated source identity
- verified / unverified ingress state
- project scope
- route authorization decision

これにより security を additive に保てます。

## Recovery Model との関係

内部 packet model は、hot path を壊さずに recovery 機能を支えるべきです。

重要な接点:

- cache entry は safely decoded できた normalized value だけを参照する
- replay は byte-exact resend のため raw bytes を使える
- rehydrate は selected route の normalized state snapshot を使える
- packet lineage で live traffic と replay traffic を区別する

## 非交渉の不変条件

- retention がある限り raw ingress bytes を復元できる
- parser が持たない確実性を normalized view が捏造しない
- metadata が payload semantics を黙って変えない
- security と diagnostics は additive に保つ
- transform は derived packet に対して行い in-place mutation しない

## 後続文書

この model は次の文書と直接整合している必要があります。

- compatibility matrix
- route configuration grammar
- fault model と overload behavior
- recovery model と cache semantics
