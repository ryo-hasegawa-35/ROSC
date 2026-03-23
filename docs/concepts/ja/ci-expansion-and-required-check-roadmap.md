# CI Expansion And Required-Check Roadmap

## 目的

この文書は、production Rust code が入る前に必要な repository automation の
baseline と、実装開始後に required check をどう広げるかを定義します。

## 現在の Baseline

repository は現在、次の workflow を使います。

- `Docs Quality`
- `PR Governance`
- `Repository Sanity Matrix`

これらは documentation integrity、pull request structure、
cross-platform repository readiness を守るためのもので、まだ Rust 実装が
存在するふりはしません。

## 実装前の Required Check

`main` では、少なくとも次を required に保ちます。

- `docs-consistency`
- `pr-body-policy`

cross-platform matrix も PR で走らせますが、required にするかは signal の
安定性を見て maintainer が判断します。

## Cross-Platform Readiness Rule

Rust code がなくても、CI は少なくとも次を証明するべきです。

- Windows、macOS、Linux で repository checkout が成立する
- required governance file が存在する
- root の主要文書に英日版がある
- fixture manifest と ADR tree が存在し、parse できる

## 最初の Code 時代の Workflow 分割

最初の Rust workspace file が入ったら、CI は次の lane に広げるべきです。

1. repository / docs quality
2. formatting / lint
3. unit / integration test
4. conformance corpus validation
5. benchmark / fuzz evidence
6. release packaging と signing evidence

## Required Check の拡張

最初の code phase が安定したら、`main` では次を required 候補にします。

- `docs-consistency`
- `pr-body-policy`
- `repo-sanity (ubuntu-latest)`
- `repo-sanity (windows-latest)`
- `repo-sanity (macos-latest)`
- 将来追加される最初の Rust formatting / test job

## Naming Rule

required check になる workflow / job 名は、安易に変えないべきです。
required status context の rename は casual cleanup ではなく、
repository-governance change として扱います。

## Non-Goal

この roadmap は、まだ次までは定義しません。

- benchmark threshold
- release signing の実装
- 将来の package publishing に向けた secret-management policy

それらは後続の implementation / release phase で決めます。
