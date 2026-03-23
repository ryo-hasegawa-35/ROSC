# Phase 03: アダプターとディスカバリ

## 目標

生 OSC の互換性を損なわずに、broker をマルチプロトコルハブへ拡張する。

## 成果物

- WebSocket / JSON adapter
- MQTT adapter
- adapter SDK interface
- OSC 1.1 の stream metadata に着想を得た service metadata model:
  - version
  - framing
  - URI
  - supported type tags
- mDNS / DNS-SD discovery
- preset device / application profile
- stream transport support:
  - TCP compatibility framing
  - stream transport 向け SLIP framing
- adapter の health / reconnection management

## 設計ルール

- adapter は意味論を拡張してよいが、core message model 自体は transport
  neutral に保つ
- どの adapter も raw OSC handling の correctness を損なってはならない
- discovery は UX を改善するが、manual static configuration は常に残す

## なぜこのフェーズは observability の後か

複数プロトコルが絡み始めると、切り分けが急に難しくなります。だからこそ、
先に可視化と運用導線を持っておく必要があります。

## 非目標

- 全 endpoint への mandatory auth
- UE5 shared memory
- Wasm user filter runtime

## 完了条件

- browser client が WebSocket 経由で broker を観測または制御できる
- MQTT 接続デバイスが hub 経由でメッセージをやり取りできる
- local network 上の device / service を自動発見できる
- discovery が失敗しても manual operation が阻害されない

## 概算工数

160-260 時間

## 価値

ここでプロジェクトは「優秀な OSC ルーター」から、
「リアルタイムメディア向けメッセージブローカー」へ進化します。
