# Maintainer Approval と Merge Behavior

## 目的

この文書は、特に repository を単一 owner account で運用している間に、
GitHub 上で maintainer がどのような review / merge 挙動を見るかを説明します。

## 現在の ROSC 設定

この文書を書いた時点で、`main` には次の保護が入っています。

- pull request 必須
- required status check 必須
- conversation resolution 必須
- approving review 1 件必須
- code owner review 必須
- push で stale review を dismiss
- admin enforcement は無効

この設定は、`main` を保護しつつ、必要なら repository owner が緊急対応できる
余地を残すためのものです。

## なぜ GitHub に bypass が出るのか

現在の ROSC 設定で bypass が出るのは普通です。理由は 2 つあります。

### 1. maintainer account が admin である

admin enforcement が無効な場合、repository admin は branch rule を bypass
できます。

### 2. 自分で作成した pull request への自分の approval は、独立した required approval としては数えられない

pull request を作った GitHub account と、approval を付けようとしている
account が同じ場合、GitHub はそれを独立した required approval として扱い
ません。

この repository では、ローカル自動化や Codex からの PR も現在は maintainer の
GitHub credential を使っているため、PR author が maintainer account として
見えることが多く、特に起きやすいです。

## これは普通なのか

はい、単一 owner の repository では普通です。

つまり、次が同時に起きても不自然ではありません。

- required review rule は設定されている
- しかし独立した approval を出せる別 account がいない
- その結果、GitHub は admin bypass を使った merge を提示する

これは branch protection が壊れているのではなく、GitHub が

- 独立した review
- repository owner の administrative override

をきちんと区別しているということです。

## 現在の ROSC におすすめの運用

第二 reviewer identity がない間は、実務上は次の流れが自然です。

1. required check がすべて通るのを待つ
2. PR の内容をしっかり読む
3. 必要なら comment や request changes を出す
4. 問題ないと判断したら merge する

この構成では、merge の判断そのものが maintainer の最終承認行為になります。

## 将来もっと厳密な review モデルにしたいなら

GitHub の approval requirement を、本当に bypass なしの独立 review gate として
動かすには、次のどれかが必要です。

- 第二の人間 reviewer account や team を追加する
- PR author を別の bot / service account にする
- 独立 approval の経路を用意したうえで admin enforcement を有効にする

逆に言うと、単一 account の運用では、同じ account が作った PR に対して
同じ account が独立 reviewer を兼ねることはできません。

## ROSC が誤解してはいけないこと

ROSC では、次を誤魔化してはいけません。

- self-authored PR への self-approval が independent review と同じである
- bypass and merge が出るのは GitHub の不具合である
- required review を入れれば、単一 account 運用でも separation of duties が成立する

## Maintenance Rule

branch protection setting が大きく変わったら、この文書と関連する
repository-governance 文書を同じ PR で更新してください。
