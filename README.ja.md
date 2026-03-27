# ROSC

ROSC は、次世代 OSC ルーティングバス / message broker を目指す
docs-first な Rust プロジェクトです。重いリアルタイム環境でも、
高速・予測可能・観測可能・復旧しやすく、既存 OSC ワークフローとの
後方互換性を保てる基盤を作ることを目的にしています。

## 現在の状態

repository は現在、最初の実装フェーズに入りました。

いま存在するもの:

- 英日でそろえた構想 / 計画文書
- 英日でそろえた設計仕様群
- repository governance と delivery planning 文書
- 次の実装フェーズに向けた GitHub workflow と review ルール
- 初期 ADR、conformance corpus 計画、benchmark fixture 計画
- Phase 01 向け Rust workspace bootstrap
- conformance fixture に接続した初期 OSC parser / encoder core
- route / config / bounded queue の最小実装

まだ存在しないもの:

- production-ready な runtime 挙動
- protocol adapter 実装
- benchmark harness の実装
- native integration の実装

## 開発の始め方

ローカルで workspace を確認する場合:

```bash
cargo test --workspace
cargo run -p rosc-broker -- check-config examples/phase-01-basic.toml
cargo run -p rosc-broker -- proxy-status examples/phase-01-basic.toml
cargo run -p rosc-broker -- proxy-status examples/phase-01-basic.toml --safe-mode
cargo run -p rosc-broker -- proxy-overview examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-readiness examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-diagnostics examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- proxy-attention examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready
cargo run -p rosc-broker -- proxy-incidents examples/phase-01-basic.toml --fail-on-warnings --require-fallback-ready --history-limit 10
cargo run -p rosc-broker -- watch-config examples/phase-01-basic.toml --poll-ms 1000 --fail-on-warnings
cargo run -p rosc-broker -- watch-udp-proxy examples/phase-01-basic.toml --poll-ms 1000 --ingress-queue-depth 1024 --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
cargo run -p rosc-broker -- diff-config examples/phase-01-basic.toml examples/phase-01-basic-changed.toml
cargo run -p rosc-broker -- serve-health 127.0.0.1:19191 --config examples/phase-01-basic.toml
cargo run -p rosc-broker -- run-udp-proxy examples/phase-01-basic.toml --health-listen 127.0.0.1:19191 --control-listen 127.0.0.1:19292 --fail-on-warnings --require-fallback-ready --safe-mode
curl -X POST http://127.0.0.1:19292/freeze
curl -X POST http://127.0.0.1:19292/routes/camera/isolate
curl -X POST http://127.0.0.1:19292/routes/restore-all
curl -X POST http://127.0.0.1:19292/destinations/udp_renderer/rehydrate
curl -X POST "http://127.0.0.1:19292/routes/camera/replay/sandbox_tap?limit=1"
curl http://127.0.0.1:19292/status
curl http://127.0.0.1:19292/report
curl http://127.0.0.1:19292/overview
curl http://127.0.0.1:19292/readiness
curl http://127.0.0.1:19292/diagnostics?limit=10
curl http://127.0.0.1:19292/attention
curl http://127.0.0.1:19292/incidents?limit=10
curl http://127.0.0.1:19292/overrides
curl http://127.0.0.1:19292/signals
curl http://127.0.0.1:19292/signals?scope=problematic
curl http://127.0.0.1:19292/blockers
curl http://127.0.0.1:19292/history/operator-actions
curl http://127.0.0.1:19292/history/config-events
```

`--control-listen` は意図的に loopback 専用です。`127.0.0.1`、`::1`、`localhost` のような
ローカル専用アドレスだけを使い、wildcard や外部から到達できる bind は拒否されます。

`proxy-status`、`proxy-overview`、`proxy-readiness`、`proxy-diagnostics`、`proxy-attention`、`proxy-incidents`
は、`jq` などへそのまま流せるように stdout へ JSON だけを出す契約にそろえています。

Docker 経由で同じ確認を行う場合:

```bash
docker compose run --rm rosc-dev cargo test --workspace
```

開発コンテナ関連:

- [Docker Compose](./compose.yaml)
- [Devcontainer](./.devcontainer/devcontainer.json)

現在の Phase 01 実装範囲:

