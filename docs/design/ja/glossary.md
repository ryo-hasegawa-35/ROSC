# Glossary

## 目的

この glossary は、concept document、design spec、将来の implementation で
用語を一貫させるためのものです。

## Core Term

### Broker

traffic を受け取り、正規化し、routing し、観測し、forward する central
runtime のことです。単なる adapter ではなく、システムの中核を指します。

### Ingress

broker の受信側。transport や adapter から packet / message がシステムへ
入る境界です。

### Egress

broker の送信側。packet または derived message が destination へ出ていく
境界です。

### Route

traffic に match し、必要なら transform を行い、明示的な fault、cache、
recovery、security policy の下で 1 つ以上の destination へ送る declarative
rule です。

### Destination

route の具体的な出力先です。route より粒度が小さく、独自の queueing と
health state を持つべき対象です。

### Adapter

外部の transport / protocol と broker core model を変換する boundary
module です。

### Service

OSC receiver、WebSocket control endpoint、discovery-visible bridge など、
broker または adapter が公開する logical capability です。

### Transport

UDP、TCP、WebSocket、MQTT、shared memory IPC などの delivery mechanism
または communications substrate です。

## Compatibility Term

### `osc1_0_strict`

明示的な type tag string を持つ standards-aligned な OSC 1.0 packet を受理し、
1.0 semantics のみを使う mode。

### `osc1_0_legacy_tolerant`

type tag string を省略した古い packet を受理しつつ、limited-inspection traffic
として正直に扱う mode。

### `osc1_1_extended`

OSC 1.0 baseline を土台に、`//` path traversal など selected extended behavior
を additive に許可する mode。

### Legacy Untyped Message

type tag string を省略しており、typed message と同等の argument-aware 処理を
安全には行えない packet。

### Opaque Packet

forward や replay はできても、broker が完全には inspect / transform できない
packet。

## Runtime Term

### Normalized View

packet が十分安全に inspect できるときに broker が使う internal typed
representation。

### Raw Packet Record

original bytes と ingress metadata を保持する immutable な ingress record。

### Capability Flag

forward、inspect、transform、cache など、どの操作が safe かを broker に
知らせる marker。

### Packet Lineage

original traffic と、derived / transformed / replayed / rehydrated traffic の
関係。

## Failure Term

### Fault Domain

sender、route、destination、adapter、broker など、failure を閉じ込めるべき
最小境界。

### Circuit Breaker

failure が threshold を超えて繰り返されたときに開き、より広いシステムを
守る protective mechanism。

### Quarantine

bad traffic を繰り返す sender や source を cooling-off interval の間隔離する、
より強い protective action。

### Overload State

`Healthy`、`Pressured`、`Degraded`、`Emergency`、`SafeMode` といった、
broker 全体の明示的な operating state。

### Shed

より重要な flow を守るために、pressure 下で work や traffic handling を
意図的に減らすこと。

## Recovery Term

### Cache

selected traffic から導出され、rehydrate、recovery、inspection に使われる
state。

### Rehydrate

full historical traffic を replay せず、cache や snapshot から current state
を戻すこと。

### Replay

debugging、testing、controlled reconstruction のために historical traffic を
再送すること。

### Late Joiner

traffic がすでに流れた後で接続し、state catch-up が必要になる node / client。

### Warm Restart

cache や configuration など selected runtime state を意図的に戻しつつ、
restart 自体は history に残す restart。

## Security Term

### Security Overlay

legacy peer のために raw OSC payload を変えずに、broker が additive に identity、
scope、verification、authorization を適用する layer。

### Scope

project、venue、workstation、namespace などの security / policy boundary。

### Verified Source

secure transport context または secure envelope rule によって identity が
authentication された sender / peer。

### Legacy Bridge

secure ingress を broker で終端し、policy が許せば downstream の legacy tool
へ plain な互換 OSC を forwarding する挙動。

## Discovery Term

### Service Metadata

identity、capability、endpoint reference、trust、freshness などを含む、
discovered または configured service の transport-neutral な記述です。

### Trust Level

observed、claimed、verified、operator approved など、discovered information に
どの程度 confidence を置けるかを示す分類です。

## Operations Term

### Safe Mode

risky な optional feature を切り、最小限有用な compatible system を維持する
reduced-capability mode。

### Topology View

ingress、route、transform、destination の接続関係を見せる dashboard view。

### Playbook

slow destination、malformed traffic storm、restart recovery など、incident
class ごとの predefined operator response pattern。

### Release Profile

`core-osc`、`ops-console`、`secure-installation` など、deployment ごとに
どの feature を含むかを定義した packaged product shape。

### Deployment Topology

localhost sidecar、single workstation hub、active / standby pair、
federated broker network など、繰り返し現れる deployment shape。

## Distributed Term

### Federation

複数 broker が selected traffic または state を交換しつつ distinct peer として
存在する mode。

### High Availability

active broker が落ちたときに別 broker が service を継続できるようにする mode。

### Active / Standby

1 台が active に traffic を捌き、もう 1 台が defined condition で引き継ぐ準備を
する high-availability pair。

## Tooling Term

### Schema

raw OSC compatibility の上に additive に重なる、intended message meaning、
constraint、tooling hint の optional typed description。

### Conformance Vector

specification または合意した compatibility rule に behavior が沿っているかを
確認するための known reference input / output。

### Interoperability Suite

real tool、transport、integration path と組み合わせて product behavior を
検証する scenario-based validation set。

### Adapter SDK

transport / protocol adapter author が external system を broker へ安全に接続する
ための supported integration surface。
