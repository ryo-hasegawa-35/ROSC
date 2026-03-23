# ROSC への貢献ガイド

## 目的

この repository は、高信頼な OSC ルーティング / broker platform のための
docs-first 基盤として作られています。貢献ルールは、
compatibility、design clarity、将来の運用信頼性を守るためにあります。

## 最初に読むもの

大きめの pull request を開く前に、まず次を読んでください。

1. [README.md](./README.md)
2. [README.ja.md](./README.ja.md)
3. [Documentation Index](./docs/README.ja.md)
4. [Design Reading Order](./docs/design/ja/reading-order.md)
5. [Implementation Readiness Checklist](./docs/design/ja/implementation-readiness-checklist.md)
6. [GitHub Foundation And Collaboration Plan](./docs/concepts/ja/github-foundation-and-collaboration-plan.md)

## 貢献フロー

1. 既存 issue から始めるか、実行可能な文脈を持つ新しい issue を作る
2. 関連する design / planning 文書と整合させる
3. `main` から short-lived branch を切る
4. `feature/<topic>`、`docs/<topic>`、`fix/<topic>` のように意図が分かる branch 名を使う
5. issue を前進させる最小の一貫した変更に絞る
6. repository の PR template を使って pull request を作る
7. `main` へ入る前に `@ryo-hasegawa-35` の最終承認を待つ

## Pull Request に求めること

重要な PR は少なくとも次を説明するべきです。

- 何を変えたか
- なぜ必要か
- どの issue を進めるか、または close するか
- どの設計文書に影響するか
- compatibility risk は何か
- 何を evidence として示すか
- rollback / fallback をどう考えるか

## Docs-First Rule

architecture、compatibility、fault handling、recovery、telemetry meaning、
repository-wide policy に触れる変更は、文書を先に更新するか、
少なくとも同じ PR に含めるべきです。

代表例:

- route semantics
- cache / recovery semantics
- plugin trust boundary
- security overlay behavior
- benchmark interpretation
- GitHub governance policy

## Bilingual Documentation Rule

project-level document は英語版と日本語版をそろえます。

少なくとも `docs/concepts/` と `docs/design/` 配下の文書を追加・更新するときは、
英日ペアを維持してください。root の主要文書も、できるだけ英日対応にします。

## Design Governance Rule

normative な design document の意味を変える PR は、次のどちらかであるべきです。

- accepted ADR を参照している
- 新しい proposed ADR を同じ planning window で追加している

入口:

- [Architecture Decision Record Index](./docs/design/ja/architecture-decision-record-index.md)

## Compatibility Rule

既存 OSC との後方互換性はこのプロジェクトの中心的価値です。
performance 目的の変更で、compatibility behavior を静かに弱めてはいけません。

## Safe Change Rule

次のような変更は避けてください。

- experimental acceleration path を mandatory にする
- observability を critical routing liveness に強く結合する
- rollback path を消す
- rehydrate と replay の区別を曖昧にする

## Repository Hygiene

- PR は focused に保つ
- 変更が触る文書を優先して更新する
- task-oriented な PR に unrelated cleanup を混ぜない
- issue と PR の title は具体的で検索しやすくする

## Security Reporting

deployment に実害を出しうる vulnerability は public issue にしないでください。
[SECURITY.ja.md](./SECURITY.ja.md) に従って報告してください。
