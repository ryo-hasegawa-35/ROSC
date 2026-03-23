# Phase 00: 基盤整備

## 目標

後続フェーズの前提になる仕様基準、リポジトリ構成、テスト、ベンチマーク基盤、
互換性ルールを整備する。

## 成果物

- コア、アダプター、ダッシュボード、プラグイン、統合 SDK の workspace 構成
- OSC 1.0 の例に基づく parser / encoder テストベクトル
- bundle、timetag、address pattern の適合テスト
- 互換性マトリクス:
  - OSC 1.0 strict mode
  - type tag 省略を含む legacy tolerance
  - OSC 1.1 を参考にした optional behavior
- ベースラインとなる benchmark harness:
  - packet parse throughput
  - routing latency
  - egress fan-out
  - burst traffic behavior
- malformed packet、nested bundle、invalid type tag 向け fuzz target
- Windows、macOS、Linux 向け cross-platform CI の土台

## このフェーズで決めるべきこと

- 内部イベント表現
- route configuration format
- error handling policy
- memory ownership model
- strict と tolerant parsing mode の切り替え方
- 開発中の dashboard を embedded にするか分離するか

## 非目標

- 本番向け dashboard はまだ作らない
- schema system はまだ作らない
- shared memory IPC はまだ作らない
- zero-trust security はまだ作らない

## 完了条件

- 正規の OSC 1.0 パケットを、意図的な正規化箇所を除き、バイト列を変えずに
  parse / re-emit できる
- message、bundle、timetag、pattern matching の基礎テストがある
- benchmark script と baseline number が再現可能になっている
- crate 構成が 3 OS でビルドできることを CI が示している

## 概算工数

40-80 時間

## このフェーズの重要性

ここが弱いと、その後の機能が曖昧な前提の上に積み上がってしまい、
後方互換性が崩れやすくなります。このフェーズは「正しさ」を偶然ではなく
資産に変えるための土台です。
