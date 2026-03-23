# Release Note と Changelog の方針

## 目的

この文書は、ROSC が実装前フェーズと実装開始後の両方で、release、preview、
主要な repository milestone をどう記述するかを定義します。

狙いは、release communication を evidence-based で
compatibility-aware、operator にも役立つものにすることです。

## 実装前フェーズのルール

production code がまだない間、changelog は runtime release があるふりをせず、
repository と設計土台に関する主要 milestone を記録します。

たとえば次を含めます。

- governance の節目
- design baseline の節目
- CI や fixture readiness の節目
- 主要な documentation / workflow hardening

## Changelog の役割

root の changelog は、すべての commit を列挙するのではなく、将来読む人に
関係する milestone を要約するために使います。

未来の読者が少なくとも次に答えられるようにします。

- この repository phase で何が変わったか
- それがなぜ重要だったか
- 次の実装フェーズが何を引き継いだか

## Release Note の役割

実行可能 artifact ができた後の release note は、少なくとも次を説明するべき
です。

- 何が変わったか
- compatibility への影響は何か
- operator-visible な影響は何か
- 何の evidence があるか
- rollback / fallback はどうするか

## 必須セクション

将来の release note には、少なくとも次を含めます。

- release の要約
- 影響する profile や deployment topology
- compatibility impact
- observability / operator workflow への影響
- recovery / rollback の注意点
- evidence へのリンク
- known limitation

## Evidence Rule

必要に応じて、release note から実際の evidence にリンクできるようにします。

- benchmark
- conformance result
- CI run
- design doc
- issue / ADR trail

ROSC は、根拠をたどれない marketing 的な主張を release note に書くべきでは
ありません。

## Compatibility Rule

compatibility に触れる release note では、その変更が次のどれに効くのかを
明記します。

- strict behavior
- legacy-tolerant behavior
- extended behavior
- compatibility semantics には影響しない

## Changelog の保守ルール

- entry は concise に保つ
- commit 単位ではなく milestone 単位で書く
- ノイズ的な列挙は避ける
- 英語版と日本語版の changelog を同時に更新する

## GitHub Release Automation の扱い

GitHub 自動生成の release note は分類補助としては便利ですが、最終的な編集層
ではありません。

人間の確認で、少なくとも次を見ます。

- compatibility-sensitive な項目が正直に強調されているか
- operator-visible な risk が埋もれていないか
- release note が実際の evidence trail と対応しているか

## 最低限守ること

ROSC の release communication は、単に見栄えを良くするためではなく、
project を信頼しやすくするためのものであるべきです。
