# 詳細デリバリープラン

## 目的

この文書は、フェーズ別ロードマップを実行可能な計画へ細かく落とし込むための
ものです。主に次の問いに答えることを目的にします。

- 大きな実装に入る前に何を決めるべきか
- どの順番で組み立てるのがよいか
- 次フェーズへ進むためにどんな証拠が必要か
- 不確実性やコストがどこに集中しているか

## デリバリートラック

実装者が 1 人でも、設計上は 6 本のトラックとして管理するのがよいです。

### Track A: 仕様と互換性

範囲:

- OSC 1.0 parser / encoder の正確性
- legacy tolerance policy
- OSC 1.1 を参考にした optional extension
- address pattern mode selection
- UDP、TCP、SLIP の framing behavior

成果物:

- compatibility matrix
- canonical test vector
- conformance suite
- strict / tolerant / extended mode の定義

### Track B: コアデータプレーン

範囲:

- ingress
- queueing
- routing
- fan-out
- egress isolation
- memory ownership と allocation policy

成果物:

- packet flow design
- backpressure policy
- performance benchmark
- overload behavior policy

### Track C: コントロールプレーンと運用

範囲:

- config model
- dashboard
- metrics
- logs
- capture / replay
- late joiner cache

成果物:

- operator workflow
- observability requirement
- failure recovery behavior

### Track D: アダプターエコシステム

範囲:

- WebSocket
- JSON
- MQTT
- mDNS / DNS-SD
- adapter SDK
- transport metadata

成果物:

- adapter contract
- metadata model
- discovery UX

### Track E: 拡張性と統合

範囲:

- Wasm filter
- schema
- code generation
- external plugin protocol
- UE5 / TouchDesigner integration
- shared memory IPC

成果物:

- plugin model
- schema lifecycle
- IPC ABI
- native integration boundary

### Track F: セキュリティ、同期、配布

範囲:

- zero-trust overlay
- project scoping
- access control
- Ableton Link
- installer
- packaging
- cross-platform service behavior

成果物:

- secure deployment profile
- sync quality model
- release packaging matrix

## 大きな実装前に通す設計ゲート

深い実装に入る前に、以下は明文化しておくべきです。

### Gate 1: 互換性契約

固定すべき内容:

- サポートする OSC 1.0 の基準挙動
- type tag 省略への寛容性
- unknown type tag への方針
- `//` をデフォルト無効にするか
- transport framing support matrix

### Gate 2: 内部イベントモデル

固定すべき内容:

- internal packet representation
- normalized value representation
- ingress packet に付随する metadata
- ownership / borrowing strategy
- broker 内部の timestamp representation

### Gate 3: ルートモデル

固定すべき内容:

- route rule grammar
- v1 の static transform capability
- route ごとの cache policy
- drop / retry / isolate semantics
- route config の validation 方法

### Gate 4: 運用モデル

固定すべき内容:

- 必須 metrics
- structured 化する logs
- replay を dry-run default にするか
- dashboard に出してよい情報
- config reload / rollback の方法

### Gate 5: 拡張境界

固定すべき内容:

- compile-time feature list
- Wasm plugin API の形
- external plugin IPC contract
- v1 における schema の範囲
- native integration の責務境界

## 詳細マイルストーン

## Milestone 00A: 仕様凍結

成果物:

- compatibility matrix
- packet 例と edge case 集
- strict / tolerant / extended mode 定義
- benchmark workload の初期定義

詰めるべき問い:

- parse 可能だが未対応の extension tag は通すか drop するか
- replay や diagnostics のために元の packet bytes を保持するか
- 1.1 の注意点を踏まえ、timetag semantics を内部でどう表現するか

## Milestone 00B: リポジトリとテスト基盤

成果物:

- Rust workspace layout
- test fixture folder
- fuzz target
- benchmark harness
- Windows、macOS、Linux の CI matrix

詰めるべき問い:

- single binary か multiple crate か
- embedded frontend asset strategy
- core crate で許容する dependency の範囲

## Milestone 01A: Fast Path Prototype

成果物:

- UDP ingress
- parser
- route match engine
- UDP egress
- metrics endpoint

成功条件:

- localhost proxy scenario が end-to-end で動く
- packet forwarding behavior が測定できる

## Milestone 01B: 高負荷耐性と分離

成果物:

- 宛先ごとの egress isolation
- queue pressure accounting
- route drop policy
- burst traffic test

成功条件:

- 1 つの悪い consumer が他を止めない

## Milestone 02A: 運用可視化

成果物:

- basic dashboard
- route graph
- throughput / drop view
- structured logs

成功条件:

- operator が Wireshark なしで route 問題を診断できる

## Milestone 02B: 復旧ツール

成果物:

- last-value cache
- late joiner sync
- capture / replay
- ring buffer inspection

成功条件:

- 落ちた node を再同期できる
- packet issue を安全に replay できる

