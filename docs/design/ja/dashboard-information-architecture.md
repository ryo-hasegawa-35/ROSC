# Dashboard Information Architecture

## 目的

この文書は、normal / abnormal な状態でも operator が broker を素早く理解
できるよう、dashboard が情報をどう構造化すべきかを定義します。

dashboard は装飾ではなく、operating model の一部です。

## 最初に答えるべき問い

dashboard はまず次の問いに答えるべきです。

- システムは健全か
- どこに pressure がたまっているか
- 何が drop されているか
- どの route / destination が不健全か
- 何が最近変わったか
- どう安全に復旧するか

## Information Hierarchy

トップレベルの情報順:

1. system health
2. active incident
3. route / destination status
4. recent change
5. deeper forensic tool

## Core View

### Overview

表示すべきもの:

- global state
- packet rate
- queue pressure summary
- top warning
- active breaker
- active quarantine

### Topology

表示すべきもの:

- ingress node
- route
- transform
- destination
- adapter state

### Routes

表示すべきもの:

- route class
- mode
- match pattern
- queue depth
- drop count
- cache policy
- recovery action

### Destinations

表示すべきもの:

- destination health
- transport type
- egress queue
- breaker state
- last error

### Traffic / Forensics

表示すべきもの:

- packet timeline
- filtered capture view
- replay session status
- correlation lookup

### Recovery

表示すべきもの:

- cache state
- rehydrate candidate
- last recovery action
- safe mode state

### Security

表示すべきもの:

- active identity
- denied event
- scope mismatch
- secure route status

## Status Model

dashboard は fault model と同じ top-level state を使うべきです。

- `Healthy`
- `Pressured`
- `Degraded`
- `Emergency`
- `SafeMode`

## Entity Model

dashboard が first-class entity として扱うべきもの:

- ingress
- adapter
- route
- transform
- destination
- packet capture session
- replay session
- cache namespace
- security scope

## Event Timeline

operator には単一の time-ordered event view が必要です。

- config change
- breaker event
- quarantine event
- replay action
- rehydrate action
- adapter disconnect
- safe mode entry

## Action Design Rule

operator action は次を満たすべきです。

- visible
- reversible when possible
- scoped
- audited

高インパクト action:

- isolate route
- disable destination
- resend cached state
- start replay
- enter safe mode

これらは scope を明示し、明確な確認を要求すべきです。

## Progressive Disclosure

dashboard は最初からすべてを見せるべきではありません。

推奨パターン:

- first overview
- unhealthy item へ drill down
- forensic tool は必要時に開く

## Data Freshness Rule

dashboard は次を明確に区別する必要があります。

- live value
- cached value
- stale value
- replay traffic

## Failure Visibility Rule

特に強調すべきもの:

- hidden queue growth risk
- critical route への pressure
- adapter disconnect storm
- repeated transform failure
- recovery に影響する stale cache

## 非交渉の不変条件

- dashboard は metric 表示だけでなく action guide を提供する
- forensic detail より先に health state を見せる
- replay と live traffic を視覚的に区別する
- operator action は audit trail を残す
- dashboard refresh が hot path を不安定化させない
