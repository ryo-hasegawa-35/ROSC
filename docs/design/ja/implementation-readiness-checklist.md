# Implementation Readiness Checklist

## 目的

この checklist は、将来の implementation が即興ではなく stable な設計前提
から始まるようにするためのものです。

意図的に、coding を始める前提で書かれています。

## Any Core Coding の前

次の文書を読んで current として受け入れていることを確認します。

- [GitHub Foundation And Collaboration Plan](../../concepts/ja/github-foundation-and-collaboration-plan.md)
- [Architecture Principles](./architecture-principles.md)
- [Internal Packet And Metadata Model](./internal-packet-and-metadata-model.md)
- [Fault Model And Overload Behavior](./fault-model-and-overload-behavior.md)
- [Recovery Model And Cache Semantics](./recovery-model-and-cache-semantics.md)
- [Compatibility Matrix](./compatibility-matrix.md)
- [Route Configuration Grammar](./route-configuration-grammar.md)

## Repository / GitHub Setup の前

確認事項:

- protected branch expectation が文書化されている
- `CODEOWNERS` に落とせる程度に review ownership boundary が明確
- issue / label taxonomy が architecture area と対応している
- documentation quality check を先に入れる計画がある

## Parser / Encoder Work の前

確認事項:

- compatibility mode を理解している
- legacy missing-type-tag policy を受け入れている
- unknown tagged value behavior を受け入れている
- raw packet retention policy が明確

## Route Authoring または Example Publication の前

確認事項:

- route grammar と cookbook example が一致している
- example が unsupported field を normative に見せていない
- route ID / destination ID の naming discipline が揃っている

## Routing Core Work の前

確認事項:

- normalized packet model が十分 stable
- route grammar が十分 stable
- cookbook example が想定する hot-path use case を反映している
- traffic class が合意済み
- queue boundary が明示されている
- overload action が明示されている

## Observability Work の前

確認事項:

- operator question が分かっている
- health state が fault model と一致している
- telemetry level と canonical metric name が合意済み
- cardinality limit が明示されている
- replay と rehydrate が分離されている
- diagnostics budget が bounded

## Benchmarking または Performance Claim の前

確認事項:

- workload definition と interpretation guide の両方が current
- benchmark context field が定義済み
- comparison rule を理解している
- release claim で speed と trustworthiness を区別できる

## Recovery Work の前

確認事項:

- cache class が定義済み
- 危険な trigger traffic が non-automatic として扱われる
- warm / durable persistence の違いが理解されている
- route-level recovery policy が存在する

## Plugin / Adapter Work の前

確認事項:

- plugin trust tier が定義済み
- adapter capability contract を受け入れている
- security boundary は broker-owned のまま
- plugin failure containment が明示されている

## Discovery / Metadata Work の前

確認事項:

- discovery が manual configuration を bypass しない
- trust level が明示されている
- service metadata shape を受け入れている
- stale discovery handling が定義されている

## Schema / Codegen Work の前

確認事項:

- schema は optional のまま
- schema type system が packet model と整合している
- code generation target が scoped されている
- schema validation level が理解されている

## SDK / External Adapter Work の前

確認事項:

- adapter SDK contract が存在する
- capability declaration vocabulary が十分 stable
- ownership rule が明示されている
- interoperability evidence expectation が分かっている

## Native IPC Work の前

確認事項:

- fallback path がある
- convenience より host-facing ABI stability を優先している
- shared memory ownership rule を受け入れている
- IPC 向け observability requirement がある

## Distributed / HA Work の前

確認事項:

- broker identity model がある
- replication scope が定義済み
- split-brain prevention model がある
- failover trigger policy がある

## Config Hot Reload Work の前

確認事項:

- validation stage が定義済み
- last-known-good policy がある
- migration visibility が保たれている
- risky apply に review または confirmation rule がある

## Packaging / Release Work の前

確認事項:

- deployment topology が文書化されている
- release profile content が文書化されている
- profile ごとの fallback story がある
- profile ごとの testing expectation が書かれている

## Compatibility Claim を出す前

確認事項:

- conformance vector が存在する
- interoperability scenario が存在する
- regression policy が定義されている
- release note が実際の evidence と対応している

## Architecture-Changing Work の前

確認事項:

- ADR index が対象 decision area を反映している
- non-trivial な semantic shift には proposed または accepted ADR がある
- affected design document が明示的に列挙されている

## Ready-To-Code Gate

coding は次を満たしたときに始めるべきです。

- 影響範囲の design doc が存在する
- term が一貫している
- failure behavior が明示されている
- fallback behavior が明示されている
- non-negotiable invariant が明確
- repository review discipline が design intent を壊さない

## Red Flag

次のいずれかに当てはまるなら implementation を一度止めるべきです。

- undocumented adapter behavior に route semantics が依存している
- recovery behavior を文書ではなく想像で補っている
- review なしに compatibility を弱める performance optimization が必要
- plugin が unrestricted broker internals を必要としている
- local acceleration path が mandatory になりそう
- repository process のままだと high-risk merge を architecture review なしで通せてしまう
