# ROSC 構想と計画

このフォルダには、Rust 製の次世代 OSC ルーティングバスに向けた
ビジョン、段階的ロードマップ、計画メモ、優先順位付け、そして GitHub /
collaboration の準備方針をまとめています。

## 参照した一次資料

このロードマップは、以下の一次資料を基準に整理しています。

- `../../references/osc-1.0-specification.pdf`
- `../../references/osc-1.1-nime-2009.pdf`
- https://opensoundcontrol.stanford.edu/spec-1_0.html
- https://opensoundcontrol.stanford.edu/spec-1_1.html
- https://opensoundcontrol.stanford.edu/files/2009-NIME-OSC-1.1.pdf
- https://opensoundcontrol.stanford.edu/

重要な解釈:

- クラシックな形で正式にまとまっているのは OSC 1.0 仕様です。
- OpenSoundControl のサイト上でも、OSC 1.1 は 1.0 と同じ形式の独立した
  仕様ページというより、2009 年の NIME 論文が 1.1 の方向性を示す主要資料
  と読めます。
- 正しく構成された OSC 1.0 メッセージは、1.1 指向の実装でも継続して扱える
  ことを前提にします。
- OSC は「完全なアプリ間プロトコル」ではなく、まずエンコーディング /
  コンテンツフォーマットとして扱うべきです。発見、認証、スキーマ、サービス
  振る舞いは上位レイヤーとして積み上げます。

## プロジェクト原則

- 既存の OSC 1.0 トラフィックとの後方互換性を最優先で守る
- 上位機能はすべて追加的なものとして設計し、生 OSC に強制しない
- ルーティングコアは小さく、決定的で、独立にテスト可能に保つ
- モノリス化を避け、必要な機能だけを組み合わせられるモジュラー構成にする
- Windows、macOS、Linux を初期段階からビルドとテストの対象にする

## フェーズ一覧

- [Phase 00](./phase-00-foundation.md): 仕様基準、ベンチマーク、テスト基盤、
  リポジトリ土台
- [Phase 01](./phase-01-core-proxy.md): ローカル OSC プロキシ、ルーティング
  エンジン、トランスポートコア
- [Phase 02](./phase-02-observability-recovery.md): ダッシュボード、キャッシュ、
  キャプチャ、リプレイ、運用機能
- [Phase 03](./phase-03-adapters-discovery.md): WebSocket、MQTT、ディスカバリ、
  プロトコルメタデータ
- [Phase 04](./phase-04-extensibility-schema.md): プラグインモデル、Wasm
  フィルター、スキーマ、コード生成
- [Phase 05](./phase-05-native-integration.md): 共有メモリ IPC と UE5 /
  TouchDesigner ネイティブ統合
- [Phase 06](./phase-06-security-sync-release.md): ゼロトラスト拡張、同期、
  ハードニング、クロスプラットフォーム配布

## 構想と計画の補助文書

- [圧倒的優位性要件](./advantage-requirements.md)
- [工数とリスク](./effort-and-risks.md)
- [詳細デリバリープラン](./detailed-delivery-plan.md)
- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [GitHub Backlog Map](./github-backlog-map.md)

## 関連する設計仕様

- [Design Reading Order](../../design/ja/reading-order.md)
- [Glossary](../../design/ja/glossary.md)
- [Implementation Readiness Checklist](../../design/ja/implementation-readiness-checklist.md)
- [アーキテクチャ原則](../../design/ja/architecture-principles.md)
- [内部パケット / メタデータモデル](../../design/ja/internal-packet-and-metadata-model.md)
- [Fault Model と Overload Behavior](../../design/ja/fault-model-and-overload-behavior.md)
- [Recovery Model と Cache Semantics](../../design/ja/recovery-model-and-cache-semantics.md)
- [Compatibility Matrix](../../design/ja/compatibility-matrix.md)
- [Route Configuration Grammar](../../design/ja/route-configuration-grammar.md)
- [Benchmark Workload Definition](../../design/ja/benchmark-workload-definition.md)
- [Route Rule Cookbook And Worked Examples](../../design/ja/route-rule-cookbook-and-worked-examples.md)
- [Metrics And Telemetry Schema](../../design/ja/metrics-and-telemetry-schema.md)
- [Benchmark Result Interpretation Guide](../../design/ja/benchmark-result-interpretation-guide.md)
- [Plugin Boundary Note](../../design/ja/plugin-boundary-note.md)
- [Security Overlay Model](../../design/ja/security-overlay-model.md)
- [Operator Workflow And Recovery Playbook](../../design/ja/operator-workflow-and-recovery-playbook.md)
- [Profile-Specific Operator Guides](../../design/ja/profile-specific-operator-guides.md)
- [Transport And Adapter Contract](../../design/ja/transport-and-adapter-contract.md)
- [Dashboard Information Architecture](../../design/ja/dashboard-information-architecture.md)
- [Native IPC ABI Note](../../design/ja/native-ipc-abi-note.md)
- [Federation And High-Availability Model](../../design/ja/federation-and-high-availability-model.md)
- [Config Validation And Migration Note](../../design/ja/config-validation-and-migration-note.md)
- [Discovery And Service Metadata Model](../../design/ja/discovery-and-service-metadata-model.md)
- [Schema Definition Format](../../design/ja/schema-definition-format.md)
- [C ABI Reference Header And Error-Code Catalog](../../design/ja/c-abi-reference-header-and-error-code-catalog.md)
- [Deployment Topology And Release Profile Guide](../../design/ja/deployment-topology-and-release-profile-guide.md)
- [Testing Strategy And Fuzz Corpus Plan](../../design/ja/testing-strategy-and-fuzz-corpus-plan.md)
- [Adapter SDK API Reference](../../design/ja/adapter-sdk-api-reference.md)
- [Conformance Vector And Interoperability Suite Guide](../../design/ja/conformance-vector-and-interoperability-suite-guide.md)
- [Schema Evolution And Deprecation Policy](../../design/ja/schema-evolution-and-deprecation-policy.md)
- [Dashboard Interaction Spec And Screen Inventory](../../design/ja/dashboard-interaction-spec-and-screen-inventory.md)
- [Release Checklist And Operational Runbook](../../design/ja/release-checklist-and-operational-runbook.md)
- [Architecture Decision Record Index](../../design/ja/architecture-decision-record-index.md)

## 推奨する実装順

このプロジェクトは次の順で進めるのが最も安全です。

1. まず生 OSC のルーティングを正確かつ高速にする
2. 次に高負荷時でも観測・復旧できるようにする
3. その後で既存 OSC を壊さずにプロトコルの幅を広げる
4. 高度機能はプラグインや拡張機構として追加する
5. ネイティブ統合はブローカーコアが安定してから入れる
6. セキュリティや高度同期は最終的に追加オプションとして載せる

## 守るべき互換性の基準

実装では常に以下を守ります。

- OSC パケットは 4 バイト境界を維持する
- 数値フィールドは OSC 定義どおりビッグエンディアンを維持する
- UDP データグラムを第一級の経路として扱う
- 既存の 1.0 アドレスパターンの挙動を保つ
- 古い送信側が type tag string を省略していても頑健に受信する
- 未知の拡張型や未対応型が来ても、ストリームを壊さず防御的に扱う
