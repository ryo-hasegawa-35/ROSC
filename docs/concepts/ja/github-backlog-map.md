# GitHub Backlog Map

## 目的

この文書は、現在の GitHub backlog がどう構成されているかを記録し、
あとから参加する contributor が Issues タブだけを手探りで追わなくても
全体像をつかめるようにするためのものです。

関連文書:

- [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
- [Detailed Delivery Plan](./detailed-delivery-plan.md)
- [Design Reading Order](../../design/ja/reading-order.md)

## 現在の Repository Governance Baseline

現在の repository には、少なくとも次の基盤があります。

- `main` が実体化され、default branch になっている
- `main` への merge は pull request 前提
- 新しい push が入ると stale review を破棄する
- protected branch では code owner review を要求する
- `CODEOWNERS` は現在 `@ryo-hasegawa-35` を指している
- conversation resolution が必須
- `main` への force-push と deletion は無効
- `Docs Quality` と `PR Governance` workflow が存在する

重要な補足:

- admin bypass は完全には強制していません
- これにより、通常の PR では owner approval が必要なまま、最終的な merge
  の主導権は repository owner に残ります

## 現在の Project 状態

Issue tracking と project tracking は、どちらもすでに使える状態です。

現在の active project は次です。

- [ROSC Delivery Board](https://github.com/users/ryo-hasegawa-35/projects/3)

現在の project baseline:

- repository は project にリンク済み
- 現在の backlog issue はすべて project に投入済み
- active な pull request も同じ project で追跡できる
- `Phase`、`Priority`、`Area` field は repository taxonomy に合わせて投入済み
- repository 側の doc に roadmap、active work、blocked work の見方を残してある

補足:

- [Issue #6](https://github.com/ryo-hasegawa-35/ROSC/issues/6) は、
  documented filter-based view で十分とするか、GitHub UI で named saved view
  まで作るかを決めるまで open のままにしています

## Milestone 一覧

- `Phase 00 - Foundation And Governance`
- `Phase 01 - Core Proxy And Routing`
- `Phase 02 - Observability And Recovery`
- `Phase 03 - Adapters And Discovery`
- `Phase 04 - Extensibility And Schema`
- `Phase 05 - Native Integration`
- `Phase 06 - Security, Sync, And Release`

## Epic 一覧

- [Issue #34](https://github.com/ryo-hasegawa-35/ROSC/issues/34)
  `Phase 00 foundation and governance`
- [Issue #35](https://github.com/ryo-hasegawa-35/ROSC/issues/35)
  `Phase 01 core proxy and routing`
- [Issue #36](https://github.com/ryo-hasegawa-35/ROSC/issues/36)
  `Phase 02 observability and recovery`
- [Issue #37](https://github.com/ryo-hasegawa-35/ROSC/issues/37)
  `Phase 03 adapters and discovery`
- [Issue #38](https://github.com/ryo-hasegawa-35/ROSC/issues/38)
  `Phase 04 extensibility and schema`
- [Issue #39](https://github.com/ryo-hasegawa-35/ROSC/issues/39)
  `Phase 05 native integration`
- [Issue #40](https://github.com/ryo-hasegawa-35/ROSC/issues/40)
  `Phase 06 security, sync, and release`

## Phase ごとの Task Map

### Phase 00

- [Issue #1](https://github.com/ryo-hasegawa-35/ROSC/issues/1)
  `Decide repository license and contributor policy`
- [Issue #3](https://github.com/ryo-hasegawa-35/ROSC/issues/3)
  `Materialize the initial ADR set from the design index`
- [Issue #8](https://github.com/ryo-hasegawa-35/ROSC/issues/8)
  `Define the Rust workspace and crate boundaries for the broker core`
- [Issue #2](https://github.com/ryo-hasegawa-35/ROSC/issues/2)
  `Build the OSC conformance corpus from the 1.0 and 1.1 references`
- [Issue #7](https://github.com/ryo-hasegawa-35/ROSC/issues/7)
  `Create benchmark fixtures and reproducible workload inputs`
- [Issue #5](https://github.com/ryo-hasegawa-35/ROSC/issues/5)
  `Expand GitHub Actions into cross-platform repository quality and future Rust CI scaffolding`
- [Issue #6](https://github.com/ryo-hasegawa-35/ROSC/issues/6)
  `Enable a GitHub Project board and seed the delivery views`

### Phase 01

- [Issue #11](https://github.com/ryo-hasegawa-35/ROSC/issues/11)
  `Implement the OSC compatibility parser and encoder core`
- [Issue #10](https://github.com/ryo-hasegawa-35/ROSC/issues/10)
  `Implement ingress transport bindings and bounded intake queues`
- [Issue #9](https://github.com/ryo-hasegawa-35/ROSC/issues/9)
  `Implement the route matcher and routing engine`
- [Issue #12](https://github.com/ryo-hasegawa-35/ROSC/issues/12)
  `Implement destination workers, circuit breakers, and fault isolation`
- [Issue #14](https://github.com/ryo-hasegawa-35/ROSC/issues/14)
  `Implement configuration loading, semantic validation, and safe hot reload`
- [Issue #13](https://github.com/ryo-hasegawa-35/ROSC/issues/13)
  `Implement the minimal metrics endpoint and health-reporting surface`

### Phase 02

- [Issue #15](https://github.com/ryo-hasegawa-35/ROSC/issues/15)
  `Implement the operations dashboard shell and core runtime pages`
- [Issue #33](https://github.com/ryo-hasegawa-35/ROSC/issues/33)
  `Implement cache classes and the late-joiner rehydrate engine`
- [Issue #16](https://github.com/ryo-hasegawa-35/ROSC/issues/16)
  `Implement bounded capture, replay, and operator recovery auditing`

### Phase 03

- [Issue #19](https://github.com/ryo-hasegawa-35/ROSC/issues/19)
  `Implement the WebSocket / JSON adapter`
- [Issue #18](https://github.com/ryo-hasegawa-35/ROSC/issues/18)
  `Implement the MQTT adapter`
- [Issue #20](https://github.com/ryo-hasegawa-35/ROSC/issues/20)
  `Implement the discovery and service-metadata runtime`
- [Issue #21](https://github.com/ryo-hasegawa-35/ROSC/issues/21)
  `Implement adapter interoperability and conformance harnesses`

### Phase 04

- [Issue #23](https://github.com/ryo-hasegawa-35/ROSC/issues/23)
  `Implement the plugin capability registry and external plugin boundary`
- [Issue #24](https://github.com/ryo-hasegawa-35/ROSC/issues/24)
  `Implement the Wasm transform runtime and hot-reload lifecycle`
- [Issue #22](https://github.com/ryo-hasegawa-35/ROSC/issues/22)
  `Implement the schema parser, validator, and compatibility-aware type model`
- [Issue #25](https://github.com/ryo-hasegawa-35/ROSC/issues/25)
  `Implement code generation targets for UE5 and TouchDesigner`

### Phase 05

- [Issue #27](https://github.com/ryo-hasegawa-35/ROSC/issues/27)
  `Implement the stable C ABI and shared-memory IPC transport`
- [Issue #28](https://github.com/ryo-hasegawa-35/ROSC/issues/28)
  `Build the UE5 native integration package`
- [Issue #26](https://github.com/ryo-hasegawa-35/ROSC/issues/26)
  `Build the TouchDesigner native integration package`

### Phase 06

- [Issue #31](https://github.com/ryo-hasegawa-35/ROSC/issues/31)
  `Implement the security overlay and verified-source enforcement`
- [Issue #29](https://github.com/ryo-hasegawa-35/ROSC/issues/29)
  `Implement timing diagnostics and advanced sync groundwork`
- [Issue #30](https://github.com/ryo-hasegawa-35/ROSC/issues/30)
  `Implement broker identity, federation, and active-standby control plane`
- [Issue #32](https://github.com/ryo-hasegawa-35/ROSC/issues/32)
  `Implement packaging, release profiles, and the CI evidence pipeline`

## 読み始めのおすすめ順

新しく repo に入る人は、次の順で見ると効率が良いです。

1. `README.md`
2. [GitHub Foundation And Collaboration Plan](./github-foundation-and-collaboration-plan.md)
3. [Design Reading Order](../../design/ja/reading-order.md)
4. 自分が触る phase の epic issue
5. 実際に着手する child task issue

## Maintenance Rule

backlog item が増えたり、split / merge / close によって planning shape が
大きく変わったときは、この map も更新するべきです。
