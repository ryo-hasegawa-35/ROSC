# Deployment Topology And Release Profile Guide

## 目的

この文書は、製品がサポートすべき主要 deployment shape と、それに対応する
release profile を定義します。

目的は、すべてに向けた肥大 binary を 1 本だけ出すことではなく、現実の運用に
合った package を明示的に設計することです。

## 設計目標

- よくある deployment shape を明示する
- package 内容を現実の operator need に合わせる
- compatibility-first profile を残す
- safe fallback path を守る

## Topology Level

### Topology A: Localhost Sidecar

向いている場合:

- 1 台の machine で UE5、TouchDesigner、その他 OSC tool が動く
- project logic を壊さず broker を差し込む

特徴:

- localhost transport が中心
- 最も friction の少ない導入経路
- first deployment model として理想的

### Topology B: Single Workstation Hub

向いている場合:

- 1 台の machine が複数の local / network peer 間を中継する

特徴:

- local と network transport の混在
- dashboard を使うことが多い
- light discovery が有用

### Topology C: Dual-Machine Show Pair

向いている場合:

- 1 台が creative software
- もう 1 台が routing、observability、operator control を担当

特徴:

- 明確な network boundary
- より強い health visibility が必要
- route separation の価値が高い

### Topology D: Segmented Installation Network

向いている場合:

- 複数の device、sensor、operator console、media node が協調する

特徴:

- discovery と service metadata の重要度が上がる
- security scope の重要度が上がる
- route / namespace 管理が中心課題になる

### Topology E: Active / Standby Pair

向いている場合:

- 最小複雑性より continuity を優先する

特徴:

- replicated config と selected state
- explicit failover handling

### Topology F: Federated Brokers

向いている場合:

- 複数 broker node が site または network segment をまたいで selected traffic
  / state を共有する

## Release Profile

### `core-osc`

含むもの:

- OSC routing core
- compatibility mode
- basic metric

向いている場合:

- 最小の強い土管が目的

### `ops-console`

含むもの:

- core-osc
- dashboard
- capture / replay
- operator recovery tool

向いている場合:

- visibility と recovery を重視する

### `browser-control`

含むもの:

- ops-console
- WebSocket / JSON adapter

向いている場合:

- browser-facing monitoring / control surface が必要

### `ue5-workstation`

含むもの:

- ops-console
- localhost performance preset
- 十分成熟していれば optional native IPC piece

向いている場合:

- UE5 中心の local workflow が主体

### `touchdesigner-kiosk`

含むもの:

- ops-console
- high-rate stream tuning preset
- 強い recovery default

向いている場合:

- TouchDesigner 中心の sensor / show work が主体

### `secure-installation`

含むもの:

- ops-console
- security overlay
- 強めの audit default
- controlled discovery profile

向いている場合:

- shared または semi-hostile network を含む

### `lab-dev`

含むもの:

- 広い feature set
- diagnostics-heavy default
- experimental capability の可視化

向いている場合:

- development、benchmarking、exploration が優先

## Profile Rule

- すべての profile は何を含まないかも明記する
- すべての profile は compatibility-first fallback story を持つ
- risky experimental feature が production-first profile に紛れ込まない

## Operator Guidance

各 profile は少なくとも次を文書化すべきです。

- intended use case
- included adapter
- included observability feature
- security posture
- recommended topology

## Upgrade And Rollback

release guidance が定義すべきもの:

- どの profile が upgrade-compatible か
- config migration をどう扱うか
- rollback path は何か

## 非交渉の不変条件

- release profile が compatibility requirement を隠してはならない
- topology guidance は architecture 自体より単純であること
- advanced profile には clear fallback path が必要
- 最小限有用な compatible deployment を first-class に保つ
