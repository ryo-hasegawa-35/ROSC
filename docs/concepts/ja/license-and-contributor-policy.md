# License And Contributor Policy

## 目的

この文書は、実装前段階で決めた repository のライセンス方針と contributor
expectation を記録するものです。

## Decision

- repository は MIT License で公開する
- contribution は、特に別ルールを定めない限り MIT 条件で受け入れる
- contributor は、自分が MIT で再許諾できない code や asset を持ち込まない

## MIT を選ぶ理由

現在のプロジェクト段階では、MIT が最も適しています。理由:

- artist、studio、researcher、toolmaker が導入しやすい
- Rust、UE5、TouchDesigner、web 系の ecosystem と相性がよい
- 実装がまだ大きくなる前に repository を法的に使いやすくできる
- architecture-heavy な段階で collaboration を余計に遅らせない

## Contributor Expectation

- substantial な作業は issue 起点で始める
- normative な design change は ADR を参照するか提案する
- `docs/concepts/` と `docs/design/` の変更では英日ペアを保つ
- security-sensitive な話題は `SECURITY.md` の方針どおり private に報告する

## Non-Goal

今回の decision では、まだ次は定めません。

- 別途の CLA process
- trademark policy
- dual licensing
- commercial support terms

これらは将来必要になった時点で追加できます。

## 影響する Repository File

- [LICENSE](../../../LICENSE)
- [README.md](../../../README.md)
- [README.ja.md](../../../README.ja.md)
- [CONTRIBUTING.md](../../../CONTRIBUTING.md)
- [CONTRIBUTING.ja.md](../../../CONTRIBUTING.ja.md)

## Review Rule

将来 relicensing、contributor policy 変更、CLA 導入を検討するなら、
silent な repository edit ではなく、明示的な architecture / governance
decision として扱うべきです。
