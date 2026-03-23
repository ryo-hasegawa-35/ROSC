# ADR-0003: Route Semantic Model Before File Format

- Status: accepted
- Date: 2026-03-23

## Context

external configuration format の詳細が先に固定される前に、
stable な route meaning が必要です。

## Decision

- route semantics を normative とし、external file syntax はその次に置く
- 最初の external configuration format は TOML を使う
- apply 前に semantic validation を必須にする
- route ID と destination ID は stable で explicit に保つ

## Consequences

- 将来 config format を変えても route meaning は保ちやすい
- hot reload と last-known-good safety を semantics 基準で守れる
- example は format-aware でありつつ format-owned にならない

## Rejected Alternatives

- TOML structure をそのまま normative definition にすること
- route behavior を runtime code path へ直接埋め込むこと

## Affected Documents

- [Route Configuration Grammar](../../ja/route-configuration-grammar.md)
- [Route Rule Cookbook And Worked Examples](../../ja/route-rule-cookbook-and-worked-examples.md)
- [Config Validation And Migration Note](../../ja/config-validation-and-migration-note.md)
