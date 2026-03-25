# Benchmark Workload Definition

## 目的

この文書は、performance、predictability、fault behavior を評価するための
workload suite を定義します。

目的は見栄えの良い throughput 数字ではありません。現実の圧力下でも broker が
信頼できるかを測ることです。

## 計測原則

- throughput だけでなく latency と jitter を測る
- steady state だけでなく overload behavior を測る
- diagnostics on / off の両方で測る
- aggregate performance だけでなく route isolation を測る
- disruption 後の recovery time を測る

## コア指標

- packets per second
- median routing latency
- p95 routing latency
- p99 routing latency
- jitter distribution
- queue depth growth
- reason ごとの drop count
- breaker open event
- rehydrate latency
- restart recovery time

## テスト環境

### Local Workstation

用途:

- localhost proxy
- shared memory 比較
- dashboard overhead

### Small Network

用途:

- discovery
- shared network noise
- multiple destination behavior

### Degraded / Synthetic Fault Environment

用途:

- malformed traffic
- stalled consumer
- transform timeout
- adapter disconnect

## Workload Suite

### Workload A: Localhost Control Path

意図:

- low-latency control traffic の baseline

traffic:

- 中程度レートの scalar control message
- 少数の critical destination

計測:

- added latency
- jitter
- metrics と dashboard tap の cost

### Workload B: Sensor Storm

意図:

- bursty な high-rate stream の検証

traffic:

- 大量の sensor-like packet
- 複数宛先 fan-out

計測:

- control-path isolation
- queue growth
- drop policy behavior

### Workload C: Mixed Show Traffic

意図:

- realistic mixed environment

traffic mix:

- camera / control value
- tracking data
- telemetry
- dashboard subscription

計測:

- traffic class 間の fairness
- tail latency
- capture / metrics の影響

### Workload D: Slow Destination

意図:

- per-destination isolation の検証

traffic:

- 意図的に stalled した destination 1 つ
- 健全な destination 複数

計測:

- healthy destination が安定するか
- breaker behavior
- queue containment

### Workload E: Malformed Traffic Flood

意図:

- parser hardening と quarantine の検証

traffic:

- invalid packet
- truncated bundle
- random type tag

計測:

- crash resistance
- quarantine timing
- healthy traffic continuity

### Workload F: Recovery And Rehydrate

意図:

- restart / reconnect 後の continuity 検証

traffic:

- stateful control namespace
- selected late joiner behavior

計測:

- restart recovery time
- rehydrate correctness
- stale cache handling

### Workload G: Wasm Transform Boundary

意図:

- packet transform における host/Wasm 境界コストの定量化

traffic:

- scalar control packet の反復
- Wasm disabled / enabled を切り替えた mixed route class

計測:

- per-packet added latency
- hot path の jitter 増分
- 可能なら copy count または allocation evidence

### Workload H: Schema Validation Depth

意図:

- validation depth と安全性向上の tradeoff を比較する

traffic:

- 同一 namespace を `off`、`shape_only`、`typed`、`strict` で比較

計測:

- throughput cost
- p95 / p99 latency の差分
- bursty sensor traffic への影響

### Workload I: Security Overlay Jitter

意図:

- secure ingress が route の jitter budget に収まるかを検証する

traffic:

- plain mode と secure mode の synchronized control traffic

計測:

- verification による追加コスト
- jitter spread の増加
- secure mode が sync-sensitive workload の tail latency を悪化させるか

## Feature Toggle Matrix

各 workload は少なくとも次の mode で実施すべきです。

- core only
- metrics enabled
- metrics plus dashboard
- capture enabled
- cache enabled
- transform enabled
- relevant な場合は security overlay enabled

## Benchmark Reporting Format

各 run で最低限残すべきもの:

- git revision または document revision
- operating system
- CPU class
- active feature toggle
- workload definition version
- route count
- destination count

## Success Interpretation

次の状態なら改善しているとみなせます。

- p95 / p99 が realistic pressure 下でも bounded
- sensor flood 中も critical traffic が安定
- degraded mode が chaotic ではなく explicit
- recovery が速く正しい
- diagnostics cost が測定可能で受容範囲

## 非交渉ルール

- benchmark traffic class は実際の製品用途を反映すること
- すべての benchmark suite に少なくとも 1 つの fault / overload case を含める
- latency 報告では、可能な限り ingress-to-egress と external network を分離する
- benchmark result は revision 比較ができる程度に再現可能であること
