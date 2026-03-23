# OSC Conformance Corpus Plan

## 目的

この文書は、将来の parser、encoder、interoperability test が使う
OSC conformance corpus の構造と初期 seed を定義します。

## Corpus Root

repository の fixture root は次です。

- `fixtures/conformance/`

主な inventory file:

- `fixtures/conformance/catalog.json`
- `fixtures/conformance/vectors/*.hex.txt`

## Coverage Goal

corpus は次をカバーするべきです。

- `osc1_0_strict`
- `osc1_0_legacy_tolerant`
- `osc1_1_extended`
- valid message と bundle
- malformed alignment、truncation、size case
- legacy missing-type-tag traffic
- extension と unknown-tag behavior

## Catalog Field

catalog entry には次を記録します。

- vector ID
- compatibility mode
- category
- expected disposition
- source reference
- fixture path
- short description

## Seed Category

- basic scalar message
- string と blob case
- bundle framing
- legacy no-type-tag payload
- malformed padding と truncation
- extended boolean と nil tag
- unknown extension tag

## Acceptance Rule

conformance corpus は benchmark corpus ではありません。
correct、safely rejected、legacy mode でのみ tolerated のどれかを
明示するためのものです。

## 初期 Seed Inventory

最初の seed inventory は次に置きます。

- `fixtures/conformance/catalog.json`

この seed があれば、最終的な自動化形式をまだ確定していなくても、
parser / encoder test の立ち上がりには十分です。
