# Security Policy

## Scope

ROSC は将来的に network-facing infrastructure software になる前提です。
そのため security issue には memory safety bug だけでなく、次のようなものも含みます。

- source verification の bypass
- identity や scope の混線
- replay や recovery の abuse path
- unsafe default configuration
- plugin / adapter trust boundary の逸脱
- cross-broker failover や federation の安全性不備

## Vulnerability の報告方法

実運用で悪用されうる vulnerability が疑われる場合は、
public issue を作らないでください。

推奨手順:

1. GitHub の private vulnerability reporting が使えるなら、それを使う
2. private reporting が使えない場合は、公開前に GitHub 経由で repository owner へ連絡する

## 含めてほしい情報

可能なら次を含めてください。

- 影響する component または design area
- 再現手順
- expected behavior と actual behavior
- deployment context
- compatibility、safety、data isolation への影響
- workaround の有無

## Disclosure の考え方

- report は可能な限り早く acknowledge する
- public speed より safe remediation を優先する
- maintainer が十分に調査・対応するまで、public disclosure は待つ

## 現在のプロジェクト段階

この repository はまだ pre-implementation 段階です。
そのため security issue は production code ではなく、design や process の形で
現れることがあります。それでも safety に実質的な影響があるなら、
private に報告する価値があります。
