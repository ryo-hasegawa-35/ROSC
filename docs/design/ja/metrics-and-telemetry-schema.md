# Metrics And Telemetry Schema

## 目的

この文書は、broker 全体で共通に使う telemetry vocabulary を定義します。

dashboard、log、alert、benchmark、将来の exporter が同じ reality を指すように
するのが目的です。export format は後から変えられても、semantics は安易に
ぶらしてはいけません。

関連文書:

- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Dashboard Information Architecture](./dashboard-information-architecture.md)
- [Benchmark Workload Definition](./benchmark-workload-definition.md)
- [Benchmark Result Interpretation Guide](./benchmark-result-interpretation-guide.md)

## Telemetry 設計原則

- Telemetry は vanity graph のためではなく action のためにある
- Critical control と optional telemetry は同じ failure budget を共有しない
- Cardinality は設計段階から bounded にする
- Raw payload を常時 log しなくても route、destination、broker identity を追えるようにする
- Export adapter が違っても canonical meaning は変えない

## Canonical Telemetry Layer

- counter
- gauge
- histogram
- structured event
- structured log
- long-latency workflow 向けの optional trace

broker では metrics と events を primary とし、trace は additive とします。

## Telemetry Level

`metrics_level` は次の semantic level を持つべきです。

- `off`
  - emergency な process health のみ
- `minimal`
  - route と destination health を低 cardinality で見る
- `standard`
  - default の operational view
- `detailed`
  - 選択した route や incident のための濃い診断
- `forensic`
  - bounded で incident-oriented な深い evidence

ルール:

- production route の基本は `standard`
- `detailed` と `forensic` は opt-in かつ bounded にする
- `forensic` は通常 profile support か explicit incident mode を前提にする

## Naming Convention

Metric 名は次の形を基本とします。

- `rosc_<domain>_<subject>_<measure>`

例:

- `rosc_ingress_packets_total`
- `rosc_route_latency_seconds`
- `rosc_destination_queue_depth`
- `rosc_cache_entries`
- `rosc_security_rejections_total`

ルール:

- counter は `_total` で終える
- gauge は現在値や量が読める形にする
- latency / duration histogram は `_seconds` を使う
- bytes は `_bytes` を使う

## 必須 Dimension

意味がある場所では、canonical schema は次の dimension を持てるようにします。

- `broker_id`
- `profile`
- `compat_mode`
- `traffic_class`
- `transport`
- `ingress_id`
- `route_id`
- `destination_id`
- `adapter_id`
- `plugin_id`
- `reason`
- `scope`

## Cardinality Rule

- `route_id` と `destination_id` は free text ではなく config 上の stable ID を使う
- Raw OSC address を default で metric label にしてはいけない
- Source IP を steady-state metric label にするのは避ける
- Correlation ID は metric label ではなく event / log に置く
- Packet 単位の label は canonical metric では禁止

## Core Metric Family

### Process And Runtime

- `rosc_process_cpu_usage_ratio`
- `rosc_process_memory_bytes`
- `rosc_runtime_task_count`
- `rosc_runtime_fd_count`
- `rosc_runtime_uptime_seconds`

### Ingress

- `rosc_ingress_packets_total`
- `rosc_ingress_bytes_total`
- `rosc_ingress_parse_failures_total`
- `rosc_ingress_rejected_total`
- `rosc_ingress_queue_depth`

### Route

- `rosc_route_matches_total`
- `rosc_route_transform_failures_total`
- `rosc_route_latency_seconds`
- `rosc_route_drops_total`
- `rosc_route_disabled_total`

### Destination / Egress

- `rosc_destination_send_total`
- `rosc_destination_send_failures_total`
- `rosc_destination_queue_depth`
- `rosc_destination_latency_seconds`
- `rosc_destination_breaker_open_total`
- `rosc_destination_quarantine_total`

### Cache And Recovery

- `rosc_cache_entries`
- `rosc_cache_writes_total`
- `rosc_cache_evictions_total`
- `rosc_recovery_rehydrate_total`
- `rosc_recovery_rehydrate_latency_seconds`
- `rosc_recovery_replay_total`

### Security And Identity

- `rosc_security_rejections_total`
- `rosc_security_verified_sources`
- `rosc_security_legacy_bridge_total`
- `rosc_security_scope_mismatch_total`

### Discovery

- `rosc_discovery_services_visible`
- `rosc_discovery_stale_services_total`
- `rosc_discovery_refresh_total`

### Plugin And Adapter

- `rosc_plugin_invocations_total`
- `rosc_plugin_failures_total`
- `rosc_plugin_timeout_total`
- `rosc_adapter_reconnect_total`
- `rosc_adapter_backpressure_total`

### Federation And HA

- `rosc_cluster_replication_lag_seconds`
- `rosc_cluster_failover_events_total`
- `rosc_cluster_peer_disconnect_total`

## Structured Event Schema

High-value event は次の field を持てるようにします。

- `event_id`
- `timestamp`
- `severity`
- `component`
- `event_type`
- `broker_id`
- `compat_mode`
- `traffic_class`
- `ingress_id`
- `route_id`
- `destination_id`
- `reason_code`
- `correlation_id`
- `operator_action_required`

optional field:

- `plugin_id`
- `scope`
- `capture_window_id`
- `config_revision`
- `peer_broker_id`

## Log Schema の期待値

Log は structured で machine-parseable であるべきです。

必須 field:

- `timestamp`
- `level`
- `component`
- `message`
- `broker_id`

relevant なときの推奨 field:

- `route_id`
- `destination_id`
- `reason_code`
- `correlation_id`
- `config_revision`

ルール:

- raw payload logging は default では off
- payload excerpt を許す場合も bounded かつ redaction-aware にする

## Correlation Identifier Rule

- Correlation ID は ingress で生成するか、trusted upstream context から引き継ぐ
- 同じ Correlation ID を event、log、capture metadata、replay record にまたがって使えるようにする
- Raw OSC compatibility のために Correlation ID を必須にしてはいけない

## Alerting Guidance

有効な alert family の例:

- baseline を超える parse failure の継続
- destination breaker の繰り返し open
- critical route での drop count 発生
- 宣言 tolerance を超える replication lag
- stateful route での cache rehydrate failure

Alert rule は profile-aware かつ traffic-class-aware であるべきです。

## Export Model

internal telemetry schema を canonical とし、exporter はそれを次へ写像します。

- Prometheus-style metrics
- OpenTelemetry metrics / logs / traces
- local dashboard stream
- forensic capture metadata

Exporter の違いで canonical field meaning を変えてはいけません。

## Retention Guidance

- minimal operational metric は可能なら通常 restart をまたいで残す
- detailed / forensic data は明示 retention policy で bounded にする
- capture metadata は incident investigation に十分な期間残す

## Non-Negotiable Invariant

- Telemetry は safety behavior を説明するためのもので、競合してはいけない
- Cardinality explosion は design bug とみなす
- optional telemetry が degrade しても critical route health は見え続けるべき
- Canonical name と meaning は benchmark と operator が revision を比較できる程度に安定であるべき
