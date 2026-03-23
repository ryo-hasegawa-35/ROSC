# Phase 01: コアプロキシ

## 目標

主な性能課題をすでに解決できる、実運用を意識したローカル OSC プロキシと
ルーティングエンジンを構築する。

## 成果物

- 複数ポート対応の UDP ingress
- UE5、TouchDesigner、一般的な OSC アプリ向け localhost proxy workflow
- lock を意識した、または lock を最小化した ingress queue
- address ベースの routing engine
- head-of-line blocking を起こさない複数宛先 fan-out
- 宛先または宛先グループごとに独立した egress task
- outbound OSC の厳格 serializer
- 設定可能な route rule:
  - forward
  - drop
  - duplicate
  - rename address
  - static transform
- metrics:
  - packets in / out
  - drops
  - queue depth
  - route hit counts
  - per-destination send latency

## 互換性要件

- UDP OSC 1.0 を default transport mode にする
- 古い送信側の type tag string 省略に対して parser は寛容性を維持する
- address pattern matching は 1.0 semantics をデフォルトにする
- 1.1 専用挙動はデフォルトで無効にする

## エンジニアリング方針

- routing core は内部で正規化した packet view を扱う
- parse と route は transport handling から分離してテストしやすくする
- backpressure behavior は偶然に任せず明示的に定義する
- 問題のある route は、バス全体を詰まらせるより drop / isolate を優先する

## 非目標

- simple metrics endpoint を超える browser dashboard
- MQTT
- WebSocket control plane
- native plugin integration

## 完了条件

- 実アプリ連携で、broker が direct localhost OSC path を置き換えられる
- 1 つの遅い宛先が無関係な宛先を止めない
- 高負荷時の packet loss behavior が測定可能かつ意図的になっている
- bursty な sensor traffic をかけても長時間安定する

## 概算工数

120-200 時間

## 価値

この段階で、すでに単独で使う価値が出ます。高度機能がなくても、
現在のスクリプトベースのルーターより「強い土管」になれます。