- strict / legacy-tolerant / extended を扱う OSC parser / encoder
- static address rename を含む route matching
- bounded ingress queue と UDP ingress binding
- breaker 付き per-destination egress worker と isolation
- Prometheus text に落とせる in-memory health / metrics export
- 初期ローカルトラブルシュート向けの HTTP `/healthz` と `/metrics` endpoint
- config diff と last-known-good を持つ safe apply 基盤
- top-level UDP ingress / destination config と end-to-end localhost proxy relay
- route ごとの cache policy と bounded rehydrate を使う最初の late-joiner recovery
- live routing と分離した bounded capture / sandbox replay / recovery audit の最小土台
- TOML から調整できる per-destination queue / drop / breaker policy
- last-known-good を維持する polling-based safe config watch flow
- bound 済み ingress へ戻る UDP destination を startup 時点で弾く proxy loop prevention
- ingress / destination / route と direct UDP fallback hint、runtime queue health を JSON で確認できる `proxy-status`
- operator warning や fallback 不足があると起動 / reload を止められる safety gate
- optional な capture / replay / restart rehydrate を落として core UDP routing を維持する最小 safe-mode launch profile
- controlled restart と将来の hot reload に向けて ingress port をきれいに返す clean shutdown
- 新しい runtime が立ち上がらなかった場合に直前の live config へ戻せる managed proxy reload supervision
- live UDP proxy と同時に health / metrics endpoint を公開できる optional な co-hosted health service
- freeze/thaw、route isolation、live status 取得を外から行える optional な control endpoint
- control endpoint から destination rehydrate と sandbox replay も叩けるようになり、live operator workflow を外から再現可能
- control endpoint は bulk route restore と percent-decoded resource id にも対応し、operator recovery flow を扱いやすくした
- runtime status と control history endpoint から bounded な recent operator action / config transition を追えるようになり、インシデント後の追跡がしやすくなった
- CLI の report 表示と control-plane の `/report` `/blockers` が同じ safety evaluation を共有するようになり、運用判断のズレを減らした
- control-plane の `/report` から structured な override / runtime signal / route signal / destination signal を読めるようにし、`/overrides` `/signals` から個別取得もできるようにした
- `proxy-overview` と control-plane の `/overview` で、report・current status・problematic signal view をまとめた operator 向け 1-shot snapshot を取得できるようにした
- `proxy-readiness` と control-plane の `/readiness` で、`ready / degraded / blocked` と理由、route/destination 集計をまとめた機械可読な readiness contract を取得できるようにした
- `proxy-diagnostics` と control-plane の `/diagnostics` で、その 1-shot snapshot に bounded な recent operator/config history も束ねて返せるようにし、インシデント一次切り分けをやりやすくした
- `proxy-attention` と control-plane の `/attention` で、active override・最新の incident highlight・いま本当に注意すべき route/destination id だけを返す compact な一次判断ビューも追加した
- `proxy-incidents` と control-plane の `/incidents` で、open blocker/warning、filtered な recent issue history、復旧に必要な problematic route/destination の詳細をまとめて返す incident-focused view も追加した
- `/signals?scope=problematic` で、operator が今見るべき route / destination signal だけに payload を絞れるようにした
- config の reject / block / reload failure も counters だけでなく reason 付きの recent history として残るようになった

## ドキュメント入口

- [Documentation Index](./docs/README.md)
- [Documentation Index (Japanese)](./docs/README.ja.md)
- [Concepts / Planning (English)](./docs/concepts/en/README.md)
- [Concepts / Planning (Japanese)](./docs/concepts/ja/README.md)
- [Design Specs (English)](./docs/design/en/README.md)
- [Design Specs (Japanese)](./docs/design/ja/README.md)
- [Changelog](./CHANGELOG.md)
- [Changelog (Japanese)](./CHANGELOG.ja.md)

## AI 協業の入口

- [Agent Brief](./.agent/AGENT.md)
- [Agent Brief (Plural Alias)](./.agents/AGENTS.md)
- [Skill Catalog](./.skill/SKILL.md)
- [Skill Catalog (Plural Alias)](./.skills/SKILLS.md)
- [AI Collaboration And Agent Interop Plan](./docs/concepts/en/ai-collaboration-and-agent-interop-plan.md)
- [AI Collaboration And Agent Interop Plan (Japanese)](./docs/concepts/ja/ai-collaboration-and-agent-interop-plan.md)
- [Gemini PR Review Assistant](./docs/concepts/ja/gemini-pr-review-assistant.md)
- [Maintainer Approval And Merge Behavior](./docs/concepts/ja/maintainer-approval-and-merge-behavior.md)
- [Release Note And Changelog Policy](./docs/concepts/ja/release-note-and-changelog-policy.md)

おすすめの読み順:

1. [Design Reading Order](./docs/design/ja/reading-order.md)
2. [Glossary](./docs/design/ja/glossary.md)
3. [Implementation Readiness Checklist](./docs/design/ja/implementation-readiness-checklist.md)
4. [GitHub Foundation And Collaboration Plan](./docs/concepts/ja/github-foundation-and-collaboration-plan.md)
5. [License And Contributor Policy](./docs/concepts/ja/license-and-contributor-policy.md)

## プロジェクト原則

- 既存 OSC 1.0 トラフィックとの後方互換性を守る
- 高度機能は additive overlay として設計し、生 OSC に強制しない
- ルーティングコアは決定的で、独立にテスト可能に保つ
- observability と recovery を後付けではなく最初から重視する
- プロジェクト文書は英語版と日本語版をそろえる
- AI の入口ディレクトリはミラーさせ、どのツールでも同じ前提を読むようにする

## Collaboration Rule

- `main` には review と approval を通った変更だけを入れる
- 作業は可能な限り Issue 起点で始める
- 大きな変更は、関連する設計文書を先に更新する
- pull request では影響する issue と設計文書を明記する
- `main` に入る変更の最終承認は `@ryo-hasegawa-35` が持つ

関連文書:

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [CONTRIBUTING.ja.md](./CONTRIBUTING.ja.md)
- [SECURITY.md](./SECURITY.md)
- [SECURITY.ja.md](./SECURITY.ja.md)
- [GitHub Foundation And Collaboration Plan](./docs/concepts/ja/github-foundation-and-collaboration-plan.md)

## 参照資料

主要な OSC 参照資料はローカルに保存しています。

- [OSC 1.0 Specification PDF](./docs/references/osc-1.0-specification.pdf)
- [OSC 1.1 NIME 2009 PDF](./docs/references/osc-1.1-nime-2009.pdf)

オンライン参照:

- [OpenSoundControl.org](https://opensoundcontrol.stanford.edu/)
- [OSC 1.0 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_0.html)
- [OSC 1.1 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_1.html)

## ライセンス

この repository は [MIT License](./LICENSE) で公開します。
理由と contributor policy は
[License And Contributor Policy](./docs/concepts/ja/license-and-contributor-policy.md)
にまとめています。
