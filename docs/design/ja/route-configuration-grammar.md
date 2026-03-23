# Route Configuration Grammar

## 目的

この文書は、route configuration の semantic grammar を定義します。

最終的な file format を永久固定することが目的ではありません。どの設定形式でも
表現すべき field、phase、behavior を明確にすることが目的です。

## 設計目標

- human-readable
- hot-reloadable
- validation-friendly
- failure behavior が明示的
- cache / recovery behavior が明示的
- security scope が明示的

## 最初の外部形式として推奨するもの

初期段階では TOML が有力です。Rust の構造体に写しやすく、operator にとっても
比較的読みやすいからです。将来 YAML をサポートしてもよいですが、semantic model
自体は YAML 特有の挙動に依存すべきではありません。

## Route Lifecycle

各 route は次を定義すべきです。

1. どの traffic に match するか
2. その traffic をどう classify するか
3. どの transform を適用するか
4. どこへ送るか
5. pressure 時にどう振る舞うか
6. どの state を cache / recover するか
7. どの security / observability rule を適用するか

## 必須 Route Field

- `id`
- `enabled`
- `mode`
- `match`
- `class`
- `destinations`

### `id`

- stable identifier
- configuration 内で一意
- log、metric、replay、operator action で使う

### `enabled`

- boolean
- disabled route は定義を残したまま live traffic を処理しない

### `mode`

許可値:

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`

### `match`

ingress selection criteria を定義します。

候補 subfield:

- `ingress_ids`
- `source_endpoints`
- `address_patterns`
- `security_scopes`
- `protocols`

### `class`

traffic class:

- `CriticalControl`
- `StatefulControl`
- `SensorStream`
- `Telemetry`
- `ForensicCapture`

### `destinations`

1 つ以上の output definition。

各 destination が持つべきもの:

- `target`
- `transport`
- optional な `encoding`
- optional な `enabled`

## Optional Route Section

- `transform`
- `queue`
- `fault`
- `cache`
- `recovery`
- `security`
- `observability`

## Match Grammar

### Address Matching

`address_patterns` は 1 つ以上の pattern を許可すべきです。

例:

- `"/ue5/camera/*"`
- `"/td/tracking/*"`
- `"//spherical"` は route mode が extended semantics を許す場合のみ

### Source Matching

候補 selector:

- exact endpoint
- ingress binding ID
- authenticated source identity
- project scope

### Protocol Matching

候補 selector:

- `osc_udp`
- `osc_tcp`
- `osc_slip`
- `ws_json`
- `mqtt`
- `ipc`

## Transform Section

目的:

- address rename
- static map
- value scale
- Wasm transform 呼び出し
- external adapter pipeline 呼び出し

候補 field:

- `rename_address`
- `map_arguments`
- `drop_arguments`
- `prepend_arguments`
- `wasm_filter`
- `external_plugin`

ルール:

- transform は safely transformable packet にしか適用しない
- transform failure は route fault policy に従う
- transform は derived packet を作り、元 packet を in-place 変更しない

## Queue Section

候補 field:

- `max_depth`
- `coalesce`
- `coalesce_key`
- `priority`
- `deadline_ms`
- `batching`

`coalesce` 値の例:

- `none`
- `by_address`
- `by_key`

ルール:

- `coalesce = "by_key"` の場合は `coalesce_key` も必須
- `coalesce_key` は arbitrary な raw payload 断片ではなく、stable な
  normalized field か adapter-provided key を参照すべき

## Fault Section

候補 field:

- `drop_policy`
- `rate_limit_per_sec`
- `burst_limit`
- `breaker_threshold`
- `quarantine_on_malformed`
- `timeout_ms`

`drop_policy` 値の例:

- `drop_newest`
- `drop_oldest`
- `sample`
- `disable_temporarily`

## Cache Section

候補 field:

- `policy`
- `key`
- `ttl_ms`
- `persist`

`policy` 値:

- `none`
- `last_value_per_address`
- `last_value_per_key`
- `snapshot_set`
- `journal_window`
- `durable_journal`

`persist` 値:

- `ephemeral`
- `warm`
- `durable`

## Recovery Section

候補 field:

- `late_joiner`
- `rehydrate_on_connect`
- `rehydrate_on_restart`
- `manual_only`
- `replay_allowed`

値の例:

- `late_joiner = "latest"`
- `rehydrate_on_connect = true`
- `replay_allowed = false`

## Security Section

候補 field:

- `scope`
- `require_verified_source`
- `allowed_identities`
- `allow_legacy_bridge`

ルール:

- security config は downstream の legacy peer に対して raw OSC payload の
  変更を要求してはならない
- route authorization は transform と egress の前に行う

## Observability Section

候補 field:

- `metrics_level`
- `capture`
- `capture_trigger`
- `log_payload`
- `emit_correlation_id`

`metrics_level` 値の例:

- `off`
- `minimal`
- `standard`
- `detailed`
- `forensic`

`capture` 値の例:

- `off`
- `on_error`
- `on_breaker`
- `always_bounded`

ルール:

- forensic capture は bounded であるべき
- diagnostics 設定が queue isolation を壊してはならない

## Example TOML Route

```toml
[[routes]]
id = "ue5_camera_fov"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_localhost_in"]
address_patterns = ["/ue5/camera/fov"]
protocols = ["osc_udp"]

[routes.transform]
rename_address = "/render/camera/fov"

[routes.queue]
max_depth = 8
coalesce = "by_address"
priority = 80

[routes.fault]
drop_policy = "drop_oldest"
breaker_threshold = 20
timeout_ms = 10

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 10000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
replay_allowed = false

[routes.security]
scope = "project_a"
require_verified_source = false
allow_legacy_bridge = true

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_renderer"
transport = "osc_udp"

[[routes.destinations]]
target = "dashboard_tap"
transport = "internal"
```

## Validation Rule

- route ID は一意であること
- unsupported な mode / pattern の組み合わせは validation 失敗にする
- cache と recovery policy の組み合わせは同時に検証する
- 危険な default は必要に応じて fail closed にする
- destination reference は解決できること

## Hot Reload Rule

- config update は apply 前に validation する
- apply 失敗で last good config を破壊しない
- route change は diff 可能な operator event として見えるようにする
- 緊急時は delete より disable の方が安全

## 非交渉の不変条件

- live route は compatibility mode を明示的に持つ
- live route は fault policy を明示的に持つ
- cache / recovery policy を traffic から偶然推測してはならない
- security scope は raw OSC payload semantics を変えずに付与できる
- config は apply 前に machine validation される
