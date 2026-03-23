# GitHub Foundation And Collaboration Plan

## 目的

この文書は、本格的な実装に入る前に整えておくべき repository と collaboration
の土台を定義します。

このプロジェクトは単なる software ではなく、trustworthy な
cross-platform infrastructure product を目指しています。なので GitHub の
設定も、compatibility、evidence、review clarity、documentation discipline
を最初から補強するものであるべきです。

関連文書:

- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [Effort And Risks](./effort-and-risks.md)
- [Maintainer Approval And Merge Behavior](./maintainer-approval-and-merge-behavior.md)
- [Release Note And Changelog Policy](./release-note-and-changelog-policy.md)
- [Implementation Readiness Checklist](../../design/ja/implementation-readiness-checklist.md)
- [Architecture Decision Record Index](../../design/ja/architecture-decision-record-index.md)

## Collaboration Principle

- risky implementation の前に docs-first
- feature expansion の前に compatibility-first
- performance claim の前に evidence-first
- workflow hardening の前に cross-platform-first
- release confidence の前に rollback-first

## 推奨する Repository Baseline

### Default Branch

- protected default branch は `main` を使う

### Working Branch

- short-lived branch を使う
- 人が作る branch は `feature/<topic>`、`docs/<topic>`、`fix/<topic>` のように、目的がすぐ分かる名前を基本にする
- もし自動化ツールが独自の branch prefix を使う場合は、それは project 全体の命名規則ではなく tool 側の例外として扱う

### Protected Branch Rule

- merge は pull request 経由
- protected branch で force-push 禁止
- merge 前に required status check
- architecture に効く変更には review 必須
- 必要なら merge 時点で最新状態へ追随していることを求める

## Repository Structure の期待値

最低限、次を明確に分離すべきです。

- source code
- docs
- benchmark
- test asset と conformance vector
- example と sample configuration
- GitHub policy file と template
- AI collaboration entry tree

既存の `docs/concepts` / `docs/design` 分離はそのまま維持します。

加えて、異なるツールが同じ前提を読めるように、`.agent/`、`.agents/`、
`.skill/`、`.skills/` の mirror も維持します。

review 補助 automation も最小権限を守り、勝手に merge しないことを前提に
します。

## 先に揃えたい GitHub Baseline File

本格的な coding 前に、少なくとも次を計画します。

- `README.md`
- `LICENSE`
- `CONTRIBUTING.md`
- `CODEOWNERS`
- pull request template
- issue template
- security policy
- changelog または release note policy

これらは architecture doc と整合しているべきで、曖昧に言い換えるだけの
文書にしてはいけません。

## Issue Taxonomy

最低限、issue は次を区別できるようにします。

- design
- implementation
- bug
- compatibility
- performance
- observability
- recovery
- security
- documentation
- research

これにより product exploration と regression work を混同しにくくなります。

## Label Family

有効な label family の例:

- `type:*`
- `area:*`
- `priority:*`
- `status:*`
- `compat:*`
- `profile:*`
- `risk:*`

例:

- `type:design`
- `area:routing-core`
- `compat:legacy-tolerant`
- `profile:secure-installation`
- `risk:operator-visible`

## Milestone Strategy

Milestone は arbitrary な日付ではなく、project phase に対応させるべきです。

- Phase 00 foundation
- Phase 01 core proxy
- Phase 02 observability and recovery
- Phase 03 adapters and discovery
- Phase 04 extensibility and schema
- Phase 05 native integration
- Phase 06 security and sync

これで issue tracking が既存 roadmap とずれにくくなります。

## Pull Request に求めること

重要な PR は次に答えるべきです。

- 何を変えたか
- なぜその変更が必要か
- どの design doc に触れるか
- compatibility risk は何か
- 何の evidence があるか
- rollback または fallback はどうするか

doc-only PR では runtime test の代わりに design consistency が evidence になりえます。

## Review Ownership

少なくとも次の ownership は明示すべきです。

- routing core と compatibility
- recovery と cache semantics
- observability と benchmark claim
- security overlay
- packaging と release process

`CODEOWNERS` はこの architecture boundary を反映すべきです。

## 実装前の Initial CI Plan

code-heavy な CI がまだなくても、repository は少なくとも次を検証すべきです。

- markdown formatting または consistency
- internal documentation link
- local reference への broken path
- 必要なら spelling / terminology check

最初の CI は code quality を装うためではなく、document quality を守るためのものにします。

## Coding 開始後の CI Expansion

後から追加すべき段階:

- formatting / lint check
- unit / integration test
- fuzz または regression entry point
- benchmark / conformance reporting hook
- release artifact verification

## Release And Tagging Policy

- tag は意味のある release / preview state に対応させる
- release note は summary だけでなく evidence へリンクする
- compatibility-sensitive change は明示的に強調する

関連:

- [Release Note と Changelog の方針](./release-note-and-changelog-policy.md)

## Maintainer Approval Behavior

単一 owner の repository では、同じ GitHub account が自分で作った PR に
独立 reviewer として approval を付けることはできません。さらに admin
enforcement が無効なら、review rule があっても GitHub は admin bypass を
提示することがあります。

詳しくは次を参照してください。

- [Maintainer Approval と Merge Behavior](./maintainer-approval-and-merge-behavior.md)

## Security And Disclosure Baseline

public distribution が広がる前に、次を決めるべきです。

- security report の送り先
- acknowledgement までの目安時間
- release と fix の調整方針

secure overlay や network-facing adapter が入るほど重要になります。

## Major Code Work 前の Documentation Gate

次のいずれかなら implementation を止めるべきです。

- major design area に英日両方の doc がない
- architecture-changing decision に ADR trail がない
- GitHub review rule だと risky compatibility change を review なしで通せてしまう

## Immediate Setup Checklist

最初の major implementation sprint の前に準備するもの:

- protected `main`
- branch naming convention
- PR template
- issue template
- label taxonomy
- milestone plan
- CODEOWNERS draft
- docs quality CI

## Non-Negotiable Rule

- Repository process は architecture を補強し、弱めてはいけない
- GitHub automation は compatibility risk と operational risk を隠さず surfaced すべき
- Documentation quality も product foundation の一部である
