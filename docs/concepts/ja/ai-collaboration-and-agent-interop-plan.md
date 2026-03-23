# AI 協業と Agent Interop 方針

## 目的

この文書は、ROSC が将来的に複数の AI システムによって開発されることを前提
に、repository 側でどのような準備をしておくべきかを定義します。

狙いは単に assistant を「反応させる」ことではなく、複数の AI が同じ
architecture、compatibility promise、review discipline、handoff quality
に収束できるようにすることです。

## なぜ必要か

AI ツールごとに、repository から文脈を読む入り口が違います。

たとえば次のような場所を探すものがあります。

- `.agent/`
- `.agents/`
- `.skill/`
- `.skills/`
- `AGENT.md`
- `AGENTS.md`
- `SKILL.md`
- `SKILLS.md`

ひとつの慣習しか用意しないと、後から参加する AI が重要な制約を読み落とし、
互いに矛盾する作業をしやすくなります。そこで ROSC では、agent discovery の
互換レイヤーを用意しつつ、短い AI 向けファイルが正式な設計体系と分離して
暴走しないようにします。

## 正本の考え方

プロジェクトの正式な意味は、引き続き `docs/` と root の repository policy
file に置きます。

AI 用の入口ディレクトリは、次の目的で使います。

- 新しい agent の立ち上がり時間を短くする
- まず読むべき制約を 1 か所にまとめる
- handoff と協業ルールを明示する
- 再利用できる local skill を案内する

つまり、正式文書の代替ではなく、正式文書へ正しく到達するための短い入口です。

## 必要な AI 入口ツリー

ROSC では、次の互換ディレクトリ群を維持します。

- `.agent/`
- `.agents/`
- `.skill/`
- `.skills/`

### Agent 系ディレクトリ

agent 系は、project 全体の前提と安全な作業ルールを伝えるために使います。

最低限、次に答えられる必要があります。

- この project は何か
- repository は今どの段階か
- 最初に何を読むべきか
- 何を壊してはいけないか
- どう handoff するべきか

### Skill 系ディレクトリ

skill 系は、役割別の具体的な作業手順を与えるために使います。

最低限、次に答えられる必要があります。

- docs 作業ではどの skill を使うか
- 設計変更ではどの skill を使うか
- issue 整理ではどの skill を使うか
- 実装前 planning ではどの skill を使うか
- compatibility review ではどの skill を使うか

## Mirror 方針

- `.agent/` と `.agents/` は意味がずれないように保つ
- `.skill/` と `.skills/` は意味がずれないように保つ
- `AGENT.md` / `AGENTS.md`、`SKILL.md` / `SKILLS.md` も意味を分岐させない
- もしこれらの短い AI 向けファイルの意味を変えるなら、必要に応じて正式文書
  も同時に更新する

## 推奨する内容構成

短い入口ファイルと補助ファイルの両方を持つのが望ましいです。

### 短い入口ファイル

最初の接触で必要なのは次です。

- mission
- project status
- must-read list
- non-negotiable rule
- handoff expectation

### 補助ファイル

補助として次を持たせます。

- repository map
- workflow と safety rule
- role-based skill guide
- handoff template

## Governance の期待値

どの AI が来ても、同じ governance に誘導されるべきです。

- architecture や semantics は docs-first
- プロジェクト文書は英日で用意する
- protected `main` には pull request review 経由で入れる
- 最終承認は repository owner が持つ
- acceleration claim よりも compatibility と rollback discipline を優先する

## Handoff Contract

実質的な AI 作業は、最低限次を含む handoff を残すべきです。

- 何を変えたか
- どの文書や issue を根拠にしたか
- 何を検証したか
- 何が未解決か
- 次に何をするのが最も良いか

ROSC は単発のやりとりではなく、複数 contributor / 複数 agent が継続的に
引き継げることを重視しているため、これは重要です。

## 既存文書との関係

この方針は、次の文書を補完するものであって置き換えるものではありません。

- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [GitHub Backlog Map](./github-backlog-map.md)
- [Implementation Readiness Checklist](../../design/ja/implementation-readiness-checklist.md)

## 最終的に守りたいこと

この repository では、異なる AI が来ても整合的な作業をしやすくしつつ、設計
規律を回避しやすくしてはいけません。
