# Dashboard Interaction Spec And Screen Inventory

## 目的

この文書は、dashboard information architecture を拡張し、interaction の
期待値と主要 screen inventory を定義します。

information architecture が「何をどこに置くか」に答える文書なら、
この文書は「operator がどう移動し、どう操作するか」に答える文書です。

## 設計目標

- fast incident handling を支える
- pressure 下の operator confusion を減らす
- high-impact action を明示的にする
- navigation を operational model に合わせる

## Screen Inventory

### Screen 1: Overview

主用途:

- instant system status

表示必須:

- current global state
- top incident
- packet throughput summary
- active breaker / quarantine count
- unhealthy entity への quick link

### Screen 2: Topology

主用途:

- system structure の理解

表示必須:

- ingress node
- route
- transform
- destination
- adapter relationship

### Screen 3: Route Detail

主用途:

- route-level diagnosis と action

表示必須:

- match definition
- mode と class
- queue state
- fault policy
- cache / recovery policy
- destination fan-out

action:

- isolate route
- resend cached state
- inspect recent history

### Screen 4: Destination Detail

主用途:

- destination 固有問題の diagnosis

表示必須:

- transport と adapter
- current health
- queue pressure
- breaker state
- recent error

action:

- disable destination
- retry / failure の inspect

### Screen 5: Traffic / Forensics

主用途:

- traffic inspection と replay setup

表示必須:

- filtered timeline
- packet detail
- lineage marker
- capture session control

action:

- filtered capture の作成
- sandbox replay の起動

### Screen 6: Recovery

主用途:

- controlled restoration と cache management

表示必須:

- cache namespace
- stale warning
- rehydrate candidate
- recent recovery action

action:

- route rehydrate
- namespace snapshot rehydrate
- stale cache invalidate

### Screen 7: Security

主用途:

- trust と access の visibility

表示必須:

- active identity
- denied event
- scope mismatch
- secure route status

### Screen 8: Config / Change Review

主用途:

- configuration を安全に review / apply

表示必須:

- pending diff
- validation finding
- risk summary
- rollback target

## Navigation Model

推奨 primary navigation:

- Overview
- Topology
- Routes
- Destinations
- Traffic
- Recovery
- Security
- Config

## High-Impact Interaction Rule

UI は次の action で explicit confirmation を要求すべきです。

- isolate route
- disable destination
- resend state
- enter safe mode
- start replay
- risky config change の apply

## Replay Interaction Rule

replay UI は少なくとも次を明示すべきです。

- replay target
- replay scope
- replay lineage marking
- sandboxed かどうか

## Change Review Interaction Rule

config apply flow が見せるべきもの:

- 何が変わるか
- validation が何と言っているか
- どんな risk があるか
- rollback path が何か

## Status Consistency

全 screen で同じ top-level state vocabulary を使うべきです。

- `Healthy`
- `Pressured`
- `Degraded`
- `Emergency`
- `SafeMode`

## 非交渉の不変条件

- dashboard は action-oriented であること
- dangerous action は obvious かつ confirmable であること
- replay と live operation を同じ見た目にしないこと
- operator は常に action の scope を把握できること
