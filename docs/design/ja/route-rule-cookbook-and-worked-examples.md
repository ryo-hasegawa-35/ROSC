# Route Rule Cookbook And Worked Examples

## 目的

この文書は、route grammar を実際の設定例へ落とし込むための cookbook です。

grammar 側は「何を表現できなければならないか」を定義します。この文書は、
compatibility、isolation、recovery safety を崩さずに、実運用でどう組むと
よいかを具体例で示します。

## 読み方の注意

- この文書は補助資料であり、normative な grammar の代わりではありません。
- もし example と grammar が食い違う場合は grammar を優先します。
- 例は最短記法よりも分かりやすさを優先します。

関連文書:

- [Route Configuration Grammar](./route-configuration-grammar.md)
- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
- [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)

## Example 1: Transparent Localhost Proxy

使う場面:

- creative tool がすでに valid な OSC 1.0 を送っている
- 最初は localhost 差し込みが最も安全
- まだ schema や transform は不要

この形がよい理由:

- strict compatibility mode が明示される
- queue depth を意図的に小さく保てる
- 最初の導入時に route の意味を追いやすい

```toml
[[routes]]
id = "ue5_local_proxy"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_ue5_local"]
address_patterns = ["/ue5/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 16
coalesce = "none"
priority = 100

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 8
timeout_ms = 5

[routes.cache]
policy = "none"
persist = "ephemeral"

[routes.recovery]
late_joiner = "none"
rehydrate_on_connect = false
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_show_router"
transport = "osc_udp"
```

operator 向けメモ:

- 最初の rollout では transform と cache を入れない方が安全です。
- 後で transform を足すなら、transparent proxy の path を別 route に分けて
  独立に観測できるようにする方がよいです。

## Example 2: Fan-Out With Independent Failure Domains

使う場面:

- 1つの ingress namespace を renderer と dashboard tap の両方へ送りたい
- destination ごとに latency tolerance が違う
- observability が primary renderer path を遅らせてはいけない

設計ルール:

- destination ごとに queue、timeout、breaker policy が違うなら、1 route に
  まとめず別 route に分ける

```toml
[[routes]]
id = "camera_fov_to_renderer"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_local_proxy"]
address_patterns = ["/render/camera/fov"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 8
coalesce = "by_address"
priority = 90

[routes.fault]
drop_policy = "drop_oldest"
breaker_threshold = 10
timeout_ms = 5

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 15000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_renderer_a"
transport = "osc_udp"

[[routes]]
id = "camera_fov_to_dashboard"
enabled = true
mode = "osc1_0_strict"
class = "Telemetry"

[routes.match]
ingress_ids = ["udp_local_proxy"]
address_patterns = ["/render/camera/fov"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 128
coalesce = "by_address"
priority = 20

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 120
burst_limit = 240
breaker_threshold = 40
timeout_ms = 25

[routes.observability]
metrics_level = "minimal"
capture = "off"
emit_correlation_id = true

[[routes.destinations]]
target = "dashboard_tap"
transport = "internal"
```

operator 向けメモ:

- renderer path は低遅延・低 depth を維持します。
- dashboard path は pressure 時に捨てられる前提を明示しています。

## Example 3: Legacy Tolerant Ingress Bridge

使う場面:

- 古い sender が OSC type tag string を省略している
- edge では legacy input を受けたいが、tolerant behavior を全体に広げたくない
- downstream には normalized な routing metadata を渡したい

```toml
[[routes]]
id = "legacy_tracking_bridge"
enabled = true
mode = "osc1_0_legacy_tolerant"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_legacy_tracker"]
address_patterns = ["/tracking/*"]
protocols = ["osc_udp"]

[routes.transform]
rename_address = "/sensors/tracking"

[routes.queue]
max_depth = 256
coalesce = "by_key"
coalesce_key = "source_entity_id"
priority = 60

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 5000
burst_limit = 8000
quarantine_on_malformed = true
timeout_ms = 10

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_td_tracker_bus"
transport = "osc_udp"
```

operator 向けメモ:

- tolerant behavior は edge に寄せて閉じ込めます。
- tolerant ingress を理由に internal strict contract を曖昧にしてはいけません。

## Example 4: Late Joiner State Cache

使う場面:

- consumer を再起動しても scene state をすぐ戻したい
- namespace が trigger ではなく state を表している
- last value で十分に復旧できる

```toml
[[routes]]
id = "scene_state_cache"
enabled = true
mode = "osc1_0_strict"
class = "StatefulControl"

[routes.match]
ingress_ids = ["udp_scene_control"]
address_patterns = ["/scene/state/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 32
coalesce = "by_address"
priority = 85

[routes.cache]
policy = "last_value_per_address"
ttl_ms = 600000
persist = "warm"

[routes.recovery]
late_joiner = "latest"
rehydrate_on_connect = true
rehydrate_on_restart = true
manual_only = false
replay_allowed = false

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_projection_nodes"
transport = "osc_udp"
```

operator 向けメモ:

- trigger 系 namespace は通常、別 route に分けて `manual_only = true`
  か、自動 rehydrate 自体を無効にすべきです。
- cache TTL は便利さではなく operational truth に合わせて決めます。

## Example 5: High-Rate Sensor Stream With Bounded Loss

使う場面:

- 位置や depth のような高レートデータを流す
- bounded loss は許容できる
- critical control traffic は別 route で必ず守りたい

