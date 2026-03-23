# Rust Workspace And Crate Boundaries

## 目的

この文書は、coding を始める前に Rust workspace の形を凍結するためのものです。

## Boundary Goal

- hot path を小さく保ち、依存を重くしない
- protocol semantics と runtime orchestration を分ける
- adapter、recovery、telemetry が parser correctness を汚さないようにする
- optional feature を optional のまま保つ

## 提案する Top-Level Layout

```text
/Cargo.toml                     workspace only
/crates/rosc-osc               OSC parse / encode primitive
/crates/rosc-packet            内部 packet / metadata model
/crates/rosc-route             route matching と routing decision
/crates/rosc-runtime           ingress、scheduling、egress isolation
/crates/rosc-config            configuration loading と semantic validation
/crates/rosc-telemetry         metrics、event、health export
/crates/rosc-recovery          cache、rehydrate、replay policy
/crates/rosc-adapter-sdk       adapter 向け stable contract
/crates/rosc-plugin-sdk        plugin 向け stable contract
/crates/rosc-security          additive security overlay service
/apps/rosc-broker              broker executable
/apps/rosc-dashboard-api       optional な dashboard backend surface
/fixtures/                     conformance と benchmark input
/docs/                         architecture と planning document
```

## Phase 01 の最小 Bootstrap Set

最初の coding window で実体化すべきなのは次だけです。

- `rosc-osc`
- `rosc-packet`
- `rosc-route`
- `rosc-runtime`
- `rosc-config`
- `rosc-telemetry`
- `apps/rosc-broker`

それ以外は、core が安定を示すまで planned のままにします。

## Responsibility Matrix

- `rosc-osc`
  - OSC の byte-level parse / encode behavior を持つ
  - runtime、telemetry、recovery crate へ依存しない
- `rosc-packet`
  - normalized packet representation と raw metadata retention を持つ
  - `rosc-osc` には依存できるが、runtime orchestration には依存しない
- `rosc-route`
  - route evaluation、destination selection、traffic-class decision を持つ
  - transport-specific adapter code から独立させる
- `rosc-runtime`
  - ingress queue、task orchestration、egress isolation、breaker flow を持つ
  - route、packet、telemetry、security boundary に依存できる
- `rosc-config`
  - external config parsing、semantic validation、last-known-good logic を持つ
  - hidden default なしで config を route/runtime structure へ落とす
- `rosc-telemetry`
  - metric name、event emission、health-reporting surface を持つ
  - bounded に保ち、routing decision 自体は持たない

## Dependency Rule

- adapter は core へ inward に依存し、逆方向依存は作らない
- telemetry は runtime に利用されるが、decision engine にしない
- recovery は packet、route、runtime、config semantics に依存できるが、
  parser correctness を recovery に依存させない
- plugin / adapter SDK crate は stable boundary を公開するが、
  unrestricted core internals は出さない

## Coding 前に凍結する Decision

- workspace は library-first とし、最初の executable は broker app 1 つにする
- UDP / OSC compatibility は core crate で実装し、external plugin に押し出さない
- dashboard UI は runtime internals へ直接依存させない
- IPC、federation、高度 adapter は後続 crate とし、Phase 01 の必須条件にしない

## Non-Goal

この文書はまだ workspace を実体化しません。
最初の implementation PR が境界を即興で決めないように、crate boundary を
定義するためのものです。