## Milestone 03A: ブラウザとデバイス統合

成果物:

- WebSocket adapter
- JSON message mapping
- adapter SDK draft

成功条件:

- browser-based monitoring / control が raw OSC flow を壊さずに動く

## Milestone 03B: ディスカバリとストリーム系トランスポート

成果物:

- mDNS / DNS-SD support
- metadata publication
- TCP / SLIP framing mode

成功条件:

- discovery が setup を楽にしつつ、manual fallback が常に可能

## Milestone 04A: 実行時拡張

成果物:

- Wasm transform API
- plugin lifecycle
- hot reload

成功条件:

- broker を再ビルドせず user-defined transform を読み込める

## Milestone 04B: スキーマとコード生成

成果物:

- schema draft
- validator
- 実ワークフロー向けの codegen target 1 本

成功条件:

- 少なくとも 1 つの実案件で、schema が runtime 前に integration mistake を
  検出できる

## Milestone 05A: IPC コア

成果物:

- C ABI wrapper
- shared memory proof of concept
- local latency measurement tooling

成功条件:

- UDP と IPC で同じ論理 route graph が動く

## Milestone 05B: ホスト統合

成果物:

- UE5 plugin
- TouchDesigner bridge plan または実装
- 通常 OSC への fallback path

成功条件:

- native integration が measurable benefit を出しつつ必須化しない

## Milestone 06A: セキュリティ拡張

成果物:

- secure route profile
- scoped project ID
- signed または tokenized access path
- abuse protection

成功条件:

- legacy operation を壊さず secure mode を提供できる

## Milestone 06B: 同期と配布

成果物:

- Ableton Link integration
- sync diagnostics
- installer または packaged release
- service-mode validation

成功条件:

- 3 OS で supported build を install / run できる

## 検証シナリオ

以下のようなトラフィックでロードマップを検証すべきです。

### Scenario 1: UE5 ローカルホストプロキシ

- UE5 が camera や gameplay control data を localhost へ送る
- broker が元の宛先と dashboard へ転送する
- 成功指標: burst load 下でも低い追加 jitter と stall のなさ

### Scenario 2: TouchDesigner センサーストーム

- 高頻度 depth / tracking data
- 複数 downstream consumer
- 成功指標: 遅い consumer がバス全体を崩壊させない

### Scenario 3: 展示復旧

- 本番中に 1 node が再起動する
- broker が cache で状態復旧する
- 成功指標: ネットワーク全体を初期化せず operator が復旧できる

### Scenario 4: ブラウザコントロールサーフェス

- browser UI が WebSocket で接続される
- OSC node は plain な互換トラフィックを受け続ける
- 成功指標: web control が base transport reliability を阻害しない

### Scenario 5: 共有ネットワーク安全性

- ノイズの多い共有ネットワーク
- broker が project scope で traffic を選別する
- 成功指標: legacy local use を壊さず無関係 traffic を遮断できる

## ベンチマーク計画

初期 benchmark は、宣伝用の数字である必要はありません。設計判断に使える
数字であることが重要です。

最低限測るもの:

- packets per second の parse throughput
- end-to-end median routing latency
- p95 / p99 jitter
- overload 時の queue growth
- 複数宛先への egress fairness
- diagnostics 有効時の replay overhead

計測モード:

- diagnostics off
- metrics only
- metrics + capture
- cache enabled
- adapter fan-out enabled

## 現在ある計画ノート

次の補助ノートはすでに存在しており、このロードマップと整合している必要が
あります。

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

## 直近で作るべき文書

実装前の計画をさらに進めるなら、次の補助文書群を active planning stack として
扱うべきです。

1. [Route Rule Cookbook And Worked Examples](../../design/ja/route-rule-cookbook-and-worked-examples.md)
2. [Profile-Specific Operator Guides](../../design/ja/profile-specific-operator-guides.md)
3. [Metrics And Telemetry Schema](../../design/ja/metrics-and-telemetry-schema.md)
4. [Benchmark Result Interpretation Guide](../../design/ja/benchmark-result-interpretation-guide.md)
5. [Architecture Decision Record Index](../../design/ja/architecture-decision-record-index.md)
6. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)

## 推奨する v0.1 の範囲

v0.1 に入れるべきもの:

- OSC 1.0 UDP core
- tolerant parser
- route rule
- destination isolation
- metrics
- minimal dashboard または metrics endpoint
- stress test と fuzzing

v0.1 から外すべきもの:

- MQTT
- full discovery automation
- Wasm runtime
- schema / codegen
- native UE5 / TouchDesigner plugin
- zero-trust
- Ableton Link

## なぜこの順番が良いか

最初に守るべき約束を確実に実現できるからです。

- raw OSC compatibility
- 高負荷でも安定した routing
- 障害時の observability

土管が信頼できる状態になってから、その上に何を積んでも強くなります。
