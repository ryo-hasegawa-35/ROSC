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
```

Docker 経由で同じ確認を行う場合:

```bash
docker compose run --rm rosc-dev cargo test --workspace
```

開発コンテナ関連:

- [Docker Compose](./compose.yaml)
- [Devcontainer](./.devcontainer/devcontainer.json)

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
