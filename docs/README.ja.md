# ROSC ドキュメント

このディレクトリは、次の 2 系統に分けて整理しています。

- [構想 / 計画 (English)](./concepts/en/README.md)
- [構想 / 計画 (Japanese)](./concepts/ja/README.md)
- [設計仕様 (English)](./design/en/README.md)
- [設計仕様 (Japanese)](./design/ja/README.md)

参考資料:

- [OSC 1.0 Specification PDF](./references/osc-1.0-specification.pdf)
- [OSC 1.1 NIME 2009 PDF](./references/osc-1.1-nime-2009.pdf)
- [OpenSoundControl.org](https://opensoundcontrol.stanford.edu/)
- [OSC 1.0 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_0.html)
- [OSC 1.1 Spec Page](https://opensoundcontrol.stanford.edu/spec-1_1.html)

## フォルダ方針

- `docs/concepts/`
  - ビジョン、ロードマップ、計画、優先順位、フェーズ整理
- `docs/design/`
  - 技術仕様、挙動モデル、設定文法、運用設計
- `docs/references/`
  - 一次資料のローカルコピー

## 作業ルール

今後追加するプロジェクト文書はすべて:

- 英語版を作る
- 日本語版を作る
- `concepts` か `design` のどちらかに明確に置く

## 最初に見るとよい入口

- ビジョンやフェーズ計画から入る:
  - [Concepts / Planning (Japanese)](./concepts/ja/README.md)
- GitHub や repository 準備から入る:
  - [GitHub Foundation And Collaboration Plan (English)](./concepts/en/github-foundation-and-collaboration-plan.md)
  - [GitHub Foundation And Collaboration Plan (Japanese)](./concepts/ja/github-foundation-and-collaboration-plan.md)
  - [AI Collaboration And Agent Interop Plan (English)](./concepts/en/ai-collaboration-and-agent-interop-plan.md)
  - [AI Collaboration And Agent Interop Plan (Japanese)](./concepts/ja/ai-collaboration-and-agent-interop-plan.md)
  - [Gemini PR Review Assistant (English)](./concepts/en/gemini-pr-review-assistant.md)
  - [Gemini PR Review Assistant (Japanese)](./concepts/ja/gemini-pr-review-assistant.md)
  - [Maintainer Approval And Merge Behavior (English)](./concepts/en/maintainer-approval-and-merge-behavior.md)
  - [Maintainer Approval And Merge Behavior (Japanese)](./concepts/ja/maintainer-approval-and-merge-behavior.md)
  - [License And Contributor Policy (English)](./concepts/en/license-and-contributor-policy.md)
  - [License And Contributor Policy (Japanese)](./concepts/ja/license-and-contributor-policy.md)
  - [CI Expansion And Required-Check Roadmap (English)](./concepts/en/ci-expansion-and-required-check-roadmap.md)
  - [CI Expansion And Required-Check Roadmap (Japanese)](./concepts/ja/ci-expansion-and-required-check-roadmap.md)
  - [Release Note And Changelog Policy (English)](./concepts/en/release-note-and-changelog-policy.md)
  - [Release Note And Changelog Policy (Japanese)](./concepts/ja/release-note-and-changelog-policy.md)
- 複数 AI の入口を確認する:
  - [Agent Brief](../.agent/AGENT.md)
  - [Skill Catalog](../.skill/SKILL.md)
- 技術設計から入る:
  - [Design Reading Order (English)](./design/en/reading-order.md)
  - [Design Reading Order (Japanese)](./design/ja/reading-order.md)
- 用語の確認:
  - [Glossary (English)](./design/en/glossary.md)
  - [Glossary (Japanese)](./design/ja/glossary.md)
- 実装前 fixture の確認:
  - [Conformance Fixtures](../fixtures/conformance/README.md)
  - [Benchmark Fixtures](../fixtures/benchmarks/README.md)
