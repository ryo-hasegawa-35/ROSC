# Federation And High-Availability Model

## 目的

この文書は、複数 broker が redundancy、segmentation、より大きな deployment
のためにどう協調するかを定義します。

## 設計目標

- まず local installation を支える
- semantics を書き換えず multi-node system へ伸ばす
- federation と failover を明確に分ける
- split-brain ambiguity を避ける

## 2 つの異なるモデル

### Federation

向いている場合:

- 複数 broker が selected traffic を意図的に交換する
- 異なる network segment や site が協調する

### High Availability

向いている場合:

- 1 つの broker が落ちても別 broker が service を継続したい
- horizontal expansion より route continuity が重要

## Broker Identity

各 broker が持つべきもの:

- stable broker ID
- instance ID
- deployment scope
- advertised capability

## Federation Link

federation link が定義すべきもの:

- peer identity
- allowed scope
- replicated route または namespace
- transport security mode
- health state

## Replication Scope

すべてを replicate すべきではありません。

候補 class:

- config only
- selected cache state
- selected journal state
- selected live traffic
- discovery metadata

## Active / Standby Model

最初の HA model として推奨:

- active broker
- standby broker
- replicated config
- replicated selected cache
- explicit failover trigger

## Failover Trigger

候補 trigger:

- active broker health loss
- threshold を超える transport loss
- operator action
- host process crash

## Split-Brain Prevention

同じ active role を 2 broker が黙って主張する状態は避けるべきです。

推奨 control:

- explicit role state
- lease または heartbeat mechanism
- operator-visible arbitration state
- authority が不確実なら safe fail-closed

## State Continuity

failover を意味あるものにするため、standby は十分な state を持つべきです。

良い候補:

- route graph
- selected cache entry
- selected journal
- 必要に応じた security scope state

## Recovery After Failover

failover 後:

- 新 active broker は continuity transition を明示する
- rehydrate rule は route-aware のまま保つ
- replay は設定がない限り自動で走らせない

## Federation Security

broker-to-broker traffic では次を保つべきです。

- peer identity
- authorized scope
- auditability

federation は route security を回避する unbounded bypass になってはなりません。

## Observability

operator が見えるべきもの:

- active / standby role
- peer health
- replication lag
- failover history
- split-brain risk indicator

## 非交渉の不変条件

- federation と failover は explicit であること
- replicated state は lineage と scope を保つこと
- authority が不確実なら duplicate active broker を黙って生まないこと
- single-broker deployment は今後も first-class で単純なままであること
