# Gemini PR Review Assistant

## 目的

この文書は、maintainer が「困惑」シグナルを出したときに、pull request へ
日本語の review 補助コメントを投稿する repository bot の動作を説明します。

## この bot がすること

maintainer が pull request 本文に `confused` リアクションを付けると、bot は
次を行います。

1. リアクションを検知する
2. 受け付けたことを示すため `+1` リアクションを付ける
3. pull request の metadata と changed file を集める
4. Gemini に日本語の review 補助要約を依頼する
5. PR comment として次を投稿する
   - ざっくり概要
   - 変更内容
   - 良い点
   - 弱い点 / 気になる点
   - リスク
   - 提案

## この bot がしないこと

この bot は次をしません。

- pull request の approve
- formal な GitHub review state としての request changes
- pull request の merge
- code の変更

あくまで comment assistant です。

## Trigger の仕組み

理想的には maintainer の `confused` リアクション自体を trigger にしたいです。

ただし GitHub Actions では、現時点で workflow trigger として
`issue_comment`、`pull_request`、`pull_request_review`、`schedule` などは
ありますが、reaction を直接 trigger にする workflow event はありません。
そのため ROSC では polling 方式を使います。

- scheduled workflow が 5 分ごとに open pull request を確認する
- PR 本文に対する trigger user の `confused` reaction を探す
- 見つかったら、その reaction を 1 回だけ処理する

手動実行用に `workflow_dispatch` も用意します。

## Secret と設定

必須:

- repository secret `GEMINI_API_KEY`

任意の repository variable:

- `REVIEW_TRIGGER_USER`
  - 未設定時は repository owner を使う
- `REVIEW_MODEL`
  - 未設定時は `gemini-2.5-flash`
- `REVIEW_MAX_FILES`
  - 未設定時は `25`
- `REVIEW_MAX_PATCH_CHARS`
  - 未設定時は `60000`

## Anti-Spam Rule

同じ reaction を schedule のたびに繰り返し処理しないように、bot は自分の
comment 内に hidden marker を書き込みます。

新しく review を出してほしい場合は、maintainer が `confused` reaction を
一度外して付け直すか、`workflow_dispatch` と `force` を使います。

## 権限の安全性

workflow permission は意図的に最小限にしています。

- `contents: read`
- `pull-requests: read`
- `issues: write`

つまり、PR を読んで comment / reaction を付けることはできますが、
merge 権限は持ちません。

## 運用上の注意

この bot は human review を置き換えるものではなく、補助するためのものです。
最終判断は maintainer が pull request 本体と bot の要約を読んだうえで行って
ください。
