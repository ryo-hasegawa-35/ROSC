# 圧倒的優位性要件

## 目的

実装計画をさらに狭める前に、このシステムが通常の OSC 利用や既存 OSC
ルーターよりも、どこで圧倒的に優れているべきかを定義します。

目標は、生 OSC を壊して別の非互換プロトコルに置き換えることではありません。
生 OSC との互換性を保ったまま、その周囲により強いランタイム、運用レイヤー、
安全性モデルを構築することです。

## 中心仮説

素の OSC は、シンプルで柔軟で広く使われているから強いです。
一方で、以下をほとんど定義しないから弱いです。

- 高負荷時の transport behavior
- congestion と backpressure policy
- fault isolation
- restart 後の recovery
- observability
- security
- schema と validation
- interoperability metadata

圧倒的な後継システムは、OSC の柔軟性を維持しつつ、これらの運用上の穴を
埋める必要があります。

## 何をもって「圧倒的に良い」とするか

このシステムは、少なくとも以下の軸で通常の OSC 運用を上回るべきです。

1. 高負荷時の追加 jitter がはるかに小さい
2. 1 つの遅い consumer や壊れた node に対する fault containment が強い
3. 汚れた共有ネットワークでの安全性が高い
4. restart や部分障害からの復帰がはるかに速い
5. 何が起きたか、なぜ起きたかを把握しやすい
6. UE5 / TouchDesigner / 既存ツール群からの移行が容易
7. コアを壊さずに拡張できる

## Pillar 1: 決定論的な性能

単にベンチマークで速いだけでは足りません。現実のひどいトラフィックでも、
挙動が予測可能であるべきです。

欲しい能力:

- preallocated buffer と bounded memory growth
- route 単位、destination 単位の queue isolation
- lock を最小化した hot path
- 可能な範囲での zero-copy / near-zero-copy packet handling
- bounded-latency な serialization / dispatch
- 偶然の崩壊ではなく、明示的な overload mode
- route 向け QoS class:
  - low-latency control
  - bursty sensor data
  - best-effort telemetry
- route ごとの priority と deadline hint
- 制御系を壊さない範囲で overhead を減らす adaptive batching
- optional な timestamp propagation と jitter measurement
- synthetic happy path ではなく、実ショー系トラフィックに基づく benchmark mode

これが優位性になる理由:

- 1 つの bad sink が全体を汚染しない
- 高頻度 sensor stream が critical control traffic を飢餓させない
- random failure ではなく graceful degradation を選べる

## Pillar 2: 障害の局所化

通常の OSC ネットワークは、うるさく全体崩壊しやすいです。
このシステムは、局所的かつ予測可能に失敗するべきです。

欲しい能力:

- destination ごとの circuit breaker
- noisy / malformed sender を隔離する quarantine mode
- 明示的な drop policy を持つ bounded queue
- route 単位の rate limiting と burst cap
- overload 時の shed-load mode
- malformed traffic を広い graph に拡散させる前に弾く validation layer
- optional plugin / adapter を broker boundary で crash-resistant に隔離
- 危険な extension を無効化して起動する safe mode boot profile

これが優位性になる理由:

- 問題が network-wide meltdown ではなく isolated incident になる
- debug の起点が明確になる

## Pillar 3: 安全性とセキュリティ

生 OSC 互換は維持しつつ、共有ネットワークや常設運用には強い保証が必要です。

欲しい能力:

- project-scoped namespace
- broker による ACL 強制
- secure ingress 向け optional な signed / tokenized envelope
- 安全な compatibility bridge:
  - secure traffic は broker で終端
  - downstream の legacy peer には plain な互換 OSC を流す
- sender identity と provenance tracking
- secure mode 向け anti-spoofing / replay protection
- route permission:
  - read
  - write
  - transform
  - observe
- critical namespace 向け schema-aware validation
- 危険な config change に対する operator approval mode

これが優位性になる理由:

- hostile / chaotic な network 上でも生きられる
- additive な security なので legacy tool を壊さない

## Pillar 4: 高速な復旧と継続性

ここは、通常の OSC ワークフローを大きく超えられる領域です。
多くの OSC システムは restart 後に脆いです。

欲しい能力:

- stateful last-value cache
- namespace 単位の recovery policy
- route config と一部 runtime state を戻す warm restart
- crash-safe config snapshot
- 選択 stream 向け durable event journal
- late-joiner catch-up
- restart した node への one-click rehydrate
- broker 自体の rolling restart support
- optional な active / standby broker pair
- health monitoring と automatic failover hook

これが優位性になる理由:

- 再起動した app が素早く正しい状態へ戻れる
- operator が初期化シーケンスを手で撒き直さなくてよい

## Pillar 5: 可観測性とフォレンジクス

このシステムは、目に見えないネットワーク挙動を見えるものにするべきです。

欲しい能力:

- real-time traffic topology view
- route ごとの throughput、latency、drop metric
- correlation ID と provenance metadata
- time-travel packet buffer
- trigger-based capture:
  - on error
  - on latency spike
  - on drop threshold
- sandbox mode での safe replay
- packet がなぜ match / transform / drop されたかを示す rule debugger
- config change と traffic impact の diff view
- operator action の audit trail

これが優位性になる理由:

- 外部 packet sniffer だけに頼らず製品内から問題を診断できる
- 再現困難なバグを再現可能にできる

## Pillar 6: 互換性と移行容易性

優位性は、導入しやすくなければ意味がありません。

欲しい能力:

- raw OSC 1.0 over UDP を第一級扱い
- type tag を省略した古い sender に対する tolerant parsing
- strict / tolerant / extended の compatibility mode
- transparent localhost proxy mode
- enhanced feature から plain OSC へ簡単に fallback できる
- 以下向けの profile preset:
  - UE5
  - TouchDesigner
  - browser control
  - sensor
  - lighting
- 既存構成向け config import helper
- deeper integration 前に試せる sidecar deployment mode

これが優位性になる理由:

- ユーザーが全システムを書き換えず導入できる
- 深い統合を求める前に broker 側が信頼を獲得できる

## Pillar 7: コアを壊さない拡張性

特殊ケースを全部コアへ入れると、普通のルーターはすぐ崩れます。

欲しい能力:

- lean build 向け Cargo feature preset
- 安全なユーザーロジック用 Wasm transform plugin
- 重量級統合向け external process plugin
- stable adapter SDK
- typed workflow 向け schema / code generation
- 一部 transform module の hot reload
- plugin behavior を試せる simulation mode

これが優位性になる理由:

- custom behavior が増えても broker が不安定なモノリスになりにくい

## Pillar 8: 時間と同期品質

タイミング品質は、OSC の周囲に新しいランタイムを作る最大の理由の 1 つです。

欲しい能力:

- 非現実的な保証を前提にしない慎重な OSC timetag handling
- timestamp provenance:
  - source で生成
  - ingress で観測
  - egress で dispatch
- route ごとの jitter measurement
- clock quality reporting
- optional な PTP / NTP / Ableton Link awareness
- local dispatch 向け deadline-aware scheduling
- transport delay と application delay を区別する diagnostics

これが優位性になる理由:

- あいまいな期待ではなく、計測された timing model をユーザーに渡せる

## Pillar 9: 高可用性と分散運用

この製品がインフラになるなら、永遠に 1 プロセス 1 マシン前提では弱いです。

欲しい能力:

- broker-to-broker federation
- node 間の route replication
- active / standby failover
- local segmentation 向け edge broker
- remote link 向け secure tunnel mode
- site ごとの policy pack

これが優位性になる理由:

- 1 台の workstation から installation network までスケールできる

## Pillar 10: オペレーター体験

本当に強いシステムは、緊張状態でも人が信頼して扱えるものです。

欲しい能力:

- 「今何が起きているか」に答える dashboard
- safe mode startup
- 制御された recovery のための traffic freeze / thaw
- one-click isolate route
- one-click resend cached state
- 明確な warning level:
  - notice
  - degraded
  - danger
- apply 前の config validation
- bad deploy 後の rollback
- UI に埋め込まれた guided recovery playbook

これが優位性になる理由:

- ライブ障害時のパニックを減らせる
- advanced behavior を人間が本当に運用できる形へ落とせる

## 非交渉の原則

どれだけ野心的な機能セットになっても、以下は守るべきです。

- raw OSC 互換を軽く壊さない
- enhanced behavior は additive かつ negotiable にする
- core data plane は周辺エコシステムよりも単純に保つ
- diagnostics が hot path を静かに壊さない
- hidden collapse ではなく explicit degradation を選ぶ

## いま cost を無視して検討する価値が高いもの

現時点では、以下は特に強い差別化要素です。

- route QoS class と deadline-aware routing
- 完全な time-travel capture / replay
- late-joiner rehydration と warm restart
- broker による namespace security
- active / standby broker continuity
- trigger-based incident capture
- schema-driven validation と code generation
- Wasm transform runtime
- route ごとの circuit breaker と quarantine model
- rollback 付き safe live config reload
- clock-quality reporting を含む measured timing diagnostics
- browser-native operations console

## 現在の計画スタック

次の設計ノートはすでに存在しており、今後も整合性を保つ必要があります。

- internal packet / metadata model
- fault model と overload behavior
- recovery model と cache semantics
- compatibility matrix
- route configuration grammar
- benchmark workload definition
- plugin boundary note
- security overlay model
- operator workflow と recovery playbook
- transport and adapter contract
- dashboard information architecture
- native IPC ABI note
- federation / high-availability model
- config validation and migration note
- discovery and service-metadata model
- schema definition format
- C ABI reference header and error-code catalog
- deployment topology and release profile guide
- testing strategy and fuzz corpus plan
- adapter SDK API reference
- conformance vector and interoperability suite guide
- schema evolution and deprecation policy
- dashboard interaction spec and screen inventory
- release checklist and operational runbook

次の補助文書はすでに存在しており、この vision と整合を保つべきです。

1. [Route Rule Cookbook And Worked Examples](../../design/ja/route-rule-cookbook-and-worked-examples.md)
2. [Profile-Specific Operator Guides](../../design/ja/profile-specific-operator-guides.md)
3. [Metrics And Telemetry Schema](../../design/ja/metrics-and-telemetry-schema.md)
4. [Benchmark Result Interpretation Guide](../../design/ja/benchmark-result-interpretation-guide.md)
5. [Architecture Decision Record Index](../../design/ja/architecture-decision-record-index.md)
6. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)

## まとめ

通常の OSC 運用を圧倒するには、単なる速度だけでは足りません。
本当に狙うべきなのは、以下の組み合わせです。

- 決定論的な性能
- 障害の局所化
- 高い observability
- additive な security
- 高速な recovery
- compatibility-first な導入性

この組み合わせがあって初めて、柔軟なメッセージフォーマットを
信頼できるインフラへ変えられます。