```toml
[[routes]]
id = "depth_stream_fanout"
enabled = true
mode = "osc1_0_strict"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_depth_sensor"]
address_patterns = ["/sensor/depth/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 2048
coalesce = "by_key"
coalesce_key = "sensor_stream_id"
priority = 40
batching = 8

[routes.fault]
drop_policy = "sample"
rate_limit_per_sec = 60000
burst_limit = 90000
breaker_threshold = 100
timeout_ms = 8

[routes.cache]
policy = "journal_window"
ttl_ms = 2500
persist = "ephemeral"

[routes.recovery]
late_joiner = "none"
rehydrate_on_connect = false
replay_allowed = true

[routes.observability]
metrics_level = "minimal"
capture = "off"
emit_correlation_id = false

[[routes.destinations]]
target = "udp_td_depth_consumer"
transport = "osc_udp"

[[routes.destinations]]
target = "ws_depth_monitor"
transport = "ws_json"
encoding = "json"
```

operator 向けメモ:

- この route は perfect retention より continuity を優先しています。
- sender が同じでも、trigger traffic を同じ route に混ぜてはいけません。

## Example 6: Slow Destination Quarantine

使う場面:

- archival や diagnostic 用 consumer が遅くなることがある
- healthy な real-time consumer へ影響させたくない
- breaker / quarantine を operator から見えるようにしたい

```toml
[[routes]]
id = "critical_to_renderer"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_show_control"]
address_patterns = ["/show/critical/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 8
coalesce = "none"
priority = 100

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 6
timeout_ms = 3

[[routes.destinations]]
target = "udp_renderer_primary"
transport = "osc_udp"

[[routes]]
id = "critical_to_archive"
enabled = true
mode = "osc1_0_strict"
class = "ForensicCapture"

[routes.match]
ingress_ids = ["udp_show_control"]
address_patterns = ["/show/critical/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 512
coalesce = "none"
priority = 10

[routes.fault]
drop_policy = "disable_temporarily"
breaker_threshold = 20
timeout_ms = 50

[routes.observability]
metrics_level = "detailed"
capture = "on_breaker"
capture_trigger = "breaker_open"
emit_correlation_id = true

[[routes.destinations]]
target = "file_capture_sink"
transport = "internal"
```

operator 向けメモ:

- archival path を最初から分離しておくと quarantine を信用しやすくなります。
- この route の breaker event は renderer instability を意味してはいけません。

## Example 7: Error-Triggered Forensic Capture

使う場面:

- 常時フル capture は高すぎる
- incident は少ないが深い evidence が必要
- failure 時だけ bounded な capture window がほしい

```toml
[[routes]]
id = "tracking_forensic_window"
enabled = true
mode = "osc1_0_strict"
class = "SensorStream"

[routes.match]
ingress_ids = ["udp_tracking_bus"]
address_patterns = ["/tracking/*"]
protocols = ["osc_udp"]

[routes.queue]
max_depth = 512
coalesce = "by_key"
coalesce_key = "tracking_subject_id"
priority = 50

[routes.fault]
drop_policy = "sample"
breaker_threshold = 50
quarantine_on_malformed = true
timeout_ms = 12

[routes.cache]
policy = "journal_window"
ttl_ms = 10000
persist = "warm"

[routes.recovery]
manual_only = true
replay_allowed = true

[routes.observability]
metrics_level = "detailed"
capture = "on_error"
capture_trigger = "malformed_or_breaker"
log_payload = false
emit_correlation_id = true

[[routes.destinations]]
target = "udp_td_tracking"
transport = "osc_udp"
```

operator 向けメモ:

- 常時 payload log より、bounded capture と強い metadata の組み合わせを優先します。
- sensor traffic の replay は live show へ自動再注入すると危険なので manual に保ちます。

## Example 8: Optional Secure Overlay Boundary

使う場面:

- shared network で source verification が必要
- downstream OSC peer は変えたくない
- authenticated traffic と legacy traffic を明示的に分けたい

```toml
[[routes]]
id = "verified_control_ingress"
enabled = true
mode = "osc1_0_strict"
class = "CriticalControl"

[routes.match]
ingress_ids = ["udp_shared_network_in"]
address_patterns = ["/project_a/control/*"]
security_scopes = ["project_a"]
protocols = ["osc_udp"]

[routes.security]
scope = "project_a"
require_verified_source = true
allowed_identities = ["ops_tablet_a", "show_controller_a"]
allow_legacy_bridge = false

[routes.queue]
max_depth = 16
coalesce = "none"
priority = 95

[routes.fault]
drop_policy = "drop_newest"
breaker_threshold = 8
timeout_ms = 5

[routes.observability]
metrics_level = "standard"
capture = "on_error"
emit_correlation_id = true

[[routes.destinations]]
target = "udp_internal_show_control"
transport = "osc_udp"
```

operator 向けメモ:

- verification は raw OSC payload を書き換えるのではなく broker boundary で行います。
- legacy route を残す場合も、明示的に分けて別 monitoring にすべきです。

## 避けるべき典型的なミス

- ingress が同じだからといって異なる traffic class を 1 route に混ぜる
- fault budget が違う destination を 1 route にまとめる
- hot path 全体で detailed / forensic metrics を常時有効にする
- state cache と replay を同じ safety mechanism とみなす
- secure overlay を全 raw OSC peer に必須化する

## 例全体から見える設計ルール

- latency、durability、trust posture が違うなら route を分ける
- tolerant compatibility mode は edge へ寄せる
- 捨ててもよい observability path は priority を明示的に下げる
- automatic recovery は state に限定し、trigger には慎重にする
- 危険な挙動は必ず metrics と operator tooling に見えるようにする
